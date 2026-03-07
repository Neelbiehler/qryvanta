use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{WorkflowDefinition, WorkflowTrigger};

use super::execution::{
    ClaimedWorkflowJob, CompleteWorkflowRunInput, CreateWorkflowRunInput, WorkflowClaimPartition,
    WorkflowQueueStats, WorkflowQueueStatsQuery, WorkflowRun, WorkflowRunAttempt,
    WorkflowRunListQuery, WorkflowWorkerHeartbeatInput,
};
use super::schedule::{ClaimedWorkflowScheduleTick, WorkflowScheduledTrigger};
use chrono::{DateTime, Utc};

/// Repository port for workflow definitions and execution history.
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    /// Saves one workflow definition.
    async fn save_workflow(
        &self,
        tenant_id: TenantId,
        workflow: WorkflowDefinition,
    ) -> AppResult<()>;

    /// Lists workflow definitions for a tenant.
    async fn list_workflows(&self, tenant_id: TenantId) -> AppResult<Vec<WorkflowDefinition>>;

    /// Returns one workflow by logical name.
    async fn find_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>>;

    /// Returns the active published workflow snapshot by logical name.
    async fn find_published_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>>;

    /// Returns one immutable published workflow snapshot by logical name and version.
    async fn find_published_workflow_version(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        version: i32,
    ) -> AppResult<Option<WorkflowDefinition>>;

    /// Publishes the current draft workflow as the next immutable version.
    async fn publish_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        published_by: &str,
    ) -> AppResult<WorkflowDefinition>;

    /// Disables the currently published workflow without changing the draft.
    async fn disable_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<WorkflowDefinition>;

    /// Lists enabled workflows matching a trigger shape.
    async fn list_enabled_workflows_for_trigger(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>>;

    /// Lists distinct enabled schedule trigger sources across tenant scope.
    async fn list_enabled_schedule_triggers(
        &self,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<WorkflowScheduledTrigger>>;

    /// Claims one persisted schedule tick slot when pending or expired.
    async fn claim_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        scheduled_for: DateTime<Utc>,
        worker_id: &str,
        lease_seconds: u32,
    ) -> AppResult<Option<ClaimedWorkflowScheduleTick>>;

    /// Marks one leased schedule tick as completed.
    async fn complete_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()>;

    /// Releases one leased schedule tick back to pending.
    async fn release_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()>;

    /// Creates a new workflow run record in running state.
    async fn create_run(
        &self,
        tenant_id: TenantId,
        input: CreateWorkflowRunInput,
    ) -> AppResult<WorkflowRun>;

    /// Enqueues one workflow run for worker execution.
    async fn enqueue_run_job(&self, tenant_id: TenantId, run_id: &str) -> AppResult<()>;

    /// Claims queued jobs for one worker with a bounded lease.
    async fn claim_jobs(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        partition: Option<WorkflowClaimPartition>,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedWorkflowJob>>;

    /// Marks one leased job as completed.
    async fn complete_job(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()>;

    /// Marks one leased job as failed with an error message.
    async fn fail_job(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()>;

    /// Updates one worker heartbeat snapshot.
    async fn upsert_worker_heartbeat(
        &self,
        worker_id: &str,
        input: WorkflowWorkerHeartbeatInput,
    ) -> AppResult<()>;

    /// Returns aggregate queue and worker heartbeat stats.
    async fn queue_stats(&self, query: WorkflowQueueStatsQuery) -> AppResult<WorkflowQueueStats>;

    /// Appends one attempt row to a workflow run.
    async fn append_run_attempt(
        &self,
        tenant_id: TenantId,
        attempt: WorkflowRunAttempt,
    ) -> AppResult<()>;

    /// Marks a workflow run as completed.
    async fn complete_run(
        &self,
        tenant_id: TenantId,
        input: CompleteWorkflowRunInput,
    ) -> AppResult<WorkflowRun>;

    /// Lists workflow runs by tenant and optional workflow filter.
    async fn list_runs(
        &self,
        tenant_id: TenantId,
        query: WorkflowRunListQuery,
    ) -> AppResult<Vec<WorkflowRun>>;

    /// Returns one workflow run by run id.
    async fn find_run(&self, tenant_id: TenantId, run_id: &str) -> AppResult<Option<WorkflowRun>>;

    /// Lists attempts for one run.
    async fn list_run_attempts(
        &self,
        tenant_id: TenantId,
        run_id: &str,
    ) -> AppResult<Vec<WorkflowRunAttempt>>;
}
