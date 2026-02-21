use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AuditAction, EntityDefinition, EntityFieldDefinition, FieldType, Permission,
    PublishedEntitySchema, RegistrationMode, RuntimeRecord,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::AuthorizationService;

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
}

/// Application service for metadata and runtime record operations.
#[derive(Clone)]
pub struct MetadataService {
    repository: Arc<dyn MetadataRepository>,
    authorization_service: AuthorizationService,
    audit_repository: Arc<dyn AuditRepository>,
}

impl MetadataService {
    /// Creates a new metadata service from a repository implementation.
    #[must_use]
    pub fn new(
        repository: Arc<dyn MetadataRepository>,
        authorization_service: AuthorizationService,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            repository,
            authorization_service,
            audit_repository,
        }
    }

    /// Registers a new entity definition.
    pub async fn register_entity(
        &self,
        actor: &UserIdentity,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> AppResult<EntityDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataEntityCreate,
            )
            .await?;

        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        let entity = EntityDefinition::new(logical_name, display_name)?;
        self.repository
            .save_entity(actor.tenant_id(), entity.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataEntityCreated,
                resource_type: "entity_definition".to_owned(),
                resource_id: entity.logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "created metadata entity '{}'",
                    entity.logical_name().as_str()
                )),
            })
            .await?;

        Ok(entity)
    }

    /// Returns every known entity definition.
    pub async fn list_entities(&self, actor: &UserIdentity) -> AppResult<Vec<EntityDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataEntityRead,
            )
            .await?;

        self.repository.list_entities(actor.tenant_id()).await
    }

    /// Saves or updates a metadata field definition for an entity.
    pub async fn save_field(
        &self,
        actor: &UserIdentity,
        input: SaveFieldInput,
    ) -> AppResult<EntityFieldDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        self.require_entity_exists(actor.tenant_id(), input.entity_logical_name.as_str())
            .await?;

        if let Some(target_entity) = input.relation_target_entity.as_deref() {
            self.require_entity_exists(actor.tenant_id(), target_entity)
                .await?;
        }

        let field = EntityFieldDefinition::new(
            input.entity_logical_name,
            input.logical_name,
            input.display_name,
            input.field_type,
            input.is_required,
            input.is_unique,
            input.default_value,
            input.relation_target_entity,
        )?;

        self.repository
            .save_field(actor.tenant_id(), field.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_field_definition".to_owned(),
                resource_id: format!(
                    "{}.{}",
                    field.entity_logical_name().as_str(),
                    field.logical_name().as_str()
                ),
                detail: Some(format!(
                    "saved metadata field '{}' on entity '{}'",
                    field.logical_name().as_str(),
                    field.entity_logical_name().as_str()
                )),
            })
            .await?;

        Ok(field)
    }

    /// Lists metadata fields for an entity.
    pub async fn list_fields(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;

        self.repository
            .list_fields(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Publishes draft metadata for an entity as an immutable versioned schema.
    pub async fn publish_entity(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<PublishedEntitySchema> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataEntityCreate,
            )
            .await?;

        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        let entity = self
            .repository
            .find_entity(actor.tenant_id(), entity_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "entity '{}' does not exist for tenant '{}'",
                    entity_logical_name,
                    actor.tenant_id()
                ))
            })?;

        let fields = self
            .repository
            .list_fields(actor.tenant_id(), entity_logical_name)
            .await?;

        if fields.is_empty() {
            return Err(AppError::Validation(format!(
                "entity '{}' requires at least one field before publishing",
                entity_logical_name
            )));
        }

        let published_schema = self
            .repository
            .publish_entity_schema(actor.tenant_id(), entity, fields, actor.subject())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataEntityPublished,
                resource_type: "entity_definition".to_owned(),
                resource_id: published_schema.entity().logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "published metadata entity '{}' at version {}",
                    published_schema.entity().logical_name().as_str(),
                    published_schema.version()
                )),
            })
            .await?;

        Ok(published_schema)
    }

    /// Returns the latest published metadata schema for an entity.
    pub async fn latest_published_schema(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataEntityRead,
            )
            .await?;

        self.repository
            .latest_published_schema(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Returns the latest published metadata schema without permission checks.
    pub async fn latest_published_schema_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        self.repository
            .latest_published_schema(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Creates a runtime record using the latest published entity schema.
    pub async fn create_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWrite,
            )
            .await?;

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let normalized_data = Self::normalize_record_payload(&schema, data)?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .create_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                normalized_data,
                unique_values,
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordCreated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "created runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Ok(record)
    }

    /// Creates a runtime record without global permission checks.
    pub async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let normalized_data = Self::normalize_record_payload(&schema, data)?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .create_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                normalized_data,
                unique_values,
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordCreated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "created runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Ok(record)
    }

    /// Updates a runtime record using the latest published entity schema.
    pub async fn update_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWrite,
            )
            .await?;

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let normalized_data = Self::normalize_record_payload(&schema, data)?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .update_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                normalized_data,
                unique_values,
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordUpdated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "updated runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Ok(record)
    }

    /// Updates a runtime record without global permission checks.
    pub async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let normalized_data = Self::normalize_record_payload(&schema, data)?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .update_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                normalized_data,
                unique_values,
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordUpdated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "updated runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Ok(record)
    }

    /// Lists runtime records for an entity.
    pub async fn list_runtime_records(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordRead,
            )
            .await?;

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .list_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await
    }

    /// Queries runtime records with exact-match field filters.
    pub async fn query_runtime_records(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordRead,
            )
            .await?;

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let schema_fields: BTreeMap<&str, &EntityFieldDefinition> = schema
            .fields()
            .iter()
            .map(|field| (field.logical_name().as_str(), field))
            .collect();
        let mut seen_filter_fields = BTreeSet::new();
        for filter in &query.filters {
            if !seen_filter_fields.insert(filter.field_logical_name.as_str()) {
                return Err(AppError::Validation(format!(
                    "duplicate runtime query filter field '{}'",
                    filter.field_logical_name
                )));
            }

            let Some(field) = schema_fields.get(filter.field_logical_name.as_str()) else {
                return Err(AppError::Validation(format!(
                    "unknown filter field '{}' for entity '{}'",
                    filter.field_logical_name, entity_logical_name
                )));
            };

            field.validate_runtime_value(&filter.field_value)?;
        }

        self.repository
            .query_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await
    }

    /// Lists runtime records without global permission checks.
    pub async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .list_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await
    }

    /// Gets a runtime record by identifier.
    pub async fn get_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordRead,
            )
            .await?;

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })
    }

    /// Gets a runtime record without global permission checks.
    pub async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })
    }

    /// Deletes a runtime record after enforcing relation-reference safeguards.
    pub async fn delete_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWrite,
            )
            .await?;

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        if self
            .repository
            .has_relation_reference(actor.tenant_id(), entity_logical_name, record_id)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "runtime record '{}' in entity '{}' cannot be deleted because it is still referenced by relation fields",
                record_id, entity_logical_name
            )));
        }

        self.repository
            .delete_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordDeleted,
                resource_type: "runtime_record".to_owned(),
                resource_id: record_id.to_owned(),
                detail: Some(format!(
                    "deleted runtime record '{}' for entity '{}'",
                    record_id, entity_logical_name
                )),
            })
            .await?;

        Ok(())
    }

    /// Deletes a runtime record without global permission checks.
    pub async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        if self
            .repository
            .has_relation_reference(actor.tenant_id(), entity_logical_name, record_id)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "runtime record '{}' in entity '{}' cannot be deleted because it is still referenced by relation fields",
                record_id, entity_logical_name
            )));
        }

        self.repository
            .delete_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordDeleted,
                resource_type: "runtime_record".to_owned(),
                resource_id: record_id.to_owned(),
                detail: Some(format!(
                    "deleted runtime record '{}' for entity '{}'",
                    record_id, entity_logical_name
                )),
            })
            .await?;

        Ok(())
    }

    async fn require_entity_exists(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<()> {
        let entity = self
            .repository
            .find_entity(tenant_id, entity_logical_name)
            .await?;

        if entity.is_none() {
            return Err(AppError::NotFound(format!(
                "entity '{}' does not exist for tenant '{}'",
                entity_logical_name, tenant_id
            )));
        }

        Ok(())
    }

    async fn published_schema_for_runtime(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<PublishedEntitySchema> {
        self.repository
            .latest_published_schema(tenant_id, entity_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "entity '{}' must be published before runtime records can be used",
                    entity_logical_name
                ))
            })
    }

    fn normalize_record_payload(schema: &PublishedEntitySchema, data: Value) -> AppResult<Value> {
        let mut object = match data {
            Value::Object(object) => object,
            _ => {
                return Err(AppError::Validation(
                    "runtime record payload must be a JSON object".to_owned(),
                ));
            }
        };

        let allowed_fields: BTreeSet<&str> = schema
            .fields()
            .iter()
            .map(|field| field.logical_name().as_str())
            .collect();
        for key in object.keys() {
            if !allowed_fields.contains(key.as_str()) {
                return Err(AppError::Validation(format!(
                    "unknown field '{}' for entity '{}'",
                    key,
                    schema.entity().logical_name().as_str()
                )));
            }
        }

        for field in schema.fields() {
            let field_name = field.logical_name().as_str();
            if let Some(value) = object.get(field_name) {
                field.validate_runtime_value(value)?;
                continue;
            }

            if let Some(default_value) = field.default_value() {
                object.insert(field_name.to_owned(), default_value.clone());
                continue;
            }

            if field.is_required() {
                return Err(AppError::Validation(format!(
                    "missing required field '{}'",
                    field_name
                )));
            }
        }

        Ok(Value::Object(object))
    }

    fn unique_values_for_record(
        schema: &PublishedEntitySchema,
        data: &Value,
    ) -> AppResult<Vec<UniqueFieldValue>> {
        let object = data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;
        let mut values = Vec::new();

        for field in schema.fields() {
            if !field.is_unique() {
                continue;
            }

            let Some(value) = object.get(field.logical_name().as_str()) else {
                continue;
            };

            values.push(UniqueFieldValue {
                field_logical_name: field.logical_name().as_str().to_owned(),
                field_value_hash: Self::hash_json_value(value)?,
            });
        }

        values.sort_by(|left, right| {
            left.field_logical_name
                .as_str()
                .cmp(right.field_logical_name.as_str())
        });

        Ok(values)
    }

    fn hash_json_value(value: &Value) -> AppResult<String> {
        let encoded = serde_json::to_vec(value).map_err(|error| {
            AppError::Internal(format!(
                "failed to encode unique field value hash input: {error}"
            ))
        })?;

        let digest = Sha256::digest(encoded);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    async fn validate_relation_values(
        &self,
        schema: &PublishedEntitySchema,
        tenant_id: TenantId,
        data: &Value,
    ) -> AppResult<()> {
        let object = data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;

        for field in schema.fields() {
            if field.field_type() != FieldType::Relation {
                continue;
            }

            let Some(relation_target) = field.relation_target_entity() else {
                continue;
            };
            let Some(value) = object.get(field.logical_name().as_str()) else {
                continue;
            };
            let Some(record_id) = value.as_str() else {
                continue;
            };

            let exists = self
                .repository
                .runtime_record_exists(tenant_id, relation_target.as_str(), record_id)
                .await?;

            if !exists {
                return Err(AppError::Validation(format!(
                    "relation field '{}' references missing record '{}' in entity '{}'",
                    field.logical_name().as_str(),
                    record_id,
                    relation_target.as_str()
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
    use qryvanta_domain::{
        AuditAction, EntityDefinition, EntityFieldDefinition, FieldType, Permission,
        PublishedEntitySchema, RuntimeRecord,
    };
    use serde_json::{Value, json};
    use tokio::sync::Mutex;
    use uuid::Uuid;

    use crate::{AuthorizationRepository, AuthorizationService};

    use super::{
        AuditEvent, AuditRepository, MetadataRepository, MetadataService, RecordListQuery,
        RuntimeRecordFilter, RuntimeRecordQuery, SaveFieldInput, UniqueFieldValue,
    };

    struct FakeRepository {
        entities: Mutex<HashMap<(TenantId, String), EntityDefinition>>,
        fields: Mutex<HashMap<(TenantId, String, String), EntityFieldDefinition>>,
        published_schemas: Mutex<HashMap<(TenantId, String), Vec<PublishedEntitySchema>>>,
        runtime_records: Mutex<HashMap<(TenantId, String, String), RuntimeRecord>>,
        unique_values: Mutex<HashMap<(TenantId, String, String, String), String>>,
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                entities: Mutex::new(HashMap::new()),
                fields: Mutex::new(HashMap::new()),
                published_schemas: Mutex::new(HashMap::new()),
                runtime_records: Mutex::new(HashMap::new()),
                unique_values: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl MetadataRepository for FakeRepository {
        async fn save_entity(
            &self,
            tenant_id: TenantId,
            entity: EntityDefinition,
        ) -> AppResult<()> {
            let key = (tenant_id, entity.logical_name().as_str().to_owned());
            let mut entities = self.entities.lock().await;

            if entities.contains_key(&key) {
                return Err(AppError::Conflict(format!(
                    "entity '{}' already exists for tenant '{}'",
                    key.1, key.0
                )));
            }

            entities.insert(key, entity);
            Ok(())
        }

        async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
            let entities = self.entities.lock().await;
            let mut listed: Vec<EntityDefinition> = entities
                .iter()
                .filter_map(|((stored_tenant_id, _), entity)| {
                    (stored_tenant_id == &tenant_id).then_some(entity.clone())
                })
                .collect();
            listed.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });
            Ok(listed)
        }

        async fn find_entity(
            &self,
            tenant_id: TenantId,
            logical_name: &str,
        ) -> AppResult<Option<EntityDefinition>> {
            Ok(self
                .entities
                .lock()
                .await
                .get(&(tenant_id, logical_name.to_owned()))
                .cloned())
        }

        async fn save_field(
            &self,
            tenant_id: TenantId,
            field: EntityFieldDefinition,
        ) -> AppResult<()> {
            let key = (
                tenant_id,
                field.entity_logical_name().as_str().to_owned(),
                field.logical_name().as_str().to_owned(),
            );
            self.fields.lock().await.insert(key, field);
            Ok(())
        }

        async fn list_fields(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
        ) -> AppResult<Vec<EntityFieldDefinition>> {
            let fields = self.fields.lock().await;
            let mut listed: Vec<EntityFieldDefinition> = fields
                .iter()
                .filter_map(|((stored_tenant_id, stored_entity, _), field)| {
                    (stored_tenant_id == &tenant_id && stored_entity == entity_logical_name)
                        .then_some(field.clone())
                })
                .collect();

            listed.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });

            Ok(listed)
        }

        async fn publish_entity_schema(
            &self,
            tenant_id: TenantId,
            entity: EntityDefinition,
            fields: Vec<EntityFieldDefinition>,
            _published_by: &str,
        ) -> AppResult<PublishedEntitySchema> {
            let key = (tenant_id, entity.logical_name().as_str().to_owned());
            let mut published = self.published_schemas.lock().await;
            let existing = published.entry(key).or_default();
            let version = existing
                .last()
                .map(|schema| schema.version() + 1)
                .unwrap_or(1);
            let schema = PublishedEntitySchema::new(entity, version, fields)?;
            existing.push(schema.clone());
            Ok(schema)
        }

        async fn latest_published_schema(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
        ) -> AppResult<Option<PublishedEntitySchema>> {
            Ok(self
                .published_schemas
                .lock()
                .await
                .get(&(tenant_id, entity_logical_name.to_owned()))
                .and_then(|versions| versions.last().cloned()))
        }

        async fn create_runtime_record(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
            data: Value,
            unique_values: Vec<UniqueFieldValue>,
        ) -> AppResult<RuntimeRecord> {
            let record_id = Uuid::new_v4().to_string();
            let record = RuntimeRecord::new(record_id, entity_logical_name, data)?;

            let mut unique_index = self.unique_values.lock().await;
            for unique_value in &unique_values {
                let key = (
                    tenant_id,
                    entity_logical_name.to_owned(),
                    unique_value.field_logical_name.clone(),
                    unique_value.field_value_hash.clone(),
                );

                if unique_index.contains_key(&key) {
                    return Err(AppError::Conflict(format!(
                        "unique constraint violated for field '{}'",
                        unique_value.field_logical_name
                    )));
                }
            }

            for unique_value in unique_values {
                unique_index.insert(
                    (
                        tenant_id,
                        entity_logical_name.to_owned(),
                        unique_value.field_logical_name,
                        unique_value.field_value_hash,
                    ),
                    record.record_id().as_str().to_owned(),
                );
            }

            self.runtime_records.lock().await.insert(
                (
                    tenant_id,
                    entity_logical_name.to_owned(),
                    record.record_id().as_str().to_owned(),
                ),
                record.clone(),
            );

            Ok(record)
        }

        async fn update_runtime_record(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
            record_id: &str,
            data: Value,
            unique_values: Vec<UniqueFieldValue>,
        ) -> AppResult<RuntimeRecord> {
            let record_key = (
                tenant_id,
                entity_logical_name.to_owned(),
                record_id.to_owned(),
            );
            if !self.runtime_records.lock().await.contains_key(&record_key) {
                return Err(AppError::NotFound(format!(
                    "runtime record '{}' does not exist",
                    record_id
                )));
            }

            let mut unique_index = self.unique_values.lock().await;
            unique_index.retain(|(_, entity, _, _), existing_record_id| {
                !(entity == entity_logical_name && existing_record_id == record_id)
            });

            for unique_value in &unique_values {
                let key = (
                    tenant_id,
                    entity_logical_name.to_owned(),
                    unique_value.field_logical_name.clone(),
                    unique_value.field_value_hash.clone(),
                );

                if unique_index
                    .get(&key)
                    .map(|existing_record_id| existing_record_id.as_str() != record_id)
                    .unwrap_or(false)
                {
                    return Err(AppError::Conflict(format!(
                        "unique constraint violated for field '{}'",
                        unique_value.field_logical_name
                    )));
                }
            }

            for unique_value in unique_values {
                unique_index.insert(
                    (
                        tenant_id,
                        entity_logical_name.to_owned(),
                        unique_value.field_logical_name,
                        unique_value.field_value_hash,
                    ),
                    record_id.to_owned(),
                );
            }

            let updated = RuntimeRecord::new(record_id, entity_logical_name, data)?;
            self.runtime_records
                .lock()
                .await
                .insert(record_key, updated.clone());

            Ok(updated)
        }

        async fn list_runtime_records(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
            query: RecordListQuery,
        ) -> AppResult<Vec<RuntimeRecord>> {
            let records = self.runtime_records.lock().await;
            let mut listed: Vec<RuntimeRecord> = records
                .iter()
                .filter_map(|((stored_tenant_id, stored_entity, _), record)| {
                    (stored_tenant_id == &tenant_id && stored_entity == entity_logical_name)
                        .then_some(record.clone())
                })
                .collect();
            listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));

            Ok(listed
                .into_iter()
                .skip(query.offset)
                .take(query.limit)
                .collect())
        }

        async fn query_runtime_records(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
            query: RuntimeRecordQuery,
        ) -> AppResult<Vec<RuntimeRecord>> {
            let records = self.runtime_records.lock().await;
            let mut listed: Vec<RuntimeRecord> = records
                .iter()
                .filter_map(|((stored_tenant_id, stored_entity, _), record)| {
                    (stored_tenant_id == &tenant_id && stored_entity == entity_logical_name)
                        .then_some(record.clone())
                })
                .filter(|record| {
                    query.filters.iter().all(|filter| {
                        record
                            .data()
                            .as_object()
                            .and_then(|data| data.get(filter.field_logical_name.as_str()))
                            .map(|value| value == &filter.field_value)
                            .unwrap_or(false)
                    })
                })
                .collect();

            listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));

            Ok(listed
                .into_iter()
                .skip(query.offset)
                .take(query.limit)
                .collect())
        }

        async fn find_runtime_record(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
            record_id: &str,
        ) -> AppResult<Option<RuntimeRecord>> {
            Ok(self
                .runtime_records
                .lock()
                .await
                .get(&(
                    tenant_id,
                    entity_logical_name.to_owned(),
                    record_id.to_owned(),
                ))
                .cloned())
        }

        async fn delete_runtime_record(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
            record_id: &str,
        ) -> AppResult<()> {
            let removed = self.runtime_records.lock().await.remove(&(
                tenant_id,
                entity_logical_name.to_owned(),
                record_id.to_owned(),
            ));
            if removed.is_none() {
                return Err(AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                )));
            }

            self.unique_values
                .lock()
                .await
                .retain(|(_, entity, _, _), existing_record_id| {
                    !(entity == entity_logical_name && existing_record_id == record_id)
                });

            Ok(())
        }

        async fn runtime_record_exists(
            &self,
            tenant_id: TenantId,
            entity_logical_name: &str,
            record_id: &str,
        ) -> AppResult<bool> {
            Ok(self.runtime_records.lock().await.contains_key(&(
                tenant_id,
                entity_logical_name.to_owned(),
                record_id.to_owned(),
            )))
        }

        async fn has_relation_reference(
            &self,
            tenant_id: TenantId,
            target_entity_logical_name: &str,
            target_record_id: &str,
        ) -> AppResult<bool> {
            let published_schemas = self.published_schemas.lock().await;
            let runtime_records = self.runtime_records.lock().await;

            for ((schema_tenant_id, _), versions) in published_schemas.iter() {
                if schema_tenant_id != &tenant_id {
                    continue;
                }

                let Some(schema) = versions.last() else {
                    continue;
                };

                let relation_fields: Vec<&EntityFieldDefinition> = schema
                    .fields()
                    .iter()
                    .filter(|field| {
                        field.field_type() == FieldType::Relation
                            && field
                                .relation_target_entity()
                                .map(|target| target.as_str() == target_entity_logical_name)
                                .unwrap_or(false)
                    })
                    .collect();

                if relation_fields.is_empty() {
                    continue;
                }

                for ((record_tenant_id, record_entity, _), record) in runtime_records.iter() {
                    if record_tenant_id != &tenant_id
                        || record_entity != schema.entity().logical_name().as_str()
                    {
                        continue;
                    }

                    let Some(data) = record.data().as_object() else {
                        continue;
                    };

                    if relation_fields.iter().any(|field| {
                        data.get(field.logical_name().as_str())
                            .and_then(Value::as_str)
                            .map(|value| value == target_record_id)
                            .unwrap_or(false)
                    }) {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
    }

    #[derive(Default)]
    struct FakeAuditRepository {
        events: Mutex<Vec<AuditEvent>>,
    }

    #[async_trait]
    impl AuditRepository for FakeAuditRepository {
        async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
            self.events.lock().await.push(event);
            Ok(())
        }
    }

    struct FakeAuthorizationRepository {
        grants: HashMap<(TenantId, String), Vec<Permission>>,
    }

    #[async_trait]
    impl AuthorizationRepository for FakeAuthorizationRepository {
        async fn list_permissions_for_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
        ) -> AppResult<Vec<Permission>> {
            Ok(self
                .grants
                .get(&(tenant_id, subject.to_owned()))
                .cloned()
                .unwrap_or_default())
        }
    }

    fn actor(tenant_id: TenantId, subject: &str) -> UserIdentity {
        UserIdentity::new(subject, subject, None, tenant_id)
    }

    fn build_service(
        grants: HashMap<(TenantId, String), Vec<Permission>>,
    ) -> (MetadataService, Arc<FakeAuditRepository>) {
        let authorization_service =
            AuthorizationService::new(Arc::new(FakeAuthorizationRepository { grants }));
        let audit_repository = Arc::new(FakeAuditRepository::default());
        let service = MetadataService::new(
            Arc::new(FakeRepository::new()),
            authorization_service,
            audit_repository.clone(),
        );
        (service, audit_repository)
    }

    #[tokio::test]
    async fn register_entity_persists_data_and_writes_audit_event() {
        let tenant_id = TenantId::new();
        let subject = "alice";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataEntityRead,
                Permission::MetadataFieldWrite,
            ],
        )]);
        let (service, audit_repository) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let created = service.register_entity(&actor, "contact", "Contact").await;
        assert!(created.is_ok());

        let entities = service.list_entities(&actor).await;
        assert!(entities.is_ok());
        assert_eq!(entities.unwrap_or_default().len(), 1);

        let events = audit_repository.events.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, AuditAction::MetadataEntityCreated);
        assert_eq!(events[0].resource_id, "contact");
    }

    #[tokio::test]
    async fn save_field_requires_field_write_permission() {
        let tenant_id = TenantId::new();
        let subject = "bob";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![Permission::MetadataEntityCreate],
        )]);
        let (service, _) = build_service(grants);
        let actor = actor(tenant_id, subject);
        let created = service.register_entity(&actor, "account", "Account").await;
        assert!(created.is_err());
    }

    #[tokio::test]
    async fn publish_entity_requires_fields() {
        let tenant_id = TenantId::new();
        let subject = "carol";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]);
        let (service, _) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let created = service.register_entity(&actor, "contact", "Contact").await;
        assert!(created.is_ok());

        let publish_result = service.publish_entity(&actor, "contact").await;
        assert!(matches!(publish_result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn create_runtime_record_applies_defaults_and_writes_audit_event() {
        let tenant_id = TenantId::new();
        let subject = "dan";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataFieldWrite,
                Permission::RuntimeRecordWrite,
                Permission::RuntimeRecordRead,
            ],
        )]);
        let (service, audit_repository) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let created = service.register_entity(&actor, "contact", "Contact").await;
        assert!(created.is_ok());

        let saved_field = service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "name".to_owned(),
                    display_name: "Name".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: true,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await;
        assert!(saved_field.is_ok());

        let saved_default = service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "active".to_owned(),
                    display_name: "Active".to_owned(),
                    field_type: FieldType::Boolean,
                    is_required: false,
                    is_unique: false,
                    default_value: Some(json!(true)),
                    relation_target_entity: None,
                },
            )
            .await;
        assert!(saved_default.is_ok());

        let published = service.publish_entity(&actor, "contact").await;
        assert!(published.is_ok());

        let created_record = service
            .create_runtime_record(&actor, "contact", json!({"name": "Alice"}))
            .await;
        assert!(created_record.is_ok());
        let created_record = created_record.unwrap_or_else(|_| unreachable!());

        let data = created_record.data().as_object();
        assert!(data.is_some());
        assert_eq!(
            data.and_then(|object| object.get("active")),
            Some(&json!(true))
        );

        let listed = service
            .list_runtime_records(
                &actor,
                "contact",
                RecordListQuery {
                    limit: 20,
                    offset: 0,
                },
            )
            .await;
        assert!(listed.is_ok());
        assert_eq!(listed.unwrap_or_default().len(), 1);

        let events = audit_repository.events.lock().await;
        assert!(events.iter().any(|event| {
            event.action == AuditAction::RuntimeRecordCreated
                && event.resource_id == created_record.record_id().as_str()
        }));
    }

    #[tokio::test]
    async fn query_runtime_records_filters_and_paginates() {
        let tenant_id = TenantId::new();
        let subject = "grace";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataFieldWrite,
                Permission::RuntimeRecordWrite,
                Permission::RuntimeRecordRead,
            ],
        )]);
        let (service, _) = build_service(grants);
        let actor = actor(tenant_id, subject);

        assert!(
            service
                .register_entity(&actor, "contact", "Contact")
                .await
                .is_ok()
        );
        assert!(
            service
                .save_field(
                    &actor,
                    SaveFieldInput {
                        entity_logical_name: "contact".to_owned(),
                        logical_name: "name".to_owned(),
                        display_name: "Name".to_owned(),
                        field_type: FieldType::Text,
                        is_required: true,
                        is_unique: false,
                        default_value: None,
                        relation_target_entity: None,
                    },
                )
                .await
                .is_ok()
        );
        assert!(
            service
                .save_field(
                    &actor,
                    SaveFieldInput {
                        entity_logical_name: "contact".to_owned(),
                        logical_name: "active".to_owned(),
                        display_name: "Active".to_owned(),
                        field_type: FieldType::Boolean,
                        is_required: true,
                        is_unique: false,
                        default_value: None,
                        relation_target_entity: None,
                    },
                )
                .await
                .is_ok()
        );
        assert!(service.publish_entity(&actor, "contact").await.is_ok());

        assert!(
            service
                .create_runtime_record(&actor, "contact", json!({"name": "Alice", "active": true}))
                .await
                .is_ok()
        );
        assert!(
            service
                .create_runtime_record(&actor, "contact", json!({"name": "Bob", "active": false}))
                .await
                .is_ok()
        );
        assert!(
            service
                .create_runtime_record(&actor, "contact", json!({"name": "Carol", "active": true}))
                .await
                .is_ok()
        );

        let queried = service
            .query_runtime_records(
                &actor,
                "contact",
                RuntimeRecordQuery {
                    limit: 1,
                    offset: 1,
                    filters: vec![RuntimeRecordFilter {
                        field_logical_name: "active".to_owned(),
                        field_value: json!(true),
                    }],
                },
            )
            .await;
        assert!(queried.is_ok());

        let queried = queried.unwrap_or_default();
        assert_eq!(queried.len(), 1);
        assert_eq!(
            queried[0]
                .data()
                .as_object()
                .and_then(|value| value.get("active")),
            Some(&json!(true))
        );
    }

    #[tokio::test]
    async fn query_runtime_records_requires_runtime_read_permission() {
        let tenant_id = TenantId::new();
        let subject = "heidi";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataFieldWrite,
                Permission::RuntimeRecordWrite,
            ],
        )]);
        let (service, _) = build_service(grants);
        let actor = actor(tenant_id, subject);

        assert!(
            service
                .register_entity(&actor, "contact", "Contact")
                .await
                .is_ok()
        );
        assert!(
            service
                .save_field(
                    &actor,
                    SaveFieldInput {
                        entity_logical_name: "contact".to_owned(),
                        logical_name: "name".to_owned(),
                        display_name: "Name".to_owned(),
                        field_type: FieldType::Text,
                        is_required: true,
                        is_unique: false,
                        default_value: None,
                        relation_target_entity: None,
                    },
                )
                .await
                .is_ok()
        );
        assert!(service.publish_entity(&actor, "contact").await.is_ok());

        let queried = service
            .query_runtime_records(
                &actor,
                "contact",
                RuntimeRecordQuery {
                    limit: 50,
                    offset: 0,
                    filters: vec![RuntimeRecordFilter {
                        field_logical_name: "name".to_owned(),
                        field_value: json!("Alice"),
                    }],
                },
            )
            .await;

        assert!(matches!(queried, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn delete_runtime_record_blocks_when_relation_exists() {
        let tenant_id = TenantId::new();
        let subject = "eve";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataFieldWrite,
                Permission::RuntimeRecordWrite,
                Permission::RuntimeRecordRead,
            ],
        )]);
        let (service, _) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let created_contact = service.register_entity(&actor, "contact", "Contact").await;
        assert!(created_contact.is_ok());
        let created_deal = service.register_entity(&actor, "deal", "Deal").await;
        assert!(created_deal.is_ok());

        let contact_name_field = service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "name".to_owned(),
                    display_name: "Name".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await;
        assert!(contact_name_field.is_ok());

        let deal_owner_field = service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "deal".to_owned(),
                    logical_name: "owner_contact_id".to_owned(),
                    display_name: "Owner Contact".to_owned(),
                    field_type: FieldType::Relation,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: Some("contact".to_owned()),
                },
            )
            .await;
        assert!(deal_owner_field.is_ok());

        let published_contact = service.publish_entity(&actor, "contact").await;
        assert!(published_contact.is_ok());
        let published_deal = service.publish_entity(&actor, "deal").await;
        assert!(published_deal.is_ok());

        let contact_record = service
            .create_runtime_record(&actor, "contact", json!({"name": "Alice"}))
            .await;
        assert!(contact_record.is_ok());
        let contact_record = contact_record.unwrap_or_else(|_| unreachable!());

        let created_deal_record = service
            .create_runtime_record(
                &actor,
                "deal",
                json!({"owner_contact_id": contact_record.record_id().as_str()}),
            )
            .await;
        assert!(created_deal_record.is_ok());

        let delete_result = service
            .delete_runtime_record(&actor, "contact", contact_record.record_id().as_str())
            .await;
        assert!(matches!(delete_result, Err(AppError::Conflict(_))));
    }

    #[tokio::test]
    async fn get_and_delete_runtime_record_succeed_when_unreferenced() {
        let tenant_id = TenantId::new();
        let subject = "frank";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataFieldWrite,
                Permission::RuntimeRecordWrite,
                Permission::RuntimeRecordRead,
            ],
        )]);
        let (service, audit_repository) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let created_entity = service.register_entity(&actor, "note", "Note").await;
        assert!(created_entity.is_ok());
        let saved_field = service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "note".to_owned(),
                    logical_name: "title".to_owned(),
                    display_name: "Title".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await;
        assert!(saved_field.is_ok());

        let published = service.publish_entity(&actor, "note").await;
        assert!(published.is_ok());

        let created_record = service
            .create_runtime_record(&actor, "note", json!({"title": "A"}))
            .await;
        assert!(created_record.is_ok());
        let created_record = created_record.unwrap_or_else(|_| unreachable!());

        let fetched = service
            .get_runtime_record(&actor, "note", created_record.record_id().as_str())
            .await;
        assert!(fetched.is_ok());

        let deleted = service
            .delete_runtime_record(&actor, "note", created_record.record_id().as_str())
            .await;
        assert!(deleted.is_ok());

        let refetch = service
            .get_runtime_record(&actor, "note", created_record.record_id().as_str())
            .await;
        assert!(matches!(refetch, Err(AppError::NotFound(_))));

        let events = audit_repository.events.lock().await;
        assert!(events.iter().any(|event| {
            event.action == AuditAction::RuntimeRecordDeleted
                && event.resource_id == created_record.record_id().as_str()
        }));
    }
}
