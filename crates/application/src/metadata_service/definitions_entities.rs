use super::*;

impl MetadataService {
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

    /// Updates mutable metadata attributes for an existing entity.
    pub async fn update_entity(
        &self,
        actor: &UserIdentity,
        input: UpdateEntityInput,
    ) -> AppResult<EntityDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataEntityCreate,
            )
            .await?;

        let existing = self
            .repository
            .find_entity(actor.tenant_id(), input.logical_name.as_str())
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "entity '{}' does not exist for tenant '{}'",
                    input.logical_name,
                    actor.tenant_id()
                ))
            })?;

        let updated = existing.with_updates(
            input.display_name,
            input.description,
            input.plural_display_name,
            input.icon,
        )?;

        self.repository
            .update_entity(actor.tenant_id(), updated.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataEntityCreated,
                resource_type: "entity_definition".to_owned(),
                resource_id: updated.logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "updated metadata entity '{}'",
                    updated.logical_name().as_str()
                )),
            })
            .await?;

        Ok(updated)
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

        let field = EntityFieldDefinition::new_with_details_and_calculation(
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
            input.calculation_expression,
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

        let updated =
            existing.with_mutable_updates_and_calculation(EntityFieldMutableUpdateInput {
                display_name: input.display_name,
                description: input.description,
                default_value: input.default_value,
                calculation_expression: input.calculation_expression,
                max_length: input.max_length,
                min_value: input.min_value,
                max_value: input.max_value,
            })?;

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
}
