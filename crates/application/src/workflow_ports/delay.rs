use async_trait::async_trait;
use qryvanta_core::AppResult;

/// Delay gateway for native workflow pause steps.
#[async_trait]
pub trait WorkflowDelayService: Send + Sync {
    /// Sleeps for the requested duration in milliseconds.
    async fn sleep(&self, duration_ms: u64) -> AppResult<()>;
}
