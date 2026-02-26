use chrono::{DateTime, Utc};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{WorkflowAction, WorkflowDefinition, WorkflowStep, WorkflowTrigger};
use serde_json::Value;

/// Workflow execution mode used by application services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowExecutionMode {
    /// Execute workflows inside API request flow.
    Inline,
    /// Queue workflow runs and execute from worker runtimes.
    Queued,
}

/// Workflow creation/update payload.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveWorkflowInput {
    /// Stable workflow logical name.
    pub logical_name: String,
    /// Workflow display name.
    pub display_name: String,
    /// Optional workflow description.
    pub description: Option<String>,
    /// Trigger configuration.
    pub trigger: WorkflowTrigger,
    /// Action configuration.
    pub action: WorkflowAction,
    /// Optional workflow canvas steps.
    pub steps: Option<Vec<WorkflowStep>>,
    /// Max execution attempts before dead-letter.
    pub max_attempts: u16,
    /// Whether workflow is enabled.
    pub is_enabled: bool,
}

/// Workflow run listing query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunListQuery {
    /// Optional workflow logical name filter.
    pub workflow_logical_name: Option<String>,
    /// Page size.
    pub limit: usize,
    /// Row offset.
    pub offset: usize,
}

/// Terminal status for one workflow run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowRunStatus {
    /// Run started and is currently executing.
    Running,
    /// Run finished successfully.
    Succeeded,
    /// Run failed and exhausted retries.
    DeadLettered,
}

impl WorkflowRunStatus {
    /// Returns stable storage value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::DeadLettered => "dead_lettered",
        }
    }

    /// Parses storage value.
    pub fn parse(value: &str) -> AppResult<Self> {
        match value {
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "dead_lettered" => Ok(Self::DeadLettered),
            _ => Err(AppError::Validation(format!(
                "unknown workflow run status '{value}'"
            ))),
        }
    }
}

/// Attempt-level status inside one workflow run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowRunAttemptStatus {
    /// Attempt succeeded.
    Succeeded,
    /// Attempt failed.
    Failed,
}

impl WorkflowRunAttemptStatus {
    /// Returns stable storage value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    /// Parses storage value.
    pub fn parse(value: &str) -> AppResult<Self> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(AppError::Validation(format!(
                "unknown workflow run attempt status '{value}'"
            ))),
        }
    }
}

/// Persisted workflow run record.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowRun {
    /// Stable run identifier.
    pub run_id: String,
    /// Workflow logical name.
    pub workflow_logical_name: String,
    /// Trigger type used for this run.
    pub trigger_type: String,
    /// Optional trigger entity scope.
    pub trigger_entity_logical_name: Option<String>,
    /// Trigger payload captured for observability.
    pub trigger_payload: Value,
    /// Terminal status.
    pub status: WorkflowRunStatus,
    /// Number of attempts that executed.
    pub attempts: i32,
    /// Dead-letter reason when applicable.
    pub dead_letter_reason: Option<String>,
    /// Run start timestamp.
    pub started_at: DateTime<Utc>,
    /// Run finish timestamp when completed.
    pub finished_at: Option<DateTime<Utc>>,
}

/// Persisted workflow run attempt record.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowRunAttempt {
    /// Run identifier.
    pub run_id: String,
    /// 1-based attempt sequence.
    pub attempt_number: i32,
    /// Attempt result status.
    pub status: WorkflowRunAttemptStatus,
    /// Optional failure details.
    pub error_message: Option<String>,
    /// Attempt execution timestamp.
    pub executed_at: DateTime<Utc>,
}

/// Internal run creation payload for repository implementations.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateWorkflowRunInput {
    /// Workflow logical name.
    pub workflow_logical_name: String,
    /// Trigger type.
    pub trigger_type: String,
    /// Optional trigger entity scope.
    pub trigger_entity_logical_name: Option<String>,
    /// Trigger payload.
    pub trigger_payload: Value,
}

/// Internal run completion payload for repository implementations.
#[derive(Debug, Clone, PartialEq)]
pub struct CompleteWorkflowRunInput {
    /// Run identifier.
    pub run_id: String,
    /// Terminal status.
    pub status: WorkflowRunStatus,
    /// Total attempts executed.
    pub attempts: i32,
    /// Optional dead-letter reason.
    pub dead_letter_reason: Option<String>,
}

/// Claimed queued workflow job returned to one worker.
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimedWorkflowJob {
    /// Job identifier.
    pub job_id: String,
    /// Tenant scope for the job.
    pub tenant_id: TenantId,
    /// Associated workflow run identifier.
    pub run_id: String,
    /// Workflow definition snapshot used for execution.
    pub workflow: WorkflowDefinition,
    /// Trigger payload captured when the run was enqueued.
    pub trigger_payload: Value,
    /// Job lease token used for fencing-token completion checks.
    pub lease_token: String,
}

/// Worker heartbeat payload persisted for queue observability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkflowWorkerHeartbeatInput {
    /// Number of jobs claimed in the latest worker cycle.
    pub claimed_jobs: u32,
    /// Number of jobs completed in the latest worker cycle.
    pub executed_jobs: u32,
    /// Number of jobs that failed in the latest worker cycle.
    pub failed_jobs: u32,
    /// Optional tenant-hash partition associated with this worker.
    pub partition: Option<WorkflowClaimPartition>,
}

/// Aggregated queue stats for operations visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkflowQueueStats {
    /// Jobs waiting to be claimed.
    pub pending_jobs: i64,
    /// Jobs currently leased by workers.
    pub leased_jobs: i64,
    /// Jobs completed successfully.
    pub completed_jobs: i64,
    /// Jobs marked failed at queue level.
    pub failed_jobs: i64,
    /// Leased jobs whose lease is expired.
    pub expired_leases: i64,
    /// Workers with a heartbeat in the active window.
    pub active_workers: i64,
}

/// Optional queue partition selector for worker job claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkflowClaimPartition {
    partition_count: u32,
    partition_index: u32,
}

/// Query options for queue stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkflowQueueStatsQuery {
    /// Active worker heartbeat window in seconds.
    pub active_window_seconds: u32,
    /// Optional tenant-hash partition scope.
    pub partition: Option<WorkflowClaimPartition>,
}

/// One distributed worker lease claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowWorkerLease {
    /// Coordination scope key.
    pub scope_key: String,
    /// Lease token used for safe release.
    pub token: String,
    /// Lease holder identity.
    pub holder_id: String,
}

impl WorkflowClaimPartition {
    /// Creates one validated queue partition selector.
    pub fn new(partition_count: u32, partition_index: u32) -> AppResult<Self> {
        if partition_count == 0 {
            return Err(AppError::Validation(
                "partition_count must be greater than zero".to_owned(),
            ));
        }

        if partition_index >= partition_count {
            return Err(AppError::Validation(format!(
                "partition_index must be less than partition_count ({partition_count})"
            )));
        }

        Ok(Self {
            partition_count,
            partition_index,
        })
    }

    /// Returns total number of partitions.
    #[must_use]
    pub fn partition_count(&self) -> u32 {
        self.partition_count
    }

    /// Returns selected partition index.
    #[must_use]
    pub fn partition_index(&self) -> u32 {
        self.partition_index
    }
}
