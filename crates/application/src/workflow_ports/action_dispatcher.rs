use async_trait::async_trait;
use qryvanta_core::AppResult;
use serde_json::Value;

/// External action dispatch type for workflow integration actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowActionDispatchType {
    /// Generic outbound HTTP request action.
    HttpRequest,
    /// Outbound webhook event delivery.
    Webhook,
    /// Outbound email delivery action.
    Email,
}

/// Dispatch payload for integration actions.
#[derive(Debug, Clone)]
pub struct WorkflowActionDispatchRequest {
    /// Dispatch category.
    pub dispatch_type: WorkflowActionDispatchType,
    /// Tenant-scoped workflow run identifier.
    pub run_id: String,
    /// Workflow step path for traceable idempotency.
    pub step_path: String,
    /// Stable idempotency key for retries.
    pub idempotency_key: String,
    /// Payload object from workflow step action data.
    pub payload: Value,
}

/// Port for external integration dispatch operations.
#[async_trait]
pub trait WorkflowActionDispatcher: Send + Sync {
    /// Dispatches one integration action request.
    async fn dispatch_action(&self, request: WorkflowActionDispatchRequest) -> AppResult<()>;
}
