use super::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

impl WorkflowService {
    /// Saves one workflow definition.
    pub async fn save_workflow(
        &self,
        actor: &UserIdentity,
        input: SaveWorkflowInput,
    ) -> AppResult<WorkflowDefinition> {
        self.require_workflow_manage(actor).await?;

        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: input.logical_name,
            display_name: input.display_name,
            description: input.description,
            trigger: input.trigger,
            action: input.action,
            steps: input.steps,
            max_attempts: input.max_attempts,
            is_enabled: input.is_enabled,
        })?;

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

    /// Reconstructs one workflow run replay model with deterministic ordering.
    pub async fn replay_run(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        run_id: &str,
    ) -> AppResult<WorkflowRunReplay> {
        self.require_workflow_read(actor).await?;

        let run = self
            .repository
            .find_run(actor.tenant_id(), run_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow run '{}' does not exist for tenant '{}'",
                    run_id,
                    actor.tenant_id()
                ))
            })?;

        if run.workflow_logical_name != workflow_logical_name {
            return Err(AppError::Validation(format!(
                "run '{}' does not belong to workflow '{}'",
                run_id, workflow_logical_name
            )));
        }

        let mut attempts = self
            .repository
            .list_run_attempts(actor.tenant_id(), run_id)
            .await?;
        attempts.sort_by(|left, right| {
            left.attempt_number
                .cmp(&right.attempt_number)
                .then_with(|| left.executed_at.cmp(&right.executed_at))
        });

        let timeline = build_replay_timeline(attempts.as_slice());
        let checksum_sha256 = replay_checksum_sha256(&run, attempts.as_slice())?;

        Ok(WorkflowRunReplay {
            run,
            attempts,
            timeline,
            checksum_sha256,
        })
    }

    pub(super) async fn require_workflow_manage(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await
    }

    pub(super) async fn require_workflow_read(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await
    }
}

fn build_replay_timeline(attempts: &[WorkflowRunAttempt]) -> Vec<WorkflowRunReplayTimelineEvent> {
    let mut timeline = Vec::new();
    let mut sequence = 1_u64;

    for attempt in attempts {
        for step_trace in &attempt.step_traces {
            timeline.push(WorkflowRunReplayTimelineEvent {
                sequence,
                attempt_number: attempt.attempt_number,
                attempt_status: attempt.status,
                attempt_executed_at: attempt.executed_at,
                step_path: step_trace.step_path.clone(),
                step_type: step_trace.step_type.clone(),
                status: step_trace.status.clone(),
                input_payload: step_trace.input_payload.clone(),
                output_payload: step_trace.output_payload.clone(),
                error_message: step_trace.error_message.clone(),
                duration_ms: step_trace.duration_ms,
            });
            sequence = sequence.saturating_add(1);
        }
    }

    timeline
}

fn replay_checksum_sha256(run: &WorkflowRun, attempts: &[WorkflowRunAttempt]) -> AppResult<String> {
    let payload = ReplayChecksumPayload::from_run(run, attempts);
    let encoded = serde_json::to_vec(&payload).map_err(|error| {
        AppError::Internal(format!(
            "failed to serialize workflow replay payload for run '{}': {error}",
            run.run_id
        ))
    })?;
    let digest = Sha256::digest(encoded);

    Ok(digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

#[derive(Serialize)]
struct ReplayChecksumPayload {
    run_id: String,
    workflow_logical_name: String,
    trigger_type: String,
    trigger_entity_logical_name: Option<String>,
    trigger_payload: serde_json::Value,
    status: String,
    attempts: i32,
    dead_letter_reason: Option<String>,
    started_at: String,
    finished_at: Option<String>,
    attempt_rows: Vec<ReplayChecksumAttemptRow>,
}

impl ReplayChecksumPayload {
    fn from_run(run: &WorkflowRun, attempts: &[WorkflowRunAttempt]) -> Self {
        Self {
            run_id: run.run_id.clone(),
            workflow_logical_name: run.workflow_logical_name.clone(),
            trigger_type: run.trigger_type.clone(),
            trigger_entity_logical_name: run.trigger_entity_logical_name.clone(),
            trigger_payload: run.trigger_payload.clone(),
            status: run.status.as_str().to_owned(),
            attempts: run.attempts,
            dead_letter_reason: run.dead_letter_reason.clone(),
            started_at: run.started_at.to_rfc3339(),
            finished_at: run.finished_at.map(|timestamp| timestamp.to_rfc3339()),
            attempt_rows: attempts
                .iter()
                .map(ReplayChecksumAttemptRow::from_attempt)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ReplayChecksumAttemptRow {
    attempt_number: i32,
    status: String,
    error_message: Option<String>,
    executed_at: String,
    step_traces: Vec<ReplayChecksumStepTraceRow>,
}

impl ReplayChecksumAttemptRow {
    fn from_attempt(attempt: &WorkflowRunAttempt) -> Self {
        Self {
            attempt_number: attempt.attempt_number,
            status: attempt.status.as_str().to_owned(),
            error_message: attempt.error_message.clone(),
            executed_at: attempt.executed_at.to_rfc3339(),
            step_traces: attempt
                .step_traces
                .iter()
                .map(ReplayChecksumStepTraceRow::from_step_trace)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ReplayChecksumStepTraceRow {
    step_path: String,
    step_type: String,
    status: String,
    input_payload: serde_json::Value,
    output_payload: serde_json::Value,
    error_message: Option<String>,
    duration_ms: Option<u64>,
}

impl ReplayChecksumStepTraceRow {
    fn from_step_trace(step_trace: &WorkflowRunStepTrace) -> Self {
        Self {
            step_path: step_trace.step_path.clone(),
            step_type: step_trace.step_type.clone(),
            status: step_trace.status.clone(),
            input_payload: step_trace.input_payload.clone(),
            output_payload: step_trace.output_payload.clone(),
            error_message: step_trace.error_message.clone(),
            duration_ms: step_trace.duration_ms,
        }
    }
}
