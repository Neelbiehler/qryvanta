use qryvanta_core::TenantId;
use qryvanta_domain::WorkflowTrigger;
use serde_json::Value;

/// Transactional runtime-record workflow event persisted with the record write.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordWorkflowEventInput {
    /// Trigger shape that should be evaluated against enabled workflows.
    pub trigger: WorkflowTrigger,
    /// Record identifier associated with the mutation.
    pub record_id: String,
    /// Fully materialized trigger payload.
    pub payload: Value,
    /// Original subject that emitted the runtime mutation.
    pub emitted_by_subject: String,
}

/// One leased runtime-record workflow event claimed for processing.
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimedRuntimeRecordWorkflowEvent {
    /// Stable event identifier.
    pub event_id: String,
    /// Tenant scope for the event.
    pub tenant_id: TenantId,
    /// Trigger shape that should be dispatched.
    pub trigger: WorkflowTrigger,
    /// Record identifier associated with the mutation.
    pub record_id: String,
    /// Fully materialized trigger payload.
    pub payload: Value,
    /// Original subject that emitted the runtime mutation.
    pub emitted_by_subject: String,
    /// Lease token used for fenced completion and release.
    pub lease_token: String,
}

/// Aggregate result from draining one batch of runtime-record workflow events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeRecordWorkflowEventDrainResult {
    /// Number of outbox events claimed in the batch.
    pub claimed_events: u32,
    /// Number of workflow runs dispatched from the claimed events.
    pub dispatched_workflows: u32,
    /// Number of claimed events released back to pending due to errors.
    pub released_events: u32,
}
