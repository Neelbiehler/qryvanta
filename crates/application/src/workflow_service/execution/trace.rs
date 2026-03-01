use super::*;

use std::time::Instant;

impl WorkflowService {
    pub(super) async fn execute_workflow_steps_with_trace(
        &self,
        actor: &UserIdentity,
        workflow: &WorkflowDefinition,
        context: WorkflowExecutionContext<'_>,
    ) -> Result<Vec<WorkflowRunStepTrace>, WorkflowExecutionErrorWithTrace> {
        let mut traces = Vec::new();

        if let Some(steps) = workflow.steps() {
            self.execute_steps_with_trace(actor, steps, context, "", &mut traces)
                .await?;
            return Ok(traces);
        }

        self.execute_action_with_trace(actor, workflow.action(), context, "0", &mut traces)
            .await?;
        Ok(traces)
    }

    pub(super) async fn execute_single_step_path_with_trace(
        &self,
        actor: &UserIdentity,
        workflow: &WorkflowDefinition,
        context: WorkflowExecutionContext<'_>,
        step_path: &str,
        traces: &mut Vec<WorkflowRunStepTrace>,
    ) -> AppResult<()> {
        let Some(steps) = workflow.steps() else {
            return self.execute_action(actor, workflow.action()).await;
        };

        let step = Self::step_by_path(steps, step_path)?;
        match step {
            WorkflowStep::LogMessage { message } => self
                .execute_action_with_trace(
                    actor,
                    &WorkflowAction::LogMessage {
                        message: message.clone(),
                    },
                    context,
                    step_path,
                    traces,
                )
                .await
                .map_err(|error| error.error),
            WorkflowStep::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => self
                .execute_action_with_trace(
                    actor,
                    &WorkflowAction::CreateRuntimeRecord {
                        entity_logical_name: entity_logical_name.clone(),
                        data: data.clone(),
                    },
                    context,
                    step_path,
                    traces,
                )
                .await
                .map_err(|error| error.error),
            WorkflowStep::Condition {
                field_path,
                operator,
                value,
                then_label: _,
                else_label: _,
                then_steps,
                else_steps,
            } => {
                let started_at = Instant::now();
                let resolved_value = value
                    .as_ref()
                    .map(|selected_value| Self::interpolate_json_value(selected_value, context))
                    .transpose()?;
                let passes = Self::evaluate_condition(
                    context.trigger_payload,
                    field_path.as_str(),
                    *operator,
                    resolved_value.as_ref(),
                )?;

                traces.push(WorkflowRunStepTrace {
                    step_path: step_path.to_owned(),
                    step_type: "condition".to_owned(),
                    status: "succeeded".to_owned(),
                    input_payload: serde_json::json!({
                        "field_path": field_path,
                        "operator": format!("{:?}", operator).to_lowercase(),
                        "value": resolved_value,
                    }),
                    output_payload: serde_json::json!({
                        "passes": passes,
                    }),
                    error_message: None,
                    duration_ms: Some(started_at.elapsed().as_millis() as u64),
                });

                if passes {
                    let then_prefix = format!("{}.then", step_path);
                    self.execute_steps_with_trace(
                        actor,
                        then_steps.as_slice(),
                        context,
                        then_prefix.as_str(),
                        traces,
                    )
                    .await
                    .map_err(|error| error.error)
                } else {
                    let else_prefix = format!("{}.else", step_path);
                    self.execute_steps_with_trace(
                        actor,
                        else_steps.as_slice(),
                        context,
                        else_prefix.as_str(),
                        traces,
                    )
                    .await
                    .map_err(|error| error.error)
                }
            }
        }
    }

    pub(super) fn execute_steps_with_trace<'a>(
        &'a self,
        actor: &'a UserIdentity,
        steps: &'a [WorkflowStep],
        context: WorkflowExecutionContext<'a>,
        path_prefix: &'a str,
        traces: &'a mut Vec<WorkflowRunStepTrace>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), WorkflowExecutionErrorWithTrace>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            for (index, step) in steps.iter().enumerate() {
                let step_path = if path_prefix.is_empty() {
                    index.to_string()
                } else {
                    format!("{path_prefix}.{index}")
                };

                match step {
                    WorkflowStep::LogMessage { message } => {
                        self.execute_action_with_trace(
                            actor,
                            &WorkflowAction::LogMessage {
                                message: message.clone(),
                            },
                            context,
                            step_path.as_str(),
                            traces,
                        )
                        .await?;
                    }
                    WorkflowStep::CreateRuntimeRecord {
                        entity_logical_name,
                        data,
                    } => {
                        self.execute_action_with_trace(
                            actor,
                            &WorkflowAction::CreateRuntimeRecord {
                                entity_logical_name: entity_logical_name.clone(),
                                data: data.clone(),
                            },
                            context,
                            step_path.as_str(),
                            traces,
                        )
                        .await?;
                    }
                    WorkflowStep::Condition {
                        field_path,
                        operator,
                        value,
                        then_label: _,
                        else_label: _,
                        then_steps,
                        else_steps,
                    } => {
                        let condition_started_at = Instant::now();
                        let resolved_value = value
                            .as_ref()
                            .map(|selected_value| {
                                Self::interpolate_json_value(selected_value, context)
                            })
                            .transpose()
                            .map_err(|error| WorkflowExecutionErrorWithTrace {
                                error,
                                step_traces: traces.clone(),
                            })?;
                        let passes = Self::evaluate_condition(
                            context.trigger_payload,
                            field_path.as_str(),
                            *operator,
                            resolved_value.as_ref(),
                        )
                        .map_err(|error| {
                            WorkflowExecutionErrorWithTrace {
                                error,
                                step_traces: traces.clone(),
                            }
                        })?;

                        let condition_duration_ms =
                            condition_started_at.elapsed().as_millis() as u64;

                        traces.push(WorkflowRunStepTrace {
                            step_path: step_path.clone(),
                            step_type: "condition".to_owned(),
                            status: "succeeded".to_owned(),
                            input_payload: serde_json::json!({
                                "field_path": field_path,
                                "operator": format!("{:?}", operator).to_lowercase(),
                                "value": resolved_value,
                            }),
                            output_payload: serde_json::json!({
                                "passes": passes,
                            }),
                            error_message: None,
                            duration_ms: Some(condition_duration_ms),
                        });

                        if passes {
                            let then_prefix = format!("{}.then", step_path);
                            self.execute_steps_with_trace(
                                actor,
                                then_steps.as_slice(),
                                context,
                                then_prefix.as_str(),
                                traces,
                            )
                            .await?;
                        } else {
                            let else_prefix = format!("{}.else", step_path);
                            self.execute_steps_with_trace(
                                actor,
                                else_steps.as_slice(),
                                context,
                                else_prefix.as_str(),
                                traces,
                            )
                            .await?;
                        }
                    }
                }
            }

            Ok(())
        })
    }

    pub(super) async fn execute_action_with_trace(
        &self,
        actor: &UserIdentity,
        action: &WorkflowAction,
        context: WorkflowExecutionContext<'_>,
        step_path: &str,
        traces: &mut Vec<WorkflowRunStepTrace>,
    ) -> Result<(), WorkflowExecutionErrorWithTrace> {
        let resolved_action = Self::interpolate_action(action, context).map_err(|error| {
            WorkflowExecutionErrorWithTrace {
                error,
                step_traces: traces.clone(),
            }
        })?;
        let step_type = resolved_action.action_type().to_owned();
        let input_payload = context.trigger_payload.clone();
        let output_payload = match &resolved_action {
            WorkflowAction::LogMessage { message } => {
                serde_json::json!({ "message": message })
            }
            WorkflowAction::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => {
                serde_json::json!({
                    "entity_logical_name": entity_logical_name,
                    "data": data,
                })
            }
        };

        let started_at = Instant::now();
        match self
            .execute_resolved_action(actor, &resolved_action, context, step_path)
            .await
        {
            Ok(()) => {
                traces.push(WorkflowRunStepTrace {
                    step_path: step_path.to_owned(),
                    step_type,
                    status: "succeeded".to_owned(),
                    input_payload,
                    output_payload,
                    error_message: None,
                    duration_ms: Some(started_at.elapsed().as_millis() as u64),
                });

                Ok(())
            }
            Err(error) => {
                let message = error.to_string();
                traces.push(WorkflowRunStepTrace {
                    step_path: step_path.to_owned(),
                    step_type,
                    status: "failed".to_owned(),
                    input_payload,
                    output_payload,
                    error_message: Some(message),
                    duration_ms: Some(started_at.elapsed().as_millis() as u64),
                });

                Err(WorkflowExecutionErrorWithTrace {
                    error,
                    step_traces: traces.clone(),
                })
            }
        }
    }
}
