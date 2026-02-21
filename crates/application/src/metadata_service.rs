use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AuditAction, EntityDefinition, EntityFieldDefinition, FieldType, Permission,
    PublishedEntitySchema, RuntimeRecord,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::AuthorizationService;
use crate::metadata_ports::{
    AuditEvent, AuditRepository, MetadataRepository, RecordListQuery, RuntimeRecordFilter,
    RuntimeRecordOperator, RuntimeRecordQuery, RuntimeRecordSort, SaveFieldInput, UniqueFieldValue,
};

/// Application service for metadata and runtime record operations.
#[derive(Clone)]
pub struct MetadataService {
    repository: Arc<dyn MetadataRepository>,
    authorization_service: AuthorizationService,
    audit_repository: Arc<dyn AuditRepository>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeAccessScope {
    All,
    Own,
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
        self.runtime_write_scope_for_actor(actor).await?;

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

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
                actor.subject(),
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

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Creates a runtime record without global permission checks.
    pub async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.runtime_write_scope_for_actor_optional(actor).await?;

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

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
                actor.subject(),
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

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Updates a runtime record using the latest published entity schema.
    pub async fn update_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        let write_scope = self.runtime_write_scope_for_actor(actor).await?;

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only update owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

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

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Updates a runtime record without global permission checks.
    pub async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        let write_scope = self
            .runtime_write_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only update owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

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

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Lists runtime records for an entity.
    pub async fn list_runtime_records(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let read_scope = self.runtime_read_scope_for_actor(actor).await?;
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own {
            query.owner_subject = Some(actor.subject().to_owned());
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let records = self
            .repository
            .list_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await?;

        Self::redact_runtime_records_if_needed(records, field_access.as_ref())
    }

    /// Queries runtime records with exact-match field filters.
    pub async fn query_runtime_records(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let read_scope = self.runtime_read_scope_for_actor(actor).await?;
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own {
            query.owner_subject = Some(actor.subject().to_owned());
        }

        if let Some(access) = &field_access {
            Self::enforce_query_readable_fields(&query, access)?;
        }

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        if query.limit == 0 {
            return Err(AppError::Validation(
                "runtime record query limit must be greater than zero".to_owned(),
            ));
        }

        let schema_fields: BTreeMap<&str, &EntityFieldDefinition> = schema
            .fields()
            .iter()
            .map(|field| (field.logical_name().as_str(), field))
            .collect();

        for filter in &query.filters {
            let Some(field) = schema_fields.get(filter.field_logical_name.as_str()) else {
                return Err(AppError::Validation(format!(
                    "unknown filter field '{}' for entity '{}'",
                    filter.field_logical_name, entity_logical_name
                )));
            };

            Self::validate_runtime_query_filter(field, filter)?;
        }

        let mut seen_sort_fields = BTreeSet::new();
        for sort in &query.sort {
            if !seen_sort_fields.insert(sort.field_logical_name.as_str()) {
                return Err(AppError::Validation(format!(
                    "duplicate runtime query sort field '{}'",
                    sort.field_logical_name
                )));
            }

            let Some(field) = schema_fields.get(sort.field_logical_name.as_str()) else {
                return Err(AppError::Validation(format!(
                    "unknown sort field '{}' for entity '{}'",
                    sort.field_logical_name, entity_logical_name
                )));
            };

            Self::validate_runtime_query_sort(field, sort)?;
        }

        let records = self
            .repository
            .query_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await?;

        Self::redact_runtime_records_if_needed(records, field_access.as_ref())
    }

    /// Lists runtime records without global permission checks.
    pub async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let read_scope = self
            .runtime_read_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own {
            query.owner_subject = Some(actor.subject().to_owned());
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let records = self
            .repository
            .list_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await?;

        Self::redact_runtime_records_if_needed(records, field_access.as_ref())
    }

    /// Gets a runtime record by identifier.
    pub async fn get_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        let read_scope = self.runtime_read_scope_for_actor(actor).await?;
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only read owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Returns whether the runtime record owner subject matches.
    pub async fn runtime_record_owned_by_subject(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool> {
        self.runtime_read_scope_for_actor(actor).await?;

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .runtime_record_owned_by_subject(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                subject,
            )
            .await
    }

    /// Gets a runtime record without global permission checks.
    pub async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        let read_scope = self
            .runtime_read_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only read owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Returns whether the runtime record owner subject matches without global checks.
    pub async fn runtime_record_owned_by_subject_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool> {
        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .runtime_record_owned_by_subject(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                subject,
            )
            .await
    }

    /// Deletes a runtime record after enforcing relation-reference safeguards.
    pub async fn delete_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        let write_scope = self.runtime_write_scope_for_actor(actor).await?;

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only delete owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

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
        let write_scope = self
            .runtime_write_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only delete owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

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

    async fn runtime_read_scope_for_actor_optional(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<Option<RuntimeAccessScope>> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordRead,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::All));
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordReadOwn,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::Own));
        }

        Ok(None)
    }

    async fn runtime_write_scope_for_actor_optional(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<Option<RuntimeAccessScope>> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWrite,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::All));
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWriteOwn,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::Own));
        }

        Ok(None)
    }

    async fn runtime_read_scope_for_actor(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<RuntimeAccessScope> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordRead,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::All);
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordReadOwn,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::Own);
        }

        Err(AppError::Forbidden(format!(
            "subject '{}' is missing runtime record read permissions in tenant '{}'",
            actor.subject(),
            actor.tenant_id()
        )))
    }

    async fn runtime_write_scope_for_actor(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<RuntimeAccessScope> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWrite,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::All);
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWriteOwn,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::Own);
        }

        Err(AppError::Forbidden(format!(
            "subject '{}' is missing runtime record write permissions in tenant '{}'",
            actor.subject(),
            actor.tenant_id()
        )))
    }

    async fn runtime_field_access_for_actor(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<crate::RuntimeFieldAccess>> {
        self.authorization_service
            .runtime_field_access(actor.tenant_id(), actor.subject(), entity_logical_name)
            .await
    }

    fn enforce_writable_fields(
        data: &Value,
        field_access: &crate::RuntimeFieldAccess,
    ) -> AppResult<()> {
        let object = data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;

        for key in object.keys() {
            if !field_access.writable_fields.contains(key.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "field '{}' is not writable for this subject",
                    key
                )));
            }
        }

        Ok(())
    }

    fn enforce_query_readable_fields(
        query: &RuntimeRecordQuery,
        field_access: &crate::RuntimeFieldAccess,
    ) -> AppResult<()> {
        for filter in &query.filters {
            if !field_access
                .readable_fields
                .contains(filter.field_logical_name.as_str())
            {
                return Err(AppError::Forbidden(format!(
                    "field '{}' is not readable for query filters",
                    filter.field_logical_name
                )));
            }
        }

        for sort in &query.sort {
            if !field_access
                .readable_fields
                .contains(sort.field_logical_name.as_str())
            {
                return Err(AppError::Forbidden(format!(
                    "field '{}' is not readable for query sorting",
                    sort.field_logical_name
                )));
            }
        }

        Ok(())
    }

    fn redact_runtime_records_if_needed(
        records: Vec<RuntimeRecord>,
        field_access: Option<&crate::RuntimeFieldAccess>,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let Some(field_access) = field_access else {
            return Ok(records);
        };

        records
            .into_iter()
            .map(|record| Self::redact_runtime_record(record, field_access))
            .collect()
    }

    fn redact_runtime_record_if_needed(
        record: RuntimeRecord,
        field_access: Option<&crate::RuntimeFieldAccess>,
    ) -> AppResult<RuntimeRecord> {
        let Some(field_access) = field_access else {
            return Ok(record);
        };

        Self::redact_runtime_record(record, field_access)
    }

    fn redact_runtime_record(
        record: RuntimeRecord,
        field_access: &crate::RuntimeFieldAccess,
    ) -> AppResult<RuntimeRecord> {
        let mut redacted = serde_json::Map::new();

        if let Some(object) = record.data().as_object() {
            for (key, value) in object {
                if field_access.readable_fields.contains(key.as_str()) {
                    redacted.insert(key.clone(), value.clone());
                }
            }
        }

        RuntimeRecord::new(
            record.record_id().as_str(),
            record.entity_logical_name().as_str(),
            Value::Object(redacted),
        )
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

    fn validate_runtime_query_filter(
        field: &EntityFieldDefinition,
        filter: &RuntimeRecordFilter,
    ) -> AppResult<()> {
        if field.field_type() != filter.field_type {
            return Err(AppError::Validation(format!(
                "query filter field type mismatch for '{}': expected '{}', got '{}'",
                filter.field_logical_name,
                field.field_type().as_str(),
                filter.field_type.as_str()
            )));
        }

        match filter.operator {
            RuntimeRecordOperator::Eq | RuntimeRecordOperator::Neq => {
                field.validate_runtime_value(&filter.field_value)?;
            }
            RuntimeRecordOperator::Gt
            | RuntimeRecordOperator::Gte
            | RuntimeRecordOperator::Lt
            | RuntimeRecordOperator::Lte => {
                if !matches!(
                    field.field_type(),
                    FieldType::Number | FieldType::Date | FieldType::DateTime
                ) {
                    return Err(AppError::Validation(format!(
                        "operator '{}' is not supported for field '{}' with type '{}'",
                        filter.operator.as_str(),
                        filter.field_logical_name,
                        field.field_type().as_str()
                    )));
                }

                field.validate_runtime_value(&filter.field_value)?;
            }
            RuntimeRecordOperator::Contains => {
                if field.field_type() != FieldType::Text {
                    return Err(AppError::Validation(format!(
                        "operator 'contains' requires text field type for '{}'",
                        filter.field_logical_name
                    )));
                }

                if !filter.field_value.is_string() {
                    return Err(AppError::Validation(format!(
                        "operator 'contains' requires string value for '{}'",
                        filter.field_logical_name
                    )));
                }
            }
            RuntimeRecordOperator::In => {
                let values = filter.field_value.as_array().ok_or_else(|| {
                    AppError::Validation(format!(
                        "operator 'in' requires array value for '{}'",
                        filter.field_logical_name
                    ))
                })?;

                if values.is_empty() {
                    return Err(AppError::Validation(format!(
                        "operator 'in' requires at least one value for '{}'",
                        filter.field_logical_name
                    )));
                }

                for value in values {
                    field.validate_runtime_value(value)?;
                }
            }
        }

        Ok(())
    }

    fn validate_runtime_query_sort(
        field: &EntityFieldDefinition,
        sort: &RuntimeRecordSort,
    ) -> AppResult<()> {
        if field.field_type() != sort.field_type {
            return Err(AppError::Validation(format!(
                "query sort field type mismatch for '{}': expected '{}', got '{}'",
                sort.field_logical_name,
                field.field_type().as_str(),
                sort.field_type.as_str()
            )));
        }

        if field.field_type() == FieldType::Json {
            return Err(AppError::Validation(format!(
                "sorting is not supported for json field '{}'",
                sort.field_logical_name
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
