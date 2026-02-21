use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{
    AuditAction, EntityDefinition, EntityFieldDefinition, FieldType, PublishedEntitySchema,
    RegistrationMode, RuntimeRecord,
};
use serde_json::Value;

/// Uniqueness index entry persisted alongside runtime records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueFieldValue {
    /// Field logical name.
    pub field_logical_name: String,
    /// Stable hash for the field value.
    pub field_value_hash: String,
}

/// Query inputs for runtime record listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordListQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
}

/// Exact-match filter for runtime record queries.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordFilter {
    /// Field logical name to compare.
    pub field_logical_name: String,
    /// Expected field value (exact JSON equality).
    pub field_value: Value,
}

/// Query inputs for runtime record listing with exact-match filters.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
    /// Exact-match filters combined with logical AND.
    pub filters: Vec<RuntimeRecordFilter>,
}

/// Input payload for metadata field create/update operations.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveFieldInput {
    /// Parent entity logical name.
    pub entity_logical_name: String,
    /// Field logical name.
    pub logical_name: String,
    /// Field display name.
    pub display_name: String,
    /// Field type.
    pub field_type: FieldType,
    /// Required field marker.
    pub is_required: bool,
    /// Unique field marker.
    pub is_unique: bool,
    /// Optional default value.
    pub default_value: Option<Value>,
    /// Optional relation target entity logical name.
    pub relation_target_entity: Option<String>,
}

/// Repository port for metadata and runtime persistence.
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

    /// Saves or updates an entity field definition.
    async fn save_field(&self, tenant_id: TenantId, field: EntityFieldDefinition) -> AppResult<()>;

    /// Lists field definitions for an entity.
    async fn list_fields(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>>;

    /// Publishes an immutable entity schema snapshot and returns the published version.
    async fn publish_entity_schema(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
        published_by: &str,
    ) -> AppResult<PublishedEntitySchema>;

    /// Returns the latest published schema for an entity.
    async fn latest_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>>;

    /// Creates a runtime record and attaches unique field index entries.
    async fn create_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
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

    /// Returns whether any relation field currently references a runtime record.
    async fn has_relation_reference(
        &self,
        tenant_id: TenantId,
        target_entity_logical_name: &str,
        target_record_id: &str,
    ) -> AppResult<bool>;
}

/// Repository port for append-only audit events.
#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Appends a single audit event.
    async fn append_event(&self, event: AuditEvent) -> AppResult<()>;
}

/// Canonical audit event payload emitted by application use-cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    /// Tenant partition key for the event.
    pub tenant_id: TenantId,
    /// Subject that performed the action.
    pub subject: String,
    /// Stable action identifier.
    pub action: AuditAction,
    /// Resource kind targeted by the action.
    pub resource_type: String,
    /// Stable resource identifier.
    pub resource_id: String,
    /// Optional human-readable detail payload.
    pub detail: Option<String>,
}

/// Repository port for subject-to-tenant resolution.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Finds the tenant associated with the provided subject claim.
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>>;

    /// Returns the active registration mode for a tenant.
    async fn registration_mode_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<RegistrationMode>;

    /// Adds a membership for the subject inside a tenant.
    async fn create_membership(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()>;

    /// Ensures the subject can be resolved to a tenant membership and returns that tenant.
    async fn ensure_membership_for_subject(
        &self,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId>;

    /// Returns the runtime contact record mapped to the subject in tenant scope.
    async fn contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Option<String>>;

    /// Saves or replaces the runtime contact record mapping for a tenant subject.
    async fn save_contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        contact_record_id: &str,
    ) -> AppResult<()>;
}
