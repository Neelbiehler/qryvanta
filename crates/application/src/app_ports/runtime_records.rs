use async_trait::async_trait;

use qryvanta_core::{AppResult, UserIdentity};
use qryvanta_domain::{FormDefinition, PublishedEntitySchema, RuntimeRecord, ViewDefinition};
use serde_json::Value;

use crate::metadata_ports::{RecordListQuery, RuntimeRecordQuery};

/// Runtime record gateway used by app-scoped execution.
#[async_trait]
pub trait RuntimeRecordService: Send + Sync {
    /// Returns latest published schema for an entity.
    async fn latest_published_schema_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>>;

    /// Lists runtime records without global permission checks.
    async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>>;

    /// Queries runtime records without global permission checks.
    async fn query_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>>;

    /// Fetches one runtime record without global permission checks.
    async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord>;

    /// Creates runtime record without global permission checks.
    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Updates runtime record without global permission checks.
    async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Deletes runtime record without global permission checks.
    async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()>;

    /// Lists standalone forms without global permission checks.
    async fn list_forms_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>>;

    /// Finds a standalone form without global permission checks.
    async fn find_form_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>>;

    /// Lists standalone views without global permission checks.
    async fn list_views_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>>;

    /// Finds a standalone view without global permission checks.
    async fn find_view_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>>;
}
