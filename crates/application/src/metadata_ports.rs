use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{
    AuditAction, EntityDefinition, EntityFieldDefinition, FieldType, PublishedEntitySchema,
    RegistrationMode, RuntimeRecord,
};
use serde_json::Value;

/// Logical composition mode for runtime query conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRecordLogicalMode {
    /// Every condition must match.
    And,
    /// Any condition may match.
    Or,
}

impl RuntimeRecordLogicalMode {
    /// Parses transport value into a logical mode.
    pub fn parse_transport(value: &str) -> AppResult<Self> {
        match value {
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            _ => Err(qryvanta_core::AppError::Validation(format!(
                "unknown runtime query logical mode '{value}'"
            ))),
        }
    }

    /// Returns the stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

/// Runtime query comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRecordOperator {
    /// JSON equality.
    Eq,
    /// JSON inequality.
    Neq,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Gte,
    /// Less than.
    Lt,
    /// Less than or equal.
    Lte,
    /// String contains comparison.
    Contains,
    /// Membership in a set of values.
    In,
}

impl RuntimeRecordOperator {
    /// Parses transport value into an operator.
    pub fn parse_transport(value: &str) -> AppResult<Self> {
        match value {
            "eq" => Ok(Self::Eq),
            "neq" => Ok(Self::Neq),
            "gt" => Ok(Self::Gt),
            "gte" => Ok(Self::Gte),
            "lt" => Ok(Self::Lt),
            "lte" => Ok(Self::Lte),
            "contains" => Ok(Self::Contains),
            "in" => Ok(Self::In),
            _ => Err(qryvanta_core::AppError::Validation(format!(
                "unknown runtime query operator '{value}'"
            ))),
        }
    }

    /// Returns the stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Neq => "neq",
            Self::Gt => "gt",
            Self::Gte => "gte",
            Self::Lt => "lt",
            Self::Lte => "lte",
            Self::Contains => "contains",
            Self::In => "in",
        }
    }
}

/// Runtime query sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRecordSortDirection {
    /// Ascending sort direction.
    Asc,
    /// Descending sort direction.
    Desc,
}

impl RuntimeRecordSortDirection {
    /// Parses transport value into sort direction.
    pub fn parse_transport(value: &str) -> AppResult<Self> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(qryvanta_core::AppError::Validation(format!(
                "unknown runtime query sort direction '{value}'"
            ))),
        }
    }

    /// Returns the stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Uniqueness index entry persisted alongside runtime records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueFieldValue {
    /// Field logical name.
    pub field_logical_name: String,
    /// Stable hash for the field value.
    pub field_value_hash: String,
}

/// Query inputs for runtime record listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordListQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
    /// Optional subject ownership filter.
    pub owner_subject: Option<String>,
}

/// Typed condition for runtime record queries.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordFilter {
    /// Field logical name to compare.
    pub field_logical_name: String,
    /// Comparison operator.
    pub operator: RuntimeRecordOperator,
    /// Field type from the published schema.
    pub field_type: FieldType,
    /// Expected field value.
    pub field_value: Value,
}

/// Sort instruction for runtime record queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRecordSort {
    /// Field logical name to sort by.
    pub field_logical_name: String,
    /// Field type from the published schema.
    pub field_type: FieldType,
    /// Sort direction.
    pub direction: RuntimeRecordSortDirection,
}

/// Query inputs for runtime record listing with exact-match filters.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecordQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
    /// Logical composition mode for conditions.
    pub logical_mode: RuntimeRecordLogicalMode,
    /// Typed query conditions.
    pub filters: Vec<RuntimeRecordFilter>,
    /// Sort instructions.
    pub sort: Vec<RuntimeRecordSort>,
    /// Optional subject ownership filter.
    pub owner_subject: Option<String>,
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
