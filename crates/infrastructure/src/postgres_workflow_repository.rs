use crate::{begin_tenant_transaction, begin_workflow_worker_transaction};
use async_trait::async_trait;
use qryvanta_application::{
    ClaimedWorkflowJob, ClaimedWorkflowScheduleTick, CompleteWorkflowRunInput,
    CreateWorkflowRunInput, WorkflowClaimPartition, WorkflowQueueStats, WorkflowQueueStatsQuery,
    WorkflowRepository, WorkflowRun, WorkflowRunAttempt, WorkflowRunAttemptStatus,
    WorkflowRunListQuery, WorkflowRunStatus, WorkflowRunStepTrace, WorkflowScheduledTrigger,
    WorkflowWorkerHeartbeatInput,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    WorkflowDefinition, WorkflowDefinitionInput, WorkflowLifecycleState, WorkflowStep,
    WorkflowTrigger,
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
    steps: Value,
    max_attempts: i16,
    lifecycle_state: String,
    current_published_version: Option<i32>,
}

#[derive(Debug, FromRow)]
struct WorkflowRunRow {
    id: uuid::Uuid,
    workflow_logical_name: String,
    workflow_version: i32,
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
    step_traces: Value,
}

#[derive(Debug, FromRow)]
struct ClaimedWorkflowJobRow {
    job_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    run_id: uuid::Uuid,
    workflow_version: i32,
    lease_token: String,
    trigger_payload: Value,
    logical_name: String,
    display_name: String,
    description: Option<String>,
    trigger_type: String,
    trigger_entity_logical_name: Option<String>,
    steps: Value,
    max_attempts: i16,
    lifecycle_state: String,
    current_published_version: Option<i32>,
}

#[derive(Debug, FromRow)]
struct WorkflowQueueStatsRow {
    pending_jobs: i64,
    leased_jobs: i64,
    completed_jobs: i64,
    failed_jobs: i64,
    expired_leases: i64,
}

#[derive(Debug, FromRow)]
struct WorkflowScheduledTriggerRow {
    tenant_id: uuid::Uuid,
    schedule_key: String,
}

#[derive(Debug, FromRow)]
struct ClaimedWorkflowScheduleTickRow {
    tenant_id: uuid::Uuid,
    schedule_key: String,
    slot_key: String,
    scheduled_for: chrono::DateTime<chrono::Utc>,
    leased_by: String,
    lease_token: String,
}

mod definitions;
mod queue;
mod runs;

#[cfg(test)]
mod tests;

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

    async fn find_published_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        self.find_published_workflow_impl(tenant_id, logical_name)
            .await
    }

    async fn find_published_workflow_version(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        version: i32,
    ) -> AppResult<Option<WorkflowDefinition>> {
        self.find_published_workflow_version_impl(tenant_id, logical_name, version)
            .await
    }

    async fn publish_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        published_by: &str,
    ) -> AppResult<WorkflowDefinition> {
        self.publish_workflow_impl(tenant_id, logical_name, published_by)
            .await
    }

    async fn disable_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<WorkflowDefinition> {
        self.disable_workflow_impl(tenant_id, logical_name).await
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

    async fn list_enabled_schedule_triggers(
        &self,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<WorkflowScheduledTrigger>> {
        self.list_enabled_schedule_triggers_impl(tenant_filter)
            .await
    }

    async fn claim_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        scheduled_for: chrono::DateTime<chrono::Utc>,
        worker_id: &str,
        lease_seconds: u32,
    ) -> AppResult<Option<ClaimedWorkflowScheduleTick>> {
        self.claim_schedule_tick_impl(
            tenant_id,
            schedule_key,
            slot_key,
            scheduled_for,
            worker_id,
            lease_seconds,
        )
        .await
    }

    async fn complete_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        self.complete_schedule_tick_impl(tenant_id, schedule_key, slot_key, worker_id, lease_token)
            .await
    }

    async fn release_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        self.release_schedule_tick_impl(
            tenant_id,
            schedule_key,
            slot_key,
            worker_id,
            lease_token,
            error_message,
        )
        .await
    }

    async fn claim_jobs(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        partition: Option<WorkflowClaimPartition>,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedWorkflowJob>> {
        self.claim_jobs_impl(worker_id, limit, lease_seconds, partition, tenant_filter)
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

    async fn find_run(&self, tenant_id: TenantId, run_id: &str) -> AppResult<Option<WorkflowRun>> {
        self.find_run_impl(tenant_id, run_id).await
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
    let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
        logical_name: row.logical_name,
        display_name: row.display_name,
        description: row.description,
        trigger: workflow_trigger_from_parts(
            row.trigger_type.as_str(),
            row.trigger_entity_logical_name.as_deref(),
        )?,
        steps: workflow_steps_from_json(row.steps)?,
        max_attempts: u16::try_from(row.max_attempts).map_err(|error| {
            AppError::Validation(format!("invalid workflow max_attempts value: {error}"))
        })?,
    })?;

    workflow.with_publish_state(
        WorkflowLifecycleState::parse(row.lifecycle_state.as_str())?,
        row.current_published_version,
    )
}

fn workflow_steps_to_json(steps: &[WorkflowStep]) -> AppResult<Value> {
    serde_json::to_value(steps).map_err(|error| {
        AppError::Validation(format!("failed to serialize workflow steps: {error}"))
    })
}

fn workflow_steps_from_json(value: Value) -> AppResult<Vec<WorkflowStep>> {
    serde_json::from_value(value).map_err(|error| {
        AppError::Validation(format!("failed to deserialize workflow steps: {error}"))
    })
}

fn workflow_step_traces_to_json(step_traces: &[WorkflowRunStepTrace]) -> AppResult<Value> {
    serde_json::to_value(step_traces).map_err(|error| {
        AppError::Validation(format!("failed to serialize workflow step traces: {error}"))
    })
}

fn workflow_step_traces_from_json(value: Value) -> AppResult<Vec<WorkflowRunStepTrace>> {
    serde_json::from_value(value).map_err(|error| {
        AppError::Validation(format!(
            "failed to deserialize workflow step traces: {error}"
        ))
    })
}

fn workflow_trigger_parts(trigger: &WorkflowTrigger) -> (&'static str, Option<&str>) {
    match trigger {
        WorkflowTrigger::Manual => ("manual", None),
        WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name,
        } => ("runtime_record_created", Some(entity_logical_name.as_str())),
        WorkflowTrigger::RuntimeRecordUpdated {
            entity_logical_name,
        } => ("runtime_record_updated", Some(entity_logical_name.as_str())),
        WorkflowTrigger::RuntimeRecordDeleted {
            entity_logical_name,
        } => ("runtime_record_deleted", Some(entity_logical_name.as_str())),
        WorkflowTrigger::ScheduleTick { schedule_key } => {
            ("schedule_tick", Some(schedule_key.as_str()))
        }
        WorkflowTrigger::WebhookReceived { webhook_key } => {
            ("webhook_received", Some(webhook_key.as_str()))
        }
        WorkflowTrigger::FormSubmitted { form_key } => ("form_submitted", Some(form_key.as_str())),
        WorkflowTrigger::InboundEmailReceived { mailbox_key } => {
            ("inbound_email_received", Some(mailbox_key.as_str()))
        }
        WorkflowTrigger::ApprovalEventReceived { approval_key } => {
            ("approval_event_received", Some(approval_key.as_str()))
        }
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
        "runtime_record_updated" => {
            let entity_logical_name = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "runtime_record_updated trigger requires trigger_entity_logical_name"
                        .to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::RuntimeRecordUpdated {
                entity_logical_name: entity_logical_name.to_owned(),
            })
        }
        "runtime_record_deleted" => {
            let entity_logical_name = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "runtime_record_deleted trigger requires trigger_entity_logical_name"
                        .to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::RuntimeRecordDeleted {
                entity_logical_name: entity_logical_name.to_owned(),
            })
        }
        "schedule_tick" => {
            let schedule_key = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "schedule_tick trigger requires trigger_entity_logical_name".to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::ScheduleTick {
                schedule_key: schedule_key.to_owned(),
            })
        }
        "webhook_received" => {
            let webhook_key = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "webhook_received trigger requires trigger_entity_logical_name".to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::WebhookReceived {
                webhook_key: webhook_key.to_owned(),
            })
        }
        "form_submitted" => {
            let form_key = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "form_submitted trigger requires trigger_entity_logical_name".to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::FormSubmitted {
                form_key: form_key.to_owned(),
            })
        }
        "inbound_email_received" => {
            let mailbox_key = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "inbound_email_received trigger requires trigger_entity_logical_name"
                        .to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::InboundEmailReceived {
                mailbox_key: mailbox_key.to_owned(),
            })
        }
        "approval_event_received" => {
            let approval_key = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "approval_event_received trigger requires trigger_entity_logical_name"
                        .to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::ApprovalEventReceived {
                approval_key: approval_key.to_owned(),
            })
        }
        _ => Err(AppError::Validation(format!(
            "unknown workflow trigger_type '{trigger_type}'"
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
        steps: row.steps,
        max_attempts: row.max_attempts,
        lifecycle_state: row.lifecycle_state,
        current_published_version: row.current_published_version,
    })?;

    Ok(ClaimedWorkflowJob {
        job_id: row.job_id.to_string(),
        tenant_id: TenantId::from_uuid(tenant_uuid),
        run_id: row.run_id.to_string(),
        workflow_version: row.workflow_version,
        workflow,
        trigger_payload: row.trigger_payload,
        lease_token: row.lease_token,
    })
}

fn workflow_run_from_row(row: WorkflowRunRow) -> AppResult<WorkflowRun> {
    Ok(WorkflowRun {
        run_id: row.id.to_string(),
        workflow_logical_name: row.workflow_logical_name,
        workflow_version: row.workflow_version,
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
        step_traces: workflow_step_traces_from_json(row.step_traces)?,
    })
}
