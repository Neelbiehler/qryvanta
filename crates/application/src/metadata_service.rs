use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AuditAction, EntityDefinition, EntityFieldDefinition, FieldType, FormDefinition,
    OptionSetDefinition, Permission, PublishedEntitySchema, RuntimeRecord, ViewDefinition,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::AuthorizationService;
use crate::metadata_ports::{
    AuditEvent, AuditRepository, MetadataRepository, RecordListQuery, RuntimeRecordConditionGroup,
    RuntimeRecordConditionNode, RuntimeRecordFilter, RuntimeRecordOperator, RuntimeRecordQuery,
    RuntimeRecordSort, SaveFieldInput, SaveFormInput, SaveOptionSetInput, SaveViewInput,
    UniqueFieldValue, UpdateFieldInput,
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
        self.register_entity_with_details(actor, logical_name, display_name, None, None, None)
            .await
    }

    /// Registers a new entity definition with optional enriched metadata.
    pub async fn register_entity_with_details(
        &self,
        actor: &UserIdentity,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        description: Option<String>,
        plural_display_name: Option<String>,
        icon: Option<String>,
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

        let entity = EntityDefinition::new_with_details(
            logical_name,
            display_name,
            description,
            plural_display_name,
            icon,
        )?;
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

        if let Some(option_set_logical_name) = input.option_set_logical_name.as_deref() {
            let option_set_exists = self
                .repository
                .find_option_set(
                    actor.tenant_id(),
                    input.entity_logical_name.as_str(),
                    option_set_logical_name,
                )
                .await?
                .is_some();
            if !option_set_exists {
                return Err(AppError::NotFound(format!(
                    "option set '{}.{}' does not exist for tenant '{}'",
                    input.entity_logical_name,
                    option_set_logical_name,
                    actor.tenant_id()
                )));
            }
        }

        let field = EntityFieldDefinition::new_with_details(
            input.entity_logical_name,
            input.logical_name,
            input.display_name,
            input.field_type,
            input.is_required,
            input.is_unique,
            input.default_value,
            input.relation_target_entity,
            input.option_set_logical_name,
            None,
            None,
            None,
            None,
        )?;

        if let Some(existing) = self
            .repository
            .find_field(
                actor.tenant_id(),
                field.entity_logical_name().as_str(),
                field.logical_name().as_str(),
            )
            .await?
            && self
                .repository
                .field_exists_in_published_schema(
                    actor.tenant_id(),
                    field.entity_logical_name().as_str(),
                    field.logical_name().as_str(),
                )
                .await?
        {
            if existing.field_type() != field.field_type() {
                return Err(AppError::Validation(format!(
                    "field type cannot be changed for published field '{}.{}'",
                    field.entity_logical_name().as_str(),
                    field.logical_name().as_str()
                )));
            }

            if existing
                .relation_target_entity()
                .map(|value| value.as_str())
                != field.relation_target_entity().map(|value| value.as_str())
            {
                return Err(AppError::Validation(format!(
                    "relation target cannot be changed for published field '{}.{}'",
                    field.entity_logical_name().as_str(),
                    field.logical_name().as_str()
                )));
            }
        }

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

    /// Saves or updates an entity option set definition.
    pub async fn save_option_set(
        &self,
        actor: &UserIdentity,
        input: SaveOptionSetInput,
    ) -> AppResult<OptionSetDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        self.require_entity_exists(actor.tenant_id(), input.entity_logical_name.as_str())
            .await?;

        let option_set = OptionSetDefinition::new(
            input.entity_logical_name,
            input.logical_name,
            input.display_name,
            input.options,
        )?;

        self.repository
            .save_option_set(actor.tenant_id(), option_set.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_option_set_definition".to_owned(),
                resource_id: format!(
                    "{}.{}",
                    option_set.entity_logical_name().as_str(),
                    option_set.logical_name().as_str()
                ),
                detail: Some(format!(
                    "saved option set '{}' on entity '{}'",
                    option_set.logical_name().as_str(),
                    option_set.entity_logical_name().as_str()
                )),
            })
            .await?;

        Ok(option_set)
    }

    /// Lists option sets for an entity.
    pub async fn list_option_sets(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<OptionSetDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;

        self.repository
            .list_option_sets(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Finds a single option set by logical name.
    pub async fn find_option_set(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<Option<OptionSetDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;

        self.repository
            .find_option_set(
                actor.tenant_id(),
                entity_logical_name,
                option_set_logical_name,
            )
            .await
    }

    /// Deletes an option set definition.
    pub async fn delete_option_set(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        let fields = self
            .repository
            .list_fields(actor.tenant_id(), entity_logical_name)
            .await?;
        let in_use = fields.iter().any(|field| {
            field
                .option_set_logical_name()
                .map(|name| name.as_str() == option_set_logical_name)
                .unwrap_or(false)
        });
        if in_use {
            return Err(AppError::Conflict(format!(
                "option set '{}.{}' cannot be deleted because fields reference it",
                entity_logical_name, option_set_logical_name
            )));
        }

        self.repository
            .delete_option_set(
                actor.tenant_id(),
                entity_logical_name,
                option_set_logical_name,
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_option_set_definition".to_owned(),
                resource_id: format!("{entity_logical_name}.{option_set_logical_name}"),
                detail: Some(format!(
                    "deleted option set '{}' on entity '{}'",
                    option_set_logical_name, entity_logical_name
                )),
            })
            .await?;

        Ok(())
    }

    /// Saves or updates a standalone form definition.
    pub async fn save_form(
        &self,
        actor: &UserIdentity,
        input: SaveFormInput,
    ) -> AppResult<FormDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;
        self.require_entity_exists(actor.tenant_id(), input.entity_logical_name.as_str())
            .await?;

        let form = FormDefinition::new(
            input.entity_logical_name,
            input.logical_name,
            input.display_name,
            input.form_type,
            input.tabs,
            input.header_fields,
        )?;

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), form.entity_logical_name().as_str())
            .await?;
        Self::validate_form_definition(&schema, &form)?;

        self.repository
            .save_form(actor.tenant_id(), form.clone())
            .await?;
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_form_definition".to_owned(),
                resource_id: format!(
                    "{}.{}",
                    form.entity_logical_name().as_str(),
                    form.logical_name().as_str()
                ),
                detail: Some(format!(
                    "saved form '{}' on entity '{}'",
                    form.logical_name().as_str(),
                    form.entity_logical_name().as_str()
                )),
            })
            .await?;
        Ok(form)
    }

    /// Lists standalone forms for an entity.
    pub async fn list_forms(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;
        self.repository
            .list_forms(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Finds a standalone form by logical name.
    pub async fn find_form(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;
        self.repository
            .find_form(actor.tenant_id(), entity_logical_name, form_logical_name)
            .await
    }

    /// Deletes a standalone form definition.
    pub async fn delete_form(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;
        self.repository
            .delete_form(actor.tenant_id(), entity_logical_name, form_logical_name)
            .await?;
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_form_definition".to_owned(),
                resource_id: format!("{entity_logical_name}.{form_logical_name}"),
                detail: Some(format!(
                    "deleted form '{}' on entity '{}'",
                    form_logical_name, entity_logical_name
                )),
            })
            .await?;
        Ok(())
    }

    /// Saves or updates a standalone view definition.
    pub async fn save_view(
        &self,
        actor: &UserIdentity,
        input: SaveViewInput,
    ) -> AppResult<ViewDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;
        self.require_entity_exists(actor.tenant_id(), input.entity_logical_name.as_str())
            .await?;

        let view = ViewDefinition::new(
            input.entity_logical_name,
            input.logical_name,
            input.display_name,
            input.view_type,
            input.columns,
            input.default_sort,
            input.filter_criteria,
            input.is_default,
        )?;
        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), view.entity_logical_name().as_str())
            .await?;
        Self::validate_view_definition(&schema, &view)?;

        if view.is_default() {
            let existing = self
                .repository
                .list_views(actor.tenant_id(), view.entity_logical_name().as_str())
                .await?;
            if existing.iter().any(|existing_view| {
                existing_view.is_default()
                    && existing_view.logical_name().as_str() != view.logical_name().as_str()
            }) {
                return Err(AppError::Conflict(format!(
                    "entity '{}' already has a default view",
                    view.entity_logical_name().as_str()
                )));
            }
        }

        self.repository
            .save_view(actor.tenant_id(), view.clone())
            .await?;
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_view_definition".to_owned(),
                resource_id: format!(
                    "{}.{}",
                    view.entity_logical_name().as_str(),
                    view.logical_name().as_str()
                ),
                detail: Some(format!(
                    "saved view '{}' on entity '{}'",
                    view.logical_name().as_str(),
                    view.entity_logical_name().as_str()
                )),
            })
            .await?;
        Ok(view)
    }

    /// Lists standalone views for an entity.
    pub async fn list_views(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;
        self.repository
            .list_views(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Finds a standalone view by logical name.
    pub async fn find_view(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;
        self.repository
            .find_view(actor.tenant_id(), entity_logical_name, view_logical_name)
            .await
    }

    /// Deletes a standalone view definition.
    pub async fn delete_view(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;
        self.repository
            .delete_view(actor.tenant_id(), entity_logical_name, view_logical_name)
            .await?;
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_view_definition".to_owned(),
                resource_id: format!("{entity_logical_name}.{view_logical_name}"),
                detail: Some(format!(
                    "deleted view '{}' on entity '{}'",
                    view_logical_name, entity_logical_name
                )),
            })
            .await?;
        Ok(())
    }

    /// Updates mutable metadata attributes for an existing field.
    pub async fn update_field(
        &self,
        actor: &UserIdentity,
        input: UpdateFieldInput,
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

        let existing = self
            .repository
            .find_field(
                actor.tenant_id(),
                input.entity_logical_name.as_str(),
                input.logical_name.as_str(),
            )
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "field '{}.{}' does not exist for tenant '{}'",
                    input.entity_logical_name,
                    input.logical_name,
                    actor.tenant_id()
                ))
            })?;

        let updated = existing.with_mutable_updates(
            input.display_name,
            input.description,
            input.default_value,
            input.max_length,
            input.min_value,
            input.max_value,
        )?;

        self.repository
            .save_field(actor.tenant_id(), updated.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_field_definition".to_owned(),
                resource_id: format!(
                    "{}.{}",
                    updated.entity_logical_name().as_str(),
                    updated.logical_name().as_str()
                ),
                detail: Some(format!(
                    "updated metadata field '{}' on entity '{}'",
                    updated.logical_name().as_str(),
                    updated.entity_logical_name().as_str()
                )),
            })
            .await?;

        Ok(updated)
    }

    /// Deletes a draft field that has never been published.
    pub async fn delete_field(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        self.require_entity_exists(actor.tenant_id(), entity_logical_name)
            .await?;

        let field_exists = self
            .repository
            .find_field(actor.tenant_id(), entity_logical_name, field_logical_name)
            .await?
            .is_some();
        if !field_exists {
            return Err(AppError::NotFound(format!(
                "field '{}.{}' does not exist for tenant '{}'",
                entity_logical_name,
                field_logical_name,
                actor.tenant_id()
            )));
        }

        let published = self
            .repository
            .field_exists_in_published_schema(
                actor.tenant_id(),
                entity_logical_name,
                field_logical_name,
            )
            .await?;
        if published {
            return Err(AppError::Conflict(format!(
                "field '{}.{}' cannot be deleted because it exists in a published schema",
                entity_logical_name, field_logical_name
            )));
        }

        self.repository
            .delete_field(actor.tenant_id(), entity_logical_name, field_logical_name)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_field_definition".to_owned(),
                resource_id: format!("{entity_logical_name}.{field_logical_name}"),
                detail: Some(format!(
                    "deleted draft metadata field '{}' from entity '{}'",
                    field_logical_name, entity_logical_name
                )),
            })
            .await?;

        Ok(())
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
        let option_sets = self
            .repository
            .list_option_sets(actor.tenant_id(), entity_logical_name)
            .await?;

        if fields.is_empty() {
            return Err(AppError::Validation(format!(
                "entity '{}' requires at least one field before publishing",
                entity_logical_name
            )));
        }

        let published_schema = self
            .repository
            .publish_entity_schema(
                actor.tenant_id(),
                entity,
                fields,
                option_sets,
                actor.subject(),
            )
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

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        self.validate_runtime_query(
            actor,
            entity_logical_name,
            &schema,
            &mut query,
            field_access.as_ref(),
        )
        .await?;

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

    /// Queries runtime records without global permission checks.
    pub async fn query_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RuntimeRecordQuery,
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

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        self.validate_runtime_query(
            actor,
            entity_logical_name,
            &schema,
            &mut query,
            field_access.as_ref(),
        )
        .await?;

        let records = self
            .repository
            .query_runtime_records(actor.tenant_id(), entity_logical_name, query)
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

    async fn validate_runtime_query(
        &self,
        actor: &UserIdentity,
        root_entity_logical_name: &str,
        root_schema: &PublishedEntitySchema,
        query: &mut RuntimeRecordQuery,
        root_field_access: Option<&crate::RuntimeFieldAccess>,
    ) -> AppResult<()> {
        if query.limit == 0 {
            return Err(AppError::Validation(
                "runtime record query limit must be greater than zero".to_owned(),
            ));
        }

        let mut schema_cache = BTreeMap::new();
        schema_cache.insert(root_entity_logical_name.to_owned(), root_schema.clone());
        let alias_entities = self
            .resolve_runtime_query_links(actor, root_entity_logical_name, query, &mut schema_cache)
            .await?;

        let mut scope_field_access = BTreeMap::new();
        if let Some(access) = root_field_access {
            scope_field_access.insert(String::new(), access.clone());
        }

        let mut entity_field_access_cache = BTreeMap::new();
        for entity_logical_name in alias_entities.values() {
            if entity_field_access_cache.contains_key(entity_logical_name) {
                continue;
            }

            let field_access = self
                .runtime_field_access_for_actor(actor, entity_logical_name)
                .await?;
            entity_field_access_cache.insert(entity_logical_name.clone(), field_access);
        }

        for (alias, entity_logical_name) in &alias_entities {
            let Some(field_access) = entity_field_access_cache
                .get(entity_logical_name)
                .and_then(Option::as_ref)
            else {
                continue;
            };

            scope_field_access.insert(alias.clone(), field_access.clone());
        }

        Self::enforce_query_readable_fields(query, &scope_field_access)?;

        for filter in &query.filters {
            let field = Self::resolve_query_field_definition(
                root_entity_logical_name,
                &alias_entities,
                &schema_cache,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
                "filter",
            )?;
            Self::validate_runtime_query_filter(field, filter)?;
        }

        if let Some(where_clause) = &query.where_clause {
            Self::validate_runtime_query_group(
                root_entity_logical_name,
                &alias_entities,
                &schema_cache,
                where_clause,
            )?;
        }

        let mut seen_sort_fields = BTreeSet::new();
        for sort in &query.sort {
            let sort_scope_key = sort.scope_alias.clone().unwrap_or_default();
            if !seen_sort_fields.insert((sort_scope_key.clone(), sort.field_logical_name.clone())) {
                return Err(AppError::Validation(format!(
                    "duplicate runtime query sort field '{}' in scope '{}'",
                    sort.field_logical_name,
                    if sort_scope_key.is_empty() {
                        root_entity_logical_name
                    } else {
                        sort_scope_key.as_str()
                    }
                )));
            }

            let field = Self::resolve_query_field_definition(
                root_entity_logical_name,
                &alias_entities,
                &schema_cache,
                sort.scope_alias.as_deref(),
                sort.field_logical_name.as_str(),
                "sort",
            )?;
            Self::validate_runtime_query_sort(field, sort)?;
        }

        Ok(())
    }

    async fn resolve_runtime_query_links(
        &self,
        actor: &UserIdentity,
        root_entity_logical_name: &str,
        query: &mut RuntimeRecordQuery,
        schema_cache: &mut BTreeMap<String, PublishedEntitySchema>,
    ) -> AppResult<BTreeMap<String, String>> {
        let mut alias_entities = BTreeMap::new();

        for link in &mut query.links {
            if link.alias.trim().is_empty() {
                return Err(AppError::Validation(
                    "runtime query link alias cannot be empty".to_owned(),
                ));
            }

            if alias_entities.contains_key(link.alias.as_str()) {
                return Err(AppError::Validation(format!(
                    "duplicate runtime query link alias '{}'",
                    link.alias
                )));
            }

            let parent_entity_logical_name = match link.parent_alias.as_deref() {
                Some(parent_alias) if !parent_alias.trim().is_empty() => alias_entities
                    .get(parent_alias)
                    .map(String::as_str)
                    .ok_or_else(|| {
                        AppError::Validation(format!(
                            "unknown runtime query parent alias '{}'",
                            parent_alias
                        ))
                    })?,
                Some(_) => {
                    return Err(AppError::Validation(
                        "runtime query link parent_alias cannot be empty".to_owned(),
                    ));
                }
                None => root_entity_logical_name,
            };

            let parent_schema = self
                .load_runtime_query_schema(
                    actor.tenant_id(),
                    parent_entity_logical_name,
                    schema_cache,
                )
                .await?;

            let relation_field_name = link.relation_field_logical_name.trim();
            if relation_field_name.is_empty() {
                return Err(AppError::Validation(
                    "runtime query link relation_field_logical_name cannot be empty".to_owned(),
                ));
            }

            let Some(relation_field) = parent_schema
                .fields()
                .iter()
                .find(|field| field.logical_name().as_str() == relation_field_name)
            else {
                return Err(AppError::Validation(format!(
                    "unknown relation field '{}' for parent entity '{}'",
                    relation_field_name, parent_entity_logical_name
                )));
            };

            if relation_field.field_type() != FieldType::Relation {
                return Err(AppError::Validation(format!(
                    "link relation field '{}' on entity '{}' must be of type 'relation'",
                    relation_field_name, parent_entity_logical_name
                )));
            }

            let Some(target_entity) = relation_field.relation_target_entity() else {
                return Err(AppError::Validation(format!(
                    "relation field '{}' on entity '{}' is missing relation target metadata",
                    relation_field_name, parent_entity_logical_name
                )));
            };

            self.load_runtime_query_schema(actor.tenant_id(), target_entity.as_str(), schema_cache)
                .await?;

            if !link.target_entity_logical_name.is_empty()
                && link.target_entity_logical_name.as_str() != target_entity.as_str()
            {
                return Err(AppError::Validation(format!(
                    "runtime query link alias '{}' target entity mismatch: expected '{}', got '{}'",
                    link.alias,
                    target_entity.as_str(),
                    link.target_entity_logical_name
                )));
            }

            link.target_entity_logical_name = target_entity.as_str().to_owned();
            link.relation_field_logical_name = relation_field_name.to_owned();
            alias_entities.insert(link.alias.clone(), target_entity.as_str().to_owned());
        }

        Ok(alias_entities)
    }

    async fn load_runtime_query_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        schema_cache: &mut BTreeMap<String, PublishedEntitySchema>,
    ) -> AppResult<PublishedEntitySchema> {
        if let Some(schema) = schema_cache.get(entity_logical_name) {
            return Ok(schema.clone());
        }

        let schema = self
            .published_schema_for_runtime(tenant_id, entity_logical_name)
            .await?;
        schema_cache.insert(entity_logical_name.to_owned(), schema.clone());
        Ok(schema)
    }

    fn enforce_query_readable_fields(
        query: &RuntimeRecordQuery,
        scope_field_access: &BTreeMap<String, crate::RuntimeFieldAccess>,
    ) -> AppResult<()> {
        for filter in &query.filters {
            Self::enforce_scope_readable_field(
                scope_field_access,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
                "query filters",
            )?;
        }

        if let Some(where_clause) = &query.where_clause {
            Self::enforce_group_readable_fields(where_clause, scope_field_access)?;
        }

        for sort in &query.sort {
            Self::enforce_scope_readable_field(
                scope_field_access,
                sort.scope_alias.as_deref(),
                sort.field_logical_name.as_str(),
                "query sorting",
            )?;
        }

        Ok(())
    }

    fn enforce_group_readable_fields(
        group: &RuntimeRecordConditionGroup,
        scope_field_access: &BTreeMap<String, crate::RuntimeFieldAccess>,
    ) -> AppResult<()> {
        for node in &group.nodes {
            match node {
                RuntimeRecordConditionNode::Filter(filter) => Self::enforce_scope_readable_field(
                    scope_field_access,
                    filter.scope_alias.as_deref(),
                    filter.field_logical_name.as_str(),
                    "query filters",
                )?,
                RuntimeRecordConditionNode::Group(nested_group) => {
                    Self::enforce_group_readable_fields(nested_group, scope_field_access)?;
                }
            }
        }

        Ok(())
    }

    fn enforce_scope_readable_field(
        scope_field_access: &BTreeMap<String, crate::RuntimeFieldAccess>,
        scope_alias: Option<&str>,
        field_logical_name: &str,
        context: &str,
    ) -> AppResult<()> {
        let scope_key = scope_alias.unwrap_or_default();
        let Some(field_access) = scope_field_access.get(scope_key) else {
            return Ok(());
        };

        if field_access.readable_fields.contains(field_logical_name) {
            return Ok(());
        }

        if scope_key.is_empty() {
            return Err(AppError::Forbidden(format!(
                "field '{}' is not readable for {}",
                field_logical_name, context
            )));
        }

        Err(AppError::Forbidden(format!(
            "field '{}' is not readable for {} in alias '{}'",
            field_logical_name, context, scope_key
        )))
    }

    fn validate_runtime_query_group(
        root_entity_logical_name: &str,
        alias_entities: &BTreeMap<String, String>,
        schema_cache: &BTreeMap<String, PublishedEntitySchema>,
        group: &RuntimeRecordConditionGroup,
    ) -> AppResult<()> {
        if group.nodes.is_empty() {
            return Err(AppError::Validation(
                "runtime query where clause must include at least one condition or nested group"
                    .to_owned(),
            ));
        }

        for node in &group.nodes {
            match node {
                RuntimeRecordConditionNode::Filter(filter) => {
                    let field = Self::resolve_query_field_definition(
                        root_entity_logical_name,
                        alias_entities,
                        schema_cache,
                        filter.scope_alias.as_deref(),
                        filter.field_logical_name.as_str(),
                        "filter",
                    )?;
                    Self::validate_runtime_query_filter(field, filter)?;
                }
                RuntimeRecordConditionNode::Group(nested_group) => {
                    Self::validate_runtime_query_group(
                        root_entity_logical_name,
                        alias_entities,
                        schema_cache,
                        nested_group,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn resolve_query_field_definition<'a>(
        root_entity_logical_name: &str,
        alias_entities: &BTreeMap<String, String>,
        schema_cache: &'a BTreeMap<String, PublishedEntitySchema>,
        scope_alias: Option<&str>,
        field_logical_name: &str,
        context: &str,
    ) -> AppResult<&'a EntityFieldDefinition> {
        let scope_entity = match scope_alias {
            Some(alias) => alias_entities
                .get(alias)
                .map(String::as_str)
                .ok_or_else(|| {
                    AppError::Validation(format!("unknown runtime query scope alias '{}'", alias))
                })?,
            None => root_entity_logical_name,
        };

        let schema = schema_cache.get(scope_entity).ok_or_else(|| {
            AppError::Internal(format!(
                "runtime query schema cache missing entity '{}'",
                scope_entity
            ))
        })?;

        let field = schema
            .fields()
            .iter()
            .find(|field| field.logical_name().as_str() == field_logical_name)
            .ok_or_else(|| match scope_alias {
                Some(alias) => AppError::Validation(format!(
                    "unknown {} field '{}' for alias '{}'",
                    context, field_logical_name, alias
                )),
                None => AppError::Validation(format!(
                    "unknown {} field '{}' for entity '{}'",
                    context, field_logical_name, root_entity_logical_name
                )),
            })?;

        Ok(field)
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
                Self::validate_choice_value_against_option_set(schema, field, value)?;
                continue;
            }

            if let Some(default_value) = field.default_value() {
                Self::validate_choice_value_against_option_set(schema, field, default_value)?;
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

    fn validate_choice_value_against_option_set(
        schema: &PublishedEntitySchema,
        field: &EntityFieldDefinition,
        value: &Value,
    ) -> AppResult<()> {
        let Some(option_set_logical_name) = field.option_set_logical_name() else {
            return Ok(());
        };
        let Some(option_set) = schema
            .option_sets()
            .iter()
            .find(|set| set.logical_name().as_str() == option_set_logical_name.as_str())
        else {
            return Err(AppError::Validation(format!(
                "field '{}.{}' references unknown option set '{}'",
                field.entity_logical_name().as_str(),
                field.logical_name().as_str(),
                option_set_logical_name.as_str()
            )));
        };

        match field.field_type() {
            FieldType::Choice => {
                let selected = value.as_i64().ok_or_else(|| {
                    AppError::Validation(format!(
                        "choice field '{}' requires integer value",
                        field.logical_name().as_str()
                    ))
                })?;
                let selected = i32::try_from(selected).map_err(|_| {
                    AppError::Validation(format!(
                        "choice field '{}' value is out of supported range",
                        field.logical_name().as_str()
                    ))
                })?;
                if !option_set.contains_value(selected) {
                    return Err(AppError::Validation(format!(
                        "choice field '{}' includes unknown option value '{}'",
                        field.logical_name().as_str(),
                        selected
                    )));
                }
            }
            FieldType::MultiChoice => {
                let selected_values = value.as_array().ok_or_else(|| {
                    AppError::Validation(format!(
                        "multichoice field '{}' requires array value",
                        field.logical_name().as_str()
                    ))
                })?;
                for selected in selected_values {
                    let selected = selected.as_i64().ok_or_else(|| {
                        AppError::Validation(format!(
                            "multichoice field '{}' values must be integers",
                            field.logical_name().as_str()
                        ))
                    })?;
                    let selected = i32::try_from(selected).map_err(|_| {
                        AppError::Validation(format!(
                            "multichoice field '{}' value is out of supported range",
                            field.logical_name().as_str()
                        ))
                    })?;
                    if !option_set.contains_value(selected) {
                        return Err(AppError::Validation(format!(
                            "multichoice field '{}' includes unknown option value '{}'",
                            field.logical_name().as_str(),
                            selected
                        )));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn published_field_names(schema: &PublishedEntitySchema) -> BTreeSet<String> {
        schema
            .fields()
            .iter()
            .map(|field| field.logical_name().as_str().to_owned())
            .collect()
    }

    fn validate_form_definition(
        schema: &PublishedEntitySchema,
        form: &FormDefinition,
    ) -> AppResult<()> {
        let field_names = Self::published_field_names(schema);
        for header_field in form.header_fields() {
            if !field_names.contains(header_field) {
                return Err(AppError::Validation(format!(
                    "form header field '{}' does not exist in published schema for entity '{}'",
                    header_field,
                    form.entity_logical_name().as_str()
                )));
            }
        }
        for tab in form.tabs() {
            for section in tab.sections() {
                for field in section.fields() {
                    if !field_names.contains(field.field_logical_name().as_str()) {
                        return Err(AppError::Validation(format!(
                            "form field '{}' does not exist in published schema for entity '{}'",
                            field.field_logical_name().as_str(),
                            form.entity_logical_name().as_str()
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_view_definition(
        schema: &PublishedEntitySchema,
        view: &ViewDefinition,
    ) -> AppResult<()> {
        let field_names = Self::published_field_names(schema);
        for column in view.columns() {
            if !field_names.contains(column.field_logical_name().as_str()) {
                return Err(AppError::Validation(format!(
                    "view column '{}' does not exist in published schema for entity '{}'",
                    column.field_logical_name().as_str(),
                    view.entity_logical_name().as_str()
                )));
            }
        }
        if let Some(default_sort) = view.default_sort()
            && !field_names.contains(default_sort.field_logical_name().as_str())
        {
            return Err(AppError::Validation(format!(
                "view default sort field '{}' does not exist in published schema for entity '{}'",
                default_sort.field_logical_name().as_str(),
                view.entity_logical_name().as_str()
            )));
        }
        if let Some(filter_group) = view.filter_criteria() {
            for condition in filter_group.conditions() {
                if !field_names.contains(condition.field_logical_name().as_str()) {
                    return Err(AppError::Validation(format!(
                        "view filter field '{}' does not exist in published schema for entity '{}'",
                        condition.field_logical_name().as_str(),
                        view.entity_logical_name().as_str()
                    )));
                }
            }
        }
        Ok(())
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
