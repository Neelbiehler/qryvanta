use async_trait::async_trait;
use qryvanta_application::WorkflowDelayService;
use qryvanta_core::AppResult;

/// Tokio-based workflow delay adapter.
pub struct TokioWorkflowDelayService;

#[async_trait]
impl WorkflowDelayService for TokioWorkflowDelayService {
    async fn sleep(&self, duration_ms: u64) -> AppResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;
        Ok(())
    }
}
