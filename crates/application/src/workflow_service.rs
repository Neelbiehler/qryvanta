use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use qryvanta_core::{AppError, AppResult, UserIdentity};
use qryvanta_domain::{
    AuditAction, Permission, RuntimeRecord, WorkflowAction, WorkflowConditionOperator,
    WorkflowDefinition, WorkflowStep, WorkflowTrigger,
};
use serde_json::Value;

use crate::metadata_service::MetadataService;
use crate::workflow_ports::{
    CompleteWorkflowRunInput, CreateWorkflowRunInput, SaveWorkflowInput, WorkflowRepository,
    WorkflowRun, WorkflowRunAttempt, WorkflowRunAttemptStatus, WorkflowRunListQuery,
    WorkflowRunStatus, WorkflowRuntimeRecordService,
};
use crate::{AuditEvent, AuditRepository, AuthorizationService};

#[async_trait]
impl WorkflowRuntimeRecordService for MetadataService {
    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.create_runtime_record_unchecked(actor, entity_logical_name, data)
            .await
    }
}

/// Workflow runtime service for trigger dispatch and execution history.
#[derive(Clone)]
pub struct WorkflowService {
    authorization_service: AuthorizationService,
    repository: Arc<dyn WorkflowRepository>,
    runtime_record_service: Arc<dyn WorkflowRuntimeRecordService>,
    audit_repository: Arc<dyn AuditRepository>,
}

impl WorkflowService {
    /// Creates a workflow service.
    #[must_use]
    pub fn new(
        authorization_service: AuthorizationService,
        repository: Arc<dyn WorkflowRepository>,
        runtime_record_service: Arc<dyn WorkflowRuntimeRecordService>,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            authorization_service,
            repository,
            runtime_record_service,
            audit_repository,
        }
    }

    /// Saves one workflow definition.
    pub async fn save_workflow(
        &self,
        actor: &UserIdentity,
        input: SaveWorkflowInput,
    ) -> AppResult<WorkflowDefinition> {
        self.require_workflow_manage(actor).await?;

        let workflow = WorkflowDefinition::new(
            input.logical_name,
            input.display_name,
            input.description,
            input.trigger,
            input.action,
            input.steps,
            input.max_attempts,
            input.is_enabled,
        )?;

        self.repository
            .save_workflow(actor.tenant_id(), workflow.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::WorkflowSaved,
                resource_type: "workflow_definition".to_owned(),
                resource_id: workflow.logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "saved workflow '{}' trigger '{}' action '{}' with {} step(s)",
                    workflow.logical_name().as_str(),
                    workflow.trigger().trigger_type(),
                    workflow.action().action_type(),
                    workflow.effective_steps().len()
                )),
            })
            .await?;

        Ok(workflow)
    }

    /// Lists workflow definitions.
    pub async fn list_workflows(&self, actor: &UserIdentity) -> AppResult<Vec<WorkflowDefinition>> {
        self.require_workflow_read(actor).await?;
        self.repository.list_workflows(actor.tenant_id()).await
    }

    /// Executes a workflow by logical name using manual trigger context.
    pub async fn execute_workflow(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        trigger_payload: Value,
    ) -> AppResult<WorkflowRun> {
        self.require_workflow_manage(actor).await?;

        let workflow = self
            .repository
            .find_workflow(actor.tenant_id(), workflow_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow '{}' does not exist for tenant '{}'",
                    workflow_logical_name,
                    actor.tenant_id()
                ))
            })?;

        if !workflow.is_enabled() {
            return Err(AppError::Conflict(format!(
                "workflow '{}' is disabled",
                workflow.logical_name().as_str()
            )));
        }

        self.execute_workflow_definition(actor, &workflow, trigger_payload)
            .await
    }

    /// Dispatches runtime record created trigger across enabled workflows.
    pub async fn dispatch_runtime_record_created(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<usize> {
        let trigger = WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name: entity_logical_name.to_owned(),
        };
        let workflows = self
            .repository
            .list_enabled_workflows_for_trigger(actor.tenant_id(), &trigger)
            .await?;

        if workflows.is_empty() {
            return Ok(0);
        }

        let workflow_actor = UserIdentity::new(
            "workflow-runtime",
            "workflow-runtime",
            None,
            actor.tenant_id(),
        );

        let mut executed = 0;
        for workflow in workflows {
            let payload = serde_json::json!({
                "entity_logical_name": entity_logical_name,
                "record_id": record_id,
                "triggered_by": actor.subject(),
            });

            if self
                .execute_workflow_definition(&workflow_actor, &workflow, payload)
                .await
                .is_ok()
            {
                executed += 1;
            }
        }

        Ok(executed)
    }

    /// Lists workflow runs for operational traceability.
    pub async fn list_runs(
        &self,
        actor: &UserIdentity,
        query: WorkflowRunListQuery,
    ) -> AppResult<Vec<WorkflowRun>> {
        self.require_workflow_read(actor).await?;
        self.repository.list_runs(actor.tenant_id(), query).await
    }

    /// Lists workflow run attempts for one run.
    pub async fn list_run_attempts(
        &self,
        actor: &UserIdentity,
        run_id: &str,
    ) -> AppResult<Vec<WorkflowRunAttempt>> {
        self.require_workflow_read(actor).await?;
        self.repository
            .list_run_attempts(actor.tenant_id(), run_id)
            .await
    }

    async fn execute_workflow_definition(
        &self,
        actor: &UserIdentity,
        workflow: &WorkflowDefinition,
        trigger_payload: Value,
    ) -> AppResult<WorkflowRun> {
        let action_plan = self.resolve_action_plan(workflow, &trigger_payload)?;

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

        let run_id = run.run_id.clone();
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
                        run_id: run_id.clone(),
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
                            run_id: run_id.clone(),
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
                    run_id,
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

    async fn require_workflow_manage(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await
    }

    async fn require_workflow_read(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await
    }
}

#[cfg(test)]
mod tests;
