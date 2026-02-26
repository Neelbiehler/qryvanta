use async_trait::async_trait;
use qryvanta_core::{AppResult, UserIdentity};
use qryvanta_domain::RuntimeRecord;
use serde_json::Value;

/// Runtime record gateway for workflow actions.
#[async_trait]
pub trait WorkflowRuntimeRecordService: Send + Sync {
    /// Creates runtime record without permission checks.
    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;
}
