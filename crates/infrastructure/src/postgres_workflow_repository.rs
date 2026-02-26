use async_trait::async_trait;
use qryvanta_application::{
    ClaimedWorkflowJob, CompleteWorkflowRunInput, CreateWorkflowRunInput, WorkflowClaimPartition,
    WorkflowQueueStats, WorkflowQueueStatsQuery, WorkflowRepository, WorkflowRun,
    WorkflowRunAttempt, WorkflowRunAttemptStatus, WorkflowRunListQuery, WorkflowRunStatus,
    WorkflowWorkerHeartbeatInput,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    WorkflowAction, WorkflowDefinition, WorkflowDefinitionInput, WorkflowStep, WorkflowTrigger,
};
use serde_json::Value;
use sqlx::{FromRow, PgPool};

/// PostgreSQL-backed workflow repository.
#[derive(Clone)]
pub struct PostgresWorkflowRepository {
    pool: PgPool,
}

impl PostgresWorkflowRepository {
    /// Creates a workflow repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct WorkflowDefinitionRow {
    logical_name: String,
    display_name: String,
    description: Option<String>,
    trigger_type: String,
    trigger_entity_logical_name: Option<String>,
    action_type: String,
    action_entity_logical_name: Option<String>,
    action_payload: Value,
    action_steps: Option<Value>,
    max_attempts: i16,
    is_enabled: bool,
}

#[derive(Debug, FromRow)]
struct WorkflowRunRow {
    id: uuid::Uuid,
    workflow_logical_name: String,
    trigger_type: String,
    trigger_entity_logical_name: Option<String>,
    trigger_payload: Value,
    status: String,
    attempts: i32,
    dead_letter_reason: Option<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow)]
struct WorkflowRunAttemptRow {
    run_id: uuid::Uuid,
    attempt_number: i32,
    status: String,
    error_message: Option<String>,
    executed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
struct ClaimedWorkflowJobRow {
    job_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    run_id: uuid::Uuid,
    lease_token: String,
    trigger_payload: Value,
    logical_name: String,
    display_name: String,
    description: Option<String>,
    trigger_type: String,
    trigger_entity_logical_name: Option<String>,
    action_type: String,
    action_entity_logical_name: Option<String>,
    action_payload: Value,
    action_steps: Option<Value>,
    max_attempts: i16,
    is_enabled: bool,
}

#[derive(Debug, FromRow)]
struct WorkflowQueueStatsRow {
    pending_jobs: i64,
    leased_jobs: i64,
    completed_jobs: i64,
    failed_jobs: i64,
    expired_leases: i64,
}

mod definitions;
mod queue;
mod runs;

#[async_trait]
impl WorkflowRepository for PostgresWorkflowRepository {
    async fn save_workflow(
        &self,
        tenant_id: TenantId,
        workflow: WorkflowDefinition,
    ) -> AppResult<()> {
        self.save_workflow_impl(tenant_id, workflow).await
    }

    async fn list_workflows(&self, tenant_id: TenantId) -> AppResult<Vec<WorkflowDefinition>> {
        self.list_workflows_impl(tenant_id).await
    }

    async fn find_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        self.find_workflow_impl(tenant_id, logical_name).await
    }

    async fn list_enabled_workflows_for_trigger(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        self.list_enabled_workflows_for_trigger_impl(tenant_id, trigger)
            .await
    }

    async fn create_run(
        &self,
        tenant_id: TenantId,
        input: CreateWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        self.create_run_impl(tenant_id, input).await
    }

    async fn enqueue_run_job(&self, tenant_id: TenantId, run_id: &str) -> AppResult<()> {
        self.enqueue_run_job_impl(tenant_id, run_id).await
    }

    async fn claim_jobs(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        partition: Option<WorkflowClaimPartition>,
    ) -> AppResult<Vec<ClaimedWorkflowJob>> {
        self.claim_jobs_impl(worker_id, limit, lease_seconds, partition)
            .await
    }

    async fn complete_job(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        self.complete_job_impl(tenant_id, job_id, worker_id, lease_token)
            .await
    }

    async fn fail_job(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        self.fail_job_impl(tenant_id, job_id, worker_id, lease_token, error_message)
            .await
    }

    async fn upsert_worker_heartbeat(
        &self,
        worker_id: &str,
        input: WorkflowWorkerHeartbeatInput,
    ) -> AppResult<()> {
        self.upsert_worker_heartbeat_impl(worker_id, input).await
    }

    async fn queue_stats(&self, query: WorkflowQueueStatsQuery) -> AppResult<WorkflowQueueStats> {
        self.queue_stats_impl(query).await
    }

    async fn append_run_attempt(
        &self,
        tenant_id: TenantId,
        attempt: WorkflowRunAttempt,
    ) -> AppResult<()> {
        self.append_run_attempt_impl(tenant_id, attempt).await
    }

    async fn complete_run(
        &self,
        tenant_id: TenantId,
        input: CompleteWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        self.complete_run_impl(tenant_id, input).await
    }

    async fn list_runs(
        &self,
        tenant_id: TenantId,
        query: WorkflowRunListQuery,
    ) -> AppResult<Vec<WorkflowRun>> {
        self.list_runs_impl(tenant_id, query).await
    }

    async fn list_run_attempts(
        &self,
        tenant_id: TenantId,
        run_id: &str,
    ) -> AppResult<Vec<WorkflowRunAttempt>> {
        self.list_run_attempts_impl(tenant_id, run_id).await
    }
}

fn workflow_definition_from_row(row: WorkflowDefinitionRow) -> AppResult<WorkflowDefinition> {
    WorkflowDefinition::new(WorkflowDefinitionInput {
        logical_name: row.logical_name,
        display_name: row.display_name,
        description: row.description,
        trigger: workflow_trigger_from_parts(
            row.trigger_type.as_str(),
            row.trigger_entity_logical_name.as_deref(),
        )?,
        action: workflow_action_from_parts(
            row.action_type.as_str(),
            row.action_entity_logical_name.as_deref(),
            row.action_payload,
        )?,
        steps: workflow_steps_from_json(row.action_steps)?,
        max_attempts: u16::try_from(row.max_attempts).map_err(|error| {
            AppError::Validation(format!("invalid workflow max_attempts value: {error}"))
        })?,
        is_enabled: row.is_enabled,
    })
}

fn workflow_steps_to_json(steps: Option<&[WorkflowStep]>) -> AppResult<Option<Value>> {
    let Some(steps) = steps else {
        return Ok(None);
    };

    serde_json::to_value(steps).map(Some).map_err(|error| {
        AppError::Validation(format!(
            "failed to serialize workflow action_steps: {error}"
        ))
    })
}

fn workflow_steps_from_json(value: Option<Value>) -> AppResult<Option<Vec<WorkflowStep>>> {
    let Some(value) = value else {
        return Ok(None);
    };

    serde_json::from_value(value).map(Some).map_err(|error| {
        AppError::Validation(format!(
            "failed to deserialize workflow action_steps: {error}"
        ))
    })
}

fn workflow_trigger_parts(trigger: &WorkflowTrigger) -> (&'static str, Option<&str>) {
    match trigger {
        WorkflowTrigger::Manual => ("manual", None),
        WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name,
        } => ("runtime_record_created", Some(entity_logical_name.as_str())),
    }
}

fn workflow_action_parts(action: &WorkflowAction) -> (&'static str, Option<&str>, Value) {
    match action {
        WorkflowAction::LogMessage { message } => {
            ("log_message", None, serde_json::json!({"message": message}))
        }
        WorkflowAction::CreateRuntimeRecord {
            entity_logical_name,
            data,
        } => (
            "create_runtime_record",
            Some(entity_logical_name.as_str()),
            data.clone(),
        ),
    }
}

fn workflow_trigger_from_parts(
    trigger_type: &str,
    trigger_entity_logical_name: Option<&str>,
) -> AppResult<WorkflowTrigger> {
    match trigger_type {
        "manual" => Ok(WorkflowTrigger::Manual),
        "runtime_record_created" => {
            let entity_logical_name = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "runtime_record_created trigger requires trigger_entity_logical_name"
                        .to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: entity_logical_name.to_owned(),
            })
        }
        _ => Err(AppError::Validation(format!(
            "unknown workflow trigger_type '{trigger_type}'"
        ))),
    }
}

fn workflow_action_from_parts(
    action_type: &str,
    action_entity_logical_name: Option<&str>,
    action_payload: Value,
) -> AppResult<WorkflowAction> {
    match action_type {
        "log_message" => {
            let message = action_payload
                .as_object()
                .and_then(|payload| payload.get("message"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    AppError::Validation(
                        "log_message action payload requires string field 'message'".to_owned(),
                    )
                })?;

            Ok(WorkflowAction::LogMessage {
                message: message.to_owned(),
            })
        }
        "create_runtime_record" => {
            let entity_logical_name = action_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "create_runtime_record action requires action_entity_logical_name".to_owned(),
                )
            })?;

            Ok(WorkflowAction::CreateRuntimeRecord {
                entity_logical_name: entity_logical_name.to_owned(),
                data: action_payload,
            })
        }
        _ => Err(AppError::Validation(format!(
            "unknown workflow action_type '{action_type}'"
        ))),
    }
}

fn claimed_workflow_job_from_row(row: ClaimedWorkflowJobRow) -> AppResult<ClaimedWorkflowJob> {
    let tenant_uuid = row.tenant_id;
    let workflow = workflow_definition_from_row(WorkflowDefinitionRow {
        logical_name: row.logical_name,
        display_name: row.display_name,
        description: row.description,
        trigger_type: row.trigger_type,
        trigger_entity_logical_name: row.trigger_entity_logical_name,
        action_type: row.action_type,
        action_entity_logical_name: row.action_entity_logical_name,
        action_payload: row.action_payload,
        action_steps: row.action_steps,
        max_attempts: row.max_attempts,
        is_enabled: row.is_enabled,
    })?;

    Ok(ClaimedWorkflowJob {
        job_id: row.job_id.to_string(),
        tenant_id: TenantId::from_uuid(tenant_uuid),
        run_id: row.run_id.to_string(),
        workflow,
        trigger_payload: row.trigger_payload,
        lease_token: row.lease_token,
    })
}

fn workflow_run_from_row(row: WorkflowRunRow) -> AppResult<WorkflowRun> {
    Ok(WorkflowRun {
        run_id: row.id.to_string(),
        workflow_logical_name: row.workflow_logical_name,
        trigger_type: row.trigger_type,
        trigger_entity_logical_name: row.trigger_entity_logical_name,
        trigger_payload: row.trigger_payload,
        status: WorkflowRunStatus::parse(row.status.as_str())?,
        attempts: row.attempts,
        dead_letter_reason: row.dead_letter_reason,
        started_at: row.started_at,
        finished_at: row.finished_at,
    })
}

fn workflow_run_attempt_from_row(row: WorkflowRunAttemptRow) -> AppResult<WorkflowRunAttempt> {
    Ok(WorkflowRunAttempt {
        run_id: row.run_id.to_string(),
        attempt_number: row.attempt_number,
        status: WorkflowRunAttemptStatus::parse(row.status.as_str())?,
        error_message: row.error_message,
        executed_at: row.executed_at,
    })
}
