use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{WorkflowDefinition, WorkflowTrigger};

use super::execution::{
    ClaimedWorkflowJob, CompleteWorkflowRunInput, CreateWorkflowRunInput, WorkflowClaimPartition,
    WorkflowQueueStats, WorkflowQueueStatsQuery, WorkflowRun, WorkflowRunAttempt,
    WorkflowRunListQuery, WorkflowWorkerHeartbeatInput,
};

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

    /// Lists enabled workflows matching a trigger shape.
    async fn list_enabled_workflows_for_trigger(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>>;

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
