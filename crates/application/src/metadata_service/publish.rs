use super::*;

impl MetadataService {
    /// Publishes draft metadata for an entity as an immutable versioned schema.
    pub async fn publish_entity(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<PublishedEntitySchema> {
        self.publish_entity_with_allowed_unpublished_entities(actor, entity_logical_name, &[])
            .await
    }

    /// Publishes draft metadata for an entity while allowing unresolved relation targets
    /// that are part of the same in-flight publish selection.
    pub async fn publish_entity_with_allowed_unpublished_entities(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        allowed_unpublished_entity_logical_names: &[String],
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

        let publish_errors = self
            .collect_publish_validation_errors(
                actor.tenant_id(),
                entity_logical_name,
                &fields,
                allowed_unpublished_entity_logical_names,
            )
            .await?;
        if !publish_errors.is_empty() {
            return Err(AppError::Validation(
                Self::format_publish_validation_errors(entity_logical_name, &publish_errors),
            ));
        }

        let published_schema = self
            .repository
            .publish_entity_schema(
                actor.tenant_id(),
                entity,
                fields.clone(),
                option_sets,
                actor.subject(),
            )
            .await?;

        self.auto_generate_default_form(actor.tenant_id(), entity_logical_name, &fields)
            .await?;
        self.auto_generate_default_view(actor.tenant_id(), entity_logical_name, &fields)
            .await?;

        let forms = self
            .repository
            .list_forms(actor.tenant_id(), entity_logical_name)
            .await?;
        let views = self
            .repository
            .list_views(actor.tenant_id(), entity_logical_name)
            .await?;
        self.repository
            .save_published_form_snapshots(
                actor.tenant_id(),
                entity_logical_name,
                published_schema.version(),
                &forms,
            )
            .await?;
        self.repository
            .save_published_view_snapshots(
                actor.tenant_id(),
                entity_logical_name,
                published_schema.version(),
                &views,
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

    /// Runs publish validation checks without creating a new published schema version.
    pub async fn publish_checks(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<String>> {
        self.publish_checks_with_allowed_unpublished_entities(actor, entity_logical_name, &[])
            .await
    }

    /// Runs publish validation checks while allowing unresolved relation targets
    /// that are part of the same in-flight publish selection.
    pub async fn publish_checks_with_allowed_unpublished_entities(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        allowed_unpublished_entity_logical_names: &[String],
    ) -> AppResult<Vec<String>> {
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

        let _entity = self
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

        self.collect_publish_validation_errors(
            actor.tenant_id(),
            entity_logical_name,
            &fields,
            allowed_unpublished_entity_logical_names,
        )
        .await
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
}
