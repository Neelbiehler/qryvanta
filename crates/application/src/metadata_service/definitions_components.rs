use super::*;

impl MetadataService {
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
        self.validate_form_definition(actor.tenant_id(), &schema, &form)
            .await?;

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
}
