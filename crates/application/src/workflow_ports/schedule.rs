use chrono::{DateTime, Utc};
use qryvanta_core::TenantId;

/// One tenant-scoped enabled schedule trigger source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowScheduledTrigger {
    /// Tenant owning the workflow schedule.
    pub tenant_id: TenantId,
    /// Stable schedule key from the workflow trigger.
    pub schedule_key: String,
}

/// One claimed persisted schedule tick slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedWorkflowScheduleTick {
    /// Tenant owning the schedule tick.
    pub tenant_id: TenantId,
    /// Stable schedule key from the workflow trigger.
    pub schedule_key: String,
    /// Deterministic slot key for the scheduled occurrence.
    pub slot_key: String,
    /// Effective UTC timestamp for the scheduled occurrence.
    pub scheduled_for: DateTime<Utc>,
    /// Worker identifier currently leasing the slot.
    pub worker_id: String,
    /// Lease token for completion or release.
    pub lease_token: String,
}

/// Scheduler drain result for one worker cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorkflowScheduleTickDrainResult {
    /// Number of due schedule slots claimed in this cycle.
    pub claimed_ticks: usize,
    /// Number of workflow dispatches created from claimed ticks.
    pub dispatched_workflows: usize,
    /// Number of claimed ticks released back to pending.
    pub released_ticks: usize,
}
