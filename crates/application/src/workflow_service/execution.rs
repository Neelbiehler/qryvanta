use super::*;
use crate::workflow_ports::WorkflowRunStepTrace;

mod actions;
mod trace;
mod values;

#[derive(Clone, Copy)]
struct WorkflowExecutionContext<'a> {
    trigger_payload: &'a Value,
    trigger_type: &'a str,
    trigger_entity_logical_name: Option<&'a str>,
    run_id: &'a str,
    attempt_number: i32,
}

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
        let mut last_error: Option<String> = None;

        for attempt_number in 1..=i32::from(workflow.max_attempts()) {
            let context = WorkflowExecutionContext {
                trigger_payload: &trigger_payload,
                trigger_type: workflow.trigger().trigger_type(),
                trigger_entity_logical_name: workflow.trigger().entity_logical_name(),
                run_id,
                attempt_number,
            };
            let attempt_result = self
                .execute_workflow_steps_with_trace(actor, workflow, context)
                .await;
            let (status, error_message, step_traces) = match attempt_result {
                Ok(step_traces) => (
                    WorkflowRunAttemptStatus::Succeeded,
                    None::<String>,
                    step_traces,
                ),
                Err(error_with_trace) => {
                    let message = error_with_trace.error.to_string();
                    last_error = Some(message.clone());
                    (
                        WorkflowRunAttemptStatus::Failed,
                        Some(message),
                        error_with_trace.step_traces,
                    )
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
                        step_traces,
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

    pub(super) async fn retry_step_for_run(
        &self,
        actor: &UserIdentity,
        workflow: &WorkflowDefinition,
        run: &WorkflowRun,
        step_path: &str,
    ) -> AppResult<WorkflowRun> {
        let attempt_number = run.attempts + 1;
        let mut traces = Vec::new();

        let result = self
            .execute_single_step_path_with_trace(
                actor,
                workflow,
                WorkflowExecutionContext {
                    trigger_payload: &run.trigger_payload,
                    trigger_type: workflow.trigger().trigger_type(),
                    trigger_entity_logical_name: workflow.trigger().entity_logical_name(),
                    run_id: run.run_id.as_str(),
                    attempt_number,
                },
                step_path,
                &mut traces,
            )
            .await;

        let (attempt_status, error_message, run_status) = match result {
            Ok(()) => (
                WorkflowRunAttemptStatus::Succeeded,
                None,
                WorkflowRunStatus::Succeeded,
            ),
            Err(error) => {
                let message = error.to_string();
                (
                    WorkflowRunAttemptStatus::Failed,
                    Some(message),
                    WorkflowRunStatus::DeadLettered,
                )
            }
        };

        self.repository
            .append_run_attempt(
                actor.tenant_id(),
                WorkflowRunAttempt {
                    run_id: run.run_id.clone(),
                    attempt_number,
                    status: attempt_status,
                    error_message: error_message.clone(),
                    executed_at: Utc::now(),
                    step_traces: traces,
                },
            )
            .await?;

        let completed_run = self
            .repository
            .complete_run(
                actor.tenant_id(),
                CompleteWorkflowRunInput {
                    run_id: run.run_id.clone(),
                    status: run_status,
                    attempts: attempt_number,
                    dead_letter_reason: error_message,
                },
            )
            .await?;

        self.append_run_audit(actor, &completed_run).await?;
        Ok(completed_run)
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

#[derive(Debug)]
struct WorkflowExecutionErrorWithTrace {
    error: AppError,
    step_traces: Vec<WorkflowRunStepTrace>,
}
