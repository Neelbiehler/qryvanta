use super::*;

impl WorkflowService {
    pub(super) async fn execute_workflow_definition(
        &self,
        actor: &UserIdentity,
        workflow: &WorkflowDefinition,
        trigger_payload: Value,
    ) -> AppResult<WorkflowRun> {
        let run = self
            .repository
            .create_run(
                actor.tenant_id(),
                CreateWorkflowRunInput {
                    workflow_logical_name: workflow.logical_name().as_str().to_owned(),
                    trigger_type: workflow.trigger().trigger_type().to_owned(),
                    trigger_entity_logical_name: workflow
                        .trigger()
                        .entity_logical_name()
                        .map(ToOwned::to_owned),
                    trigger_payload: trigger_payload.clone(),
                },
            )
            .await?;

        self.execute_existing_run(actor, workflow, run.run_id.as_str(), trigger_payload)
            .await
    }

    pub(super) async fn enqueue_workflow_definition(
        &self,
        actor: &UserIdentity,
        workflow: &WorkflowDefinition,
        trigger_payload: Value,
    ) -> AppResult<WorkflowRun> {
        let run = self
            .repository
            .create_run(
                actor.tenant_id(),
                CreateWorkflowRunInput {
                    workflow_logical_name: workflow.logical_name().as_str().to_owned(),
                    trigger_type: workflow.trigger().trigger_type().to_owned(),
                    trigger_entity_logical_name: workflow
                        .trigger()
                        .entity_logical_name()
                        .map(ToOwned::to_owned),
                    trigger_payload,
                },
            )
            .await?;

        self.repository
            .enqueue_run_job(actor.tenant_id(), run.run_id.as_str())
            .await?;

        Ok(run)
    }

    pub(super) async fn execute_existing_run(
        &self,
        actor: &UserIdentity,
        workflow: &WorkflowDefinition,
        run_id: &str,
        trigger_payload: Value,
    ) -> AppResult<WorkflowRun> {
        let action_plan = self.resolve_action_plan(workflow, &trigger_payload)?;
        let mut last_error: Option<String> = None;

        for attempt_number in 1..=i32::from(workflow.max_attempts()) {
            let result = self
                .execute_action_plan(actor, action_plan.as_slice())
                .await;
            let (status, error_message) = match result {
                Ok(()) => (WorkflowRunAttemptStatus::Succeeded, None),
                Err(error) => {
                    let message = error.to_string();
                    last_error = Some(message.clone());
                    (WorkflowRunAttemptStatus::Failed, Some(message))
                }
            };

            self.repository
                .append_run_attempt(
                    actor.tenant_id(),
                    WorkflowRunAttempt {
                        run_id: run_id.to_owned(),
                        attempt_number,
                        status,
                        error_message: error_message.clone(),
                        executed_at: Utc::now(),
                    },
                )
                .await?;

            if status == WorkflowRunAttemptStatus::Succeeded {
                let completed_run = self
                    .repository
                    .complete_run(
                        actor.tenant_id(),
                        CompleteWorkflowRunInput {
                            run_id: run_id.to_owned(),
                            status: WorkflowRunStatus::Succeeded,
                            attempts: attempt_number,
                            dead_letter_reason: None,
                        },
                    )
                    .await?;

                self.append_run_audit(actor, &completed_run).await?;
                return Ok(completed_run);
            }
        }

        let completed_run = self
            .repository
            .complete_run(
                actor.tenant_id(),
                CompleteWorkflowRunInput {
                    run_id: run_id.to_owned(),
                    status: WorkflowRunStatus::DeadLettered,
                    attempts: i32::from(workflow.max_attempts()),
                    dead_letter_reason: last_error,
                },
            )
            .await?;

        self.append_run_audit(actor, &completed_run).await?;
        Ok(completed_run)
    }

    async fn execute_action_plan(
        &self,
        actor: &UserIdentity,
        action_plan: &[WorkflowAction],
    ) -> AppResult<()> {
        for action in action_plan {
            self.execute_action(actor, action).await?;
        }

        Ok(())
    }

    fn resolve_action_plan(
        &self,
        workflow: &WorkflowDefinition,
        trigger_payload: &Value,
    ) -> AppResult<Vec<WorkflowAction>> {
        let Some(steps) = workflow.steps() else {
            return Ok(vec![workflow.action().clone()]);
        };

        let mut action_plan = Vec::new();
        Self::append_step_actions(steps, trigger_payload, &mut action_plan)?;
        Ok(action_plan)
    }

    fn append_step_actions(
        steps: &[WorkflowStep],
        trigger_payload: &Value,
        action_plan: &mut Vec<WorkflowAction>,
    ) -> AppResult<()> {
        for step in steps {
            match step {
                WorkflowStep::LogMessage { message } => {
                    action_plan.push(WorkflowAction::LogMessage {
                        message: message.clone(),
                    });
                }
                WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name,
                    data,
                } => {
                    action_plan.push(WorkflowAction::CreateRuntimeRecord {
                        entity_logical_name: entity_logical_name.clone(),
                        data: data.clone(),
                    });
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
                    let passes = Self::evaluate_condition(
                        trigger_payload,
                        field_path.as_str(),
                        *operator,
                        value.as_ref(),
                    )?;

                    if passes {
                        Self::append_step_actions(
                            then_steps.as_slice(),
                            trigger_payload,
                            action_plan,
                        )?;
                    } else {
                        Self::append_step_actions(
                            else_steps.as_slice(),
                            trigger_payload,
                            action_plan,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    fn evaluate_condition(
        trigger_payload: &Value,
        field_path: &str,
        operator: WorkflowConditionOperator,
        value: Option<&Value>,
    ) -> AppResult<bool> {
        let selected_value = Self::payload_value_by_path(trigger_payload, field_path);
        match operator {
            WorkflowConditionOperator::Exists => Ok(selected_value.is_some()),
            WorkflowConditionOperator::Equals => {
                let expected_value = value.ok_or_else(|| {
                    AppError::Validation(
                        "workflow condition equals operator requires a comparison value".to_owned(),
                    )
                })?;

                Ok(selected_value == Some(expected_value))
            }
            WorkflowConditionOperator::NotEquals => {
                let expected_value = value.ok_or_else(|| {
                    AppError::Validation(
                        "workflow condition not_equals operator requires a comparison value"
                            .to_owned(),
                    )
                })?;

                Ok(selected_value != Some(expected_value))
            }
        }
    }

    fn payload_value_by_path<'a>(payload: &'a Value, field_path: &str) -> Option<&'a Value> {
        let mut current_value = payload;
        for segment in field_path.split('.') {
            if segment.is_empty() {
                return None;
            }

            current_value = current_value.as_object()?.get(segment)?;
        }

        Some(current_value)
    }

    async fn execute_action(&self, actor: &UserIdentity, action: &WorkflowAction) -> AppResult<()> {
        match action {
            WorkflowAction::LogMessage { .. } => Ok(()),
            WorkflowAction::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => {
                self.runtime_record_service
                    .create_runtime_record_unchecked(
                        actor,
                        entity_logical_name.as_str(),
                        data.clone(),
                    )
                    .await?;
                Ok(())
            }
        }
    }

    async fn append_run_audit(&self, actor: &UserIdentity, run: &WorkflowRun) -> AppResult<()> {
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::WorkflowRunCompleted,
                resource_type: "workflow_run".to_owned(),
                resource_id: run.run_id.clone(),
                detail: Some(format!(
                    "workflow '{}' finished with status '{}' after {} attempt(s)",
                    run.workflow_logical_name,
                    run.status.as_str(),
                    run.attempts
                )),
            })
            .await
    }
}
