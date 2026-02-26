use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{
    BusinessRuleDefinition, EntityDefinition, EntityFieldDefinition, FormDefinition,
    OptionSetDefinition, PublishedEntitySchema, RuntimeRecord, ViewDefinition,
};
use serde_json::Value;

use super::{RecordListQuery, RuntimeRecordQuery, UniqueFieldValue};

/// Legacy aggregate repository port for metadata and runtime persistence.
#[async_trait]
pub trait MetadataRepository: Send + Sync {
    /// Saves an entity definition.
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()>;

    /// Lists all entity definitions.
    async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>>;

    /// Looks up a single entity definition by logical name.
    async fn find_entity(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<EntityDefinition>>;

    /// Updates an existing entity definition.
    async fn update_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()>;

    /// Saves or updates an entity field definition.
    async fn save_field(&self, tenant_id: TenantId, field: EntityFieldDefinition) -> AppResult<()>;

    /// Lists field definitions for an entity.
    async fn list_fields(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>>;

    /// Looks up a single field definition by logical name.
    async fn find_field(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<Option<EntityFieldDefinition>>;

    /// Deletes a field definition by logical name.
    async fn delete_field(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<()>;

    /// Returns whether the field exists in any published schema version.
    async fn field_exists_in_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<bool>;

    /// Saves or updates an option set definition.
    async fn save_option_set(
        &self,
        tenant_id: TenantId,
        option_set: OptionSetDefinition,
    ) -> AppResult<()>;

    /// Lists option sets for an entity.
    async fn list_option_sets(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<OptionSetDefinition>>;

    /// Finds a single option set by logical name.
    async fn find_option_set(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<Option<OptionSetDefinition>>;

    /// Deletes an option set by logical name.
    async fn delete_option_set(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<()>;

    /// Saves or updates a standalone form definition.
    async fn save_form(&self, tenant_id: TenantId, form: FormDefinition) -> AppResult<()>;

    /// Lists standalone forms for an entity.
    async fn list_forms(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>>;

    /// Finds a standalone form by logical name.
    async fn find_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>>;

    /// Deletes a standalone form by logical name.
    async fn delete_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<()>;

    /// Saves or updates a standalone view definition.
    async fn save_view(&self, tenant_id: TenantId, view: ViewDefinition) -> AppResult<()>;

    /// Lists standalone views for an entity.
    async fn list_views(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>>;

    /// Finds a standalone view by logical name.
    async fn find_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>>;

    /// Deletes a standalone view by logical name.
    async fn delete_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<()>;

    /// Saves or updates a business rule definition.
    async fn save_business_rule(
        &self,
        tenant_id: TenantId,
        business_rule: BusinessRuleDefinition,
    ) -> AppResult<()>;

    /// Lists business rules for an entity.
    async fn list_business_rules(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<BusinessRuleDefinition>>;

    /// Finds one business rule by logical name.
    async fn find_business_rule(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<Option<BusinessRuleDefinition>>;

    /// Deletes a business rule by logical name.
    async fn delete_business_rule(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<()>;

    /// Publishes an immutable entity schema snapshot and returns the published version.
    async fn publish_entity_schema(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
        option_sets: Vec<OptionSetDefinition>,
        published_by: &str,
    ) -> AppResult<PublishedEntitySchema>;

    /// Returns the latest published schema for an entity.
    async fn latest_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>>;

    /// Persists published form snapshots for an entity/schema version.
    async fn save_published_form_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        forms: &[FormDefinition],
    ) -> AppResult<()>;

    /// Persists published view snapshots for an entity/schema version.
    async fn save_published_view_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        views: &[ViewDefinition],
    ) -> AppResult<()>;

    /// Returns latest published form snapshots for an entity.
    async fn list_latest_published_form_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>>;

    /// Returns latest published view snapshots for an entity.
    async fn list_latest_published_view_snapshots(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>>;

    /// Creates a runtime record and attaches unique field index entries.
    async fn create_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        created_by_subject: &str,
    ) -> AppResult<RuntimeRecord>;

    /// Updates a runtime record and replaces unique field index entries.
    async fn update_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord>;

    /// Lists runtime records for an entity.
    async fn list_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>>;

    /// Queries runtime records for an entity using exact-match field filters.
    async fn query_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>>;

    /// Finds a runtime record by identifier.
    async fn find_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<Option<RuntimeRecord>>;

    /// Deletes a runtime record by identifier.
    async fn delete_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()>;

    /// Checks whether a runtime record exists in the provided entity scope.
    async fn runtime_record_exists(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<bool>;

    /// Returns whether a runtime record belongs to the provided subject.
    async fn runtime_record_owned_by_subject(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool>;

    /// Returns whether any relation field currently references a runtime record.
    async fn has_relation_reference(
        &self,
        tenant_id: TenantId,
        target_entity_logical_name: &str,
        target_record_id: &str,
    ) -> AppResult<bool>;
}

/// Focused metadata definition repository trait.
#[async_trait]
pub trait MetadataDefinitionsRepository: MetadataRepository {}

impl<T: MetadataRepository + ?Sized> MetadataDefinitionsRepository for T {}

/// Focused metadata component repository trait.
#[async_trait]
pub trait MetadataComponentsRepository: MetadataRepository {}

impl<T: MetadataRepository + ?Sized> MetadataComponentsRepository for T {}

/// Focused metadata publish repository trait.
#[async_trait]
pub trait MetadataPublishRepository: MetadataRepository {}

impl<T: MetadataRepository + ?Sized> MetadataPublishRepository for T {}

/// Focused runtime record repository trait.
#[async_trait]
pub trait MetadataRuntimeRepository: MetadataRepository {}

impl<T: MetadataRepository + ?Sized> MetadataRuntimeRepository for T {}

/// Composed metadata repository interface grouped by repository concerns.
pub trait MetadataRepositoryByConcern:
    MetadataDefinitionsRepository
    + MetadataComponentsRepository
    + MetadataPublishRepository
    + MetadataRuntimeRepository
    + Send
    + Sync
{
}

impl<T> MetadataRepositoryByConcern for T where
    T: MetadataDefinitionsRepository
        + MetadataComponentsRepository
        + MetadataPublishRepository
        + MetadataRuntimeRepository
        + Send
        + Sync
{
}
