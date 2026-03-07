use super::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

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
            steps: input.steps,
            max_attempts: input.max_attempts,
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
                    "saved workflow '{}' trigger '{}' with {} step(s)",
                    workflow.logical_name().as_str(),
                    workflow.trigger().trigger_type(),
                    workflow.steps().len()
                )),
            })
            .await?;

        if input.is_enabled {
            return self
                .publish_workflow(actor, workflow.logical_name().as_str())
                .await;
        }

        Ok(workflow)
    }

    /// Publishes the current workflow draft as the next active immutable version.
    pub async fn publish_workflow(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
    ) -> AppResult<WorkflowDefinition> {
        self.require_workflow_manage(actor).await?;
        let publish_errors = self.publish_checks(actor, workflow_logical_name).await?;
        if !publish_errors.is_empty() {
            return Err(AppError::Validation(publish_errors.join("; ")));
        }

        let workflow = self
            .repository
            .publish_workflow(actor.tenant_id(), workflow_logical_name, actor.subject())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::WorkflowPublished,
                resource_type: "workflow_definition".to_owned(),
                resource_id: workflow.logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "published workflow '{}' at version {}",
                    workflow.logical_name().as_str(),
                    workflow.published_version().unwrap_or_default()
                )),
            })
            .await?;

        Ok(workflow)
    }

    /// Disables the currently published workflow version.
    pub async fn disable_workflow(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
    ) -> AppResult<WorkflowDefinition> {
        self.require_workflow_manage(actor).await?;

        let workflow = self
            .repository
            .disable_workflow(actor.tenant_id(), workflow_logical_name)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::WorkflowDisabled,
                resource_type: "workflow_definition".to_owned(),
                resource_id: workflow.logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "disabled workflow '{}' at version {}",
                    workflow.logical_name().as_str(),
                    workflow.published_version().unwrap_or_default()
                )),
            })
            .await?;

        Ok(workflow)
    }

    /// Returns whether publishing this workflow draft requires recent step-up verification.
    pub async fn publish_requires_recent_step_up(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
    ) -> AppResult<bool> {
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

        Ok(workflow.contains_outbound_integration_steps())
    }

    /// Returns whether disabling the active published workflow requires recent step-up verification.
    pub async fn disable_requires_recent_step_up(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
    ) -> AppResult<bool> {
        self.require_workflow_manage(actor).await?;

        let workflow = self
            .repository
            .find_published_workflow(actor.tenant_id(), workflow_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow '{}' does not have a published version for tenant '{}'",
                    workflow_logical_name,
                    actor.tenant_id()
                ))
            })?;

        Ok(workflow.contains_outbound_integration_steps())
    }

    /// Lists workflow definitions.
    pub async fn list_workflows(&self, actor: &UserIdentity) -> AppResult<Vec<WorkflowDefinition>> {
        self.require_workflow_read(actor).await?;
        self.repository.list_workflows(actor.tenant_id()).await
    }

    /// Returns one draft workflow definition by logical name.
    pub async fn find_workflow(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        self.require_workflow_read(actor).await?;
        self.repository
            .find_workflow(actor.tenant_id(), workflow_logical_name)
            .await
    }

    /// Returns one immutable published workflow snapshot by version.
    pub async fn find_published_workflow_version(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        version: i32,
    ) -> AppResult<Option<WorkflowDefinition>> {
        self.require_workflow_read(actor).await?;
        self.repository
            .find_published_workflow_version(actor.tenant_id(), workflow_logical_name, version)
            .await
    }

    /// Runs publish validation checks for one workflow draft.
    pub async fn publish_checks(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
    ) -> AppResult<Vec<String>> {
        self.publish_checks_with_allowed_unpublished_entities(actor, workflow_logical_name, &[])
            .await
    }

    /// Runs publish validation checks while allowing selected unpublished entities.
    pub async fn publish_checks_with_allowed_unpublished_entities(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        allowed_unpublished_entity_logical_names: &[String],
    ) -> AppResult<Vec<String>> {
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

        let allowed_unpublished_entities = allowed_unpublished_entity_logical_names
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let referenced_entities = collect_workflow_entity_references(&workflow);
        let mut errors = Vec::new();

        for entity_logical_name in referenced_entities {
            if allowed_unpublished_entities.contains(entity_logical_name.as_str()) {
                continue;
            }

            let has_published_schema = self
                .runtime_record_service
                .has_published_entity_schema(actor, entity_logical_name.as_str())
                .await?;
            if !has_published_schema {
                errors.push(format!(
                    "dependency check failed: workflow '{}' -> entity '{}' requires a published schema or inclusion in this publish selection",
                    workflow.logical_name().as_str(),
                    entity_logical_name
                ));
            }
        }

        errors.extend(collect_workflow_governance_violations(&workflow));

        Ok(errors)
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
                Permission::WorkflowManage,
            )
            .await
    }

    pub(super) async fn require_workflow_read(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(actor.tenant_id(), actor.subject(), Permission::WorkflowRead)
            .await
    }
}

fn collect_workflow_entity_references(workflow: &WorkflowDefinition) -> Vec<String> {
    let mut referenced_entities = Vec::new();

    if let Some(entity_logical_name) = workflow_entity_reference_from_trigger(workflow.trigger()) {
        referenced_entities.push(entity_logical_name.to_owned());
    }

    collect_step_entity_references(workflow.steps(), &mut referenced_entities);

    let mut unique_entities = Vec::new();
    let mut seen = HashSet::new();
    for entity_logical_name in referenced_entities {
        if seen.insert(entity_logical_name.clone()) {
            unique_entities.push(entity_logical_name);
        }
    }

    unique_entities
}

fn workflow_entity_reference_from_trigger(trigger: &WorkflowTrigger) -> Option<&str> {
    match trigger {
        WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name,
        }
        | WorkflowTrigger::RuntimeRecordUpdated {
            entity_logical_name,
        }
        | WorkflowTrigger::RuntimeRecordDeleted {
            entity_logical_name,
        } => Some(entity_logical_name.as_str()),
        WorkflowTrigger::Manual
        | WorkflowTrigger::ScheduleTick { .. }
        | WorkflowTrigger::WebhookReceived { .. }
        | WorkflowTrigger::FormSubmitted { .. }
        | WorkflowTrigger::InboundEmailReceived { .. }
        | WorkflowTrigger::ApprovalEventReceived { .. } => None,
    }
}

fn collect_step_entity_references(steps: &[WorkflowStep], referenced_entities: &mut Vec<String>) {
    for step in steps {
        match step {
            WorkflowStep::CreateRuntimeRecord {
                entity_logical_name,
                ..
            }
            | WorkflowStep::UpdateRuntimeRecord {
                entity_logical_name,
                ..
            }
            | WorkflowStep::DeleteRuntimeRecord {
                entity_logical_name,
                ..
            }
            | WorkflowStep::AssignOwner {
                entity_logical_name,
                ..
            }
            | WorkflowStep::ApprovalRequest {
                entity_logical_name,
                ..
            } => referenced_entities.push(entity_logical_name.clone()),
            WorkflowStep::Condition {
                then_steps,
                else_steps,
                ..
            } => {
                collect_step_entity_references(then_steps, referenced_entities);
                collect_step_entity_references(else_steps, referenced_entities);
            }
            WorkflowStep::LogMessage { .. }
            | WorkflowStep::SendEmail { .. }
            | WorkflowStep::HttpRequest { .. }
            | WorkflowStep::Webhook { .. }
            | WorkflowStep::Delay { .. } => {}
        }
    }
}

fn collect_workflow_governance_violations(workflow: &WorkflowDefinition) -> Vec<String> {
    let mut violations = Vec::new();
    collect_step_governance_violations(
        workflow.logical_name().as_str(),
        workflow.steps(),
        "",
        &mut violations,
    );
    violations
}

fn collect_step_governance_violations(
    workflow_logical_name: &str,
    steps: &[WorkflowStep],
    path_prefix: &str,
    violations: &mut Vec<String>,
) {
    for (index, step) in steps.iter().enumerate() {
        let step_path = if path_prefix.is_empty() {
            index.to_string()
        } else {
            format!("{path_prefix}.{index}")
        };

        match step {
            WorkflowStep::HttpRequest {
                headers,
                header_secret_refs,
                ..
            }
            | WorkflowStep::Webhook {
                headers,
                header_secret_refs,
                ..
            } => {
                collect_sensitive_header_violations(
                    workflow_logical_name,
                    step_path.as_str(),
                    step.step_type(),
                    headers.as_ref(),
                    header_secret_refs.as_ref(),
                    violations,
                );
            }
            WorkflowStep::Condition {
                then_steps,
                else_steps,
                ..
            } => {
                collect_step_governance_violations(
                    workflow_logical_name,
                    then_steps,
                    format!("{step_path}.then").as_str(),
                    violations,
                );
                collect_step_governance_violations(
                    workflow_logical_name,
                    else_steps,
                    format!("{step_path}.else").as_str(),
                    violations,
                );
            }
            WorkflowStep::LogMessage { .. }
            | WorkflowStep::CreateRuntimeRecord { .. }
            | WorkflowStep::UpdateRuntimeRecord { .. }
            | WorkflowStep::DeleteRuntimeRecord { .. }
            | WorkflowStep::SendEmail { .. }
            | WorkflowStep::AssignOwner { .. }
            | WorkflowStep::ApprovalRequest { .. }
            | WorkflowStep::Delay { .. } => {}
        }
    }
}

fn collect_sensitive_header_violations(
    workflow_logical_name: &str,
    step_path: &str,
    step_type: &str,
    headers: Option<&serde_json::Value>,
    header_secret_refs: Option<&serde_json::Value>,
    violations: &mut Vec<String>,
) {
    let Some(headers) = headers.and_then(serde_json::Value::as_object) else {
        return;
    };

    let secret_ref_header_names = header_secret_refs
        .and_then(serde_json::Value::as_object)
        .map(|map| {
            map.keys()
                .map(|key| key.to_ascii_lowercase())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    for header_name in headers.keys() {
        if is_sensitive_workflow_header_name(header_name.as_str())
            && !secret_ref_header_names.contains(&header_name.to_ascii_lowercase())
        {
            violations.push(format!(
                "workflow governance failed: workflow '{}' step '{}' ({}) uses disallowed inline credential header '{}'; move credentials to secret-managed infrastructure instead",
                workflow_logical_name,
                step_path,
                step_type,
                header_name,
            ));
        }
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
