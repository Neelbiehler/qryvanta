use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId, UserIdentity};
use qryvanta_domain::RuntimeRecord;
use serde_json::Value;

use super::ClaimedRuntimeRecordWorkflowEvent;

/// Runtime record gateway for workflow actions.
#[async_trait]
pub trait WorkflowRuntimeRecordService: Send + Sync {
    /// Returns whether the entity currently has any published schema version.
    async fn has_published_entity_schema(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<bool>;

    /// Creates runtime record without permission checks.
    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Updates runtime record without permission checks.
    async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Deletes runtime record without permission checks.
    async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()>;

    /// Claims one batch of pending runtime-record workflow events.
    async fn claim_runtime_record_workflow_events(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedRuntimeRecordWorkflowEvent>>;

    /// Marks one leased runtime-record workflow event as completed.
    async fn complete_runtime_record_workflow_event(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()>;

    /// Releases one leased runtime-record workflow event back to pending.
    async fn release_runtime_record_workflow_event(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()>;
}
