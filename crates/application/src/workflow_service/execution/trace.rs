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

        self.execute_steps_with_trace(actor, workflow.steps(), context, "", &mut traces)
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
        let step = Self::step_by_path(workflow.steps(), step_path)?;
        match step {
            WorkflowStep::LogMessage { .. }
            | WorkflowStep::CreateRuntimeRecord { .. }
            | WorkflowStep::UpdateRuntimeRecord { .. }
            | WorkflowStep::DeleteRuntimeRecord { .. }
            | WorkflowStep::SendEmail { .. }
            | WorkflowStep::HttpRequest { .. }
            | WorkflowStep::Webhook { .. }
            | WorkflowStep::AssignOwner { .. }
            | WorkflowStep::ApprovalRequest { .. }
            | WorkflowStep::Delay { .. } => self
                .execute_step_with_trace(actor, step, context, step_path, traces)
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
                    WorkflowStep::LogMessage { .. }
                    | WorkflowStep::CreateRuntimeRecord { .. }
                    | WorkflowStep::UpdateRuntimeRecord { .. }
                    | WorkflowStep::DeleteRuntimeRecord { .. }
                    | WorkflowStep::SendEmail { .. }
                    | WorkflowStep::HttpRequest { .. }
                    | WorkflowStep::Webhook { .. }
                    | WorkflowStep::AssignOwner { .. }
                    | WorkflowStep::ApprovalRequest { .. }
                    | WorkflowStep::Delay { .. } => {
                        self.execute_step_with_trace(
                            actor,
                            step,
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

    pub(super) async fn execute_step_with_trace(
        &self,
        actor: &UserIdentity,
        step: &WorkflowStep,
        context: WorkflowExecutionContext<'_>,
        step_path: &str,
        traces: &mut Vec<WorkflowRunStepTrace>,
    ) -> Result<(), WorkflowExecutionErrorWithTrace> {
        let resolved_step = Self::interpolate_step(step, context).map_err(|error| {
            WorkflowExecutionErrorWithTrace {
                error,
                step_traces: traces.clone(),
            }
        })?;
        let step_type = resolved_step.step_type().to_owned();
        let input_payload = context.trigger_payload.clone();
        let output_payload = match &resolved_step {
            WorkflowStep::LogMessage { message } => {
                serde_json::json!({ "message": message })
            }
            WorkflowStep::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => {
                serde_json::json!({
                    "entity_logical_name": entity_logical_name,
                    "data": data,
                })
            }
            WorkflowStep::UpdateRuntimeRecord {
                entity_logical_name,
                record_id,
                data,
            } => {
                serde_json::json!({
                    "entity_logical_name": entity_logical_name,
                    "record_id": record_id,
                    "data": data,
                })
            }
            WorkflowStep::DeleteRuntimeRecord {
                entity_logical_name,
                record_id,
            } => {
                serde_json::json!({
                    "entity_logical_name": entity_logical_name,
                    "record_id": record_id,
                })
            }
            WorkflowStep::SendEmail {
                to,
                subject,
                body,
                html_body,
            } => {
                serde_json::json!({
                    "to": to,
                    "subject": subject,
                    "body": body,
                    "html_body": html_body,
                })
            }
            WorkflowStep::HttpRequest {
                method,
                url,
                headers,
                header_secret_refs,
                body,
            } => {
                serde_json::json!({
                    "method": method,
                    "url": url,
                    "headers": redact_sensitive_workflow_headers(headers.as_ref()),
                    "header_secret_refs": redact_workflow_header_secret_refs(header_secret_refs.as_ref()),
                    "body": body,
                })
            }
            WorkflowStep::Webhook {
                endpoint,
                event,
                headers,
                header_secret_refs,
                payload,
            } => {
                serde_json::json!({
                    "endpoint": endpoint,
                    "event": event,
                    "headers": redact_sensitive_workflow_headers(headers.as_ref()),
                    "header_secret_refs": redact_workflow_header_secret_refs(header_secret_refs.as_ref()),
                    "payload": payload,
                })
            }
            WorkflowStep::AssignOwner {
                entity_logical_name,
                record_id,
                owner_id,
                reason,
            } => {
                serde_json::json!({
                    "entity_logical_name": entity_logical_name,
                    "record_id": record_id,
                    "owner_id": owner_id,
                    "reason": reason,
                })
            }
            WorkflowStep::ApprovalRequest {
                entity_logical_name,
                record_id,
                request_type,
                requested_by,
                approver_id,
                reason,
                payload,
            } => {
                serde_json::json!({
                    "entity_logical_name": entity_logical_name,
                    "record_id": record_id,
                    "request_type": request_type,
                    "requested_by": requested_by,
                    "approver_id": approver_id,
                    "reason": reason,
                    "payload": payload,
                })
            }
            WorkflowStep::Delay {
                duration_ms,
                reason,
            } => {
                serde_json::json!({
                    "duration_ms": duration_ms,
                    "reason": reason,
                })
            }
            WorkflowStep::Condition { .. } => {
                return Err(WorkflowExecutionErrorWithTrace {
                    error: AppError::Validation(
                        "condition step cannot execute as an action".to_owned(),
                    ),
                    step_traces: traces.clone(),
                });
            }
        };

        let started_at = Instant::now();
        match self
            .execute_resolved_step(actor, &resolved_step, context, step_path)
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
