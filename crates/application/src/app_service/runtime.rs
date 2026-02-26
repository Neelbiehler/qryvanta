use super::*;

impl AppService {
    /// Returns effective capabilities for one app entity and subject.
    pub async fn entity_capabilities_for_subject(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<SubjectEntityPermission> {
        self.ensure_subject_can_access_app(actor, app_logical_name)
            .await?;

        self.repository
            .subject_entity_permission(
                actor.tenant_id(),
                actor.subject(),
                app_logical_name,
                entity_logical_name,
            )
            .await?
            .ok_or_else(|| {
                AppError::Forbidden(format!(
                    "subject '{}' has no app capabilities for entity '{}' in app '{}'",
                    actor.subject(),
                    entity_logical_name,
                    app_logical_name
                ))
            })
    }

    /// Fetches published schema for a worker-facing app entity.
    pub async fn schema_for_subject(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<PublishedEntitySchema> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .latest_published_schema_unchecked(actor, entity_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "entity '{}' has no published schema",
                    entity_logical_name
                ))
            })
    }

    /// Lists runtime records in app scope.
    pub async fn list_records(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .list_runtime_records_unchecked(actor, entity_logical_name, query)
            .await
    }

    /// Queries runtime records in app scope.
    pub async fn query_records(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        for link in &query.links {
            self.require_entity_action(
                actor,
                app_logical_name,
                link.target_entity_logical_name.as_str(),
                AppEntityAction::Read,
            )
            .await?;
        }

        self.runtime_record_service
            .query_runtime_records_unchecked(actor, entity_logical_name, query)
            .await
    }

    /// Fetches one runtime record in app scope.
    pub async fn get_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .get_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }

    /// Creates one runtime record in app scope.
    pub async fn create_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Create,
        )
        .await?;

        self.runtime_record_service
            .create_runtime_record_unchecked(actor, entity_logical_name, data)
            .await
    }

    /// Updates one runtime record in app scope.
    pub async fn update_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Update,
        )
        .await?;

        self.runtime_record_service
            .update_runtime_record_unchecked(actor, entity_logical_name, record_id, data)
            .await
    }

    /// Lists standalone forms for a worker-facing app entity.
    pub async fn list_entity_forms(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .list_forms_unchecked(actor, entity_logical_name)
            .await
    }

    /// Fetches one standalone form for a worker-facing app entity.
    pub async fn get_entity_form(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<FormDefinition> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .find_form_unchecked(actor, entity_logical_name, form_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "form '{}' does not exist for entity '{}'",
                    form_logical_name, entity_logical_name
                ))
            })
    }

    /// Lists standalone views for a worker-facing app entity.
    pub async fn list_entity_views(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .list_views_unchecked(actor, entity_logical_name)
            .await
    }

    /// Fetches one standalone view for a worker-facing app entity.
    pub async fn get_entity_view(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<ViewDefinition> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .find_view_unchecked(actor, entity_logical_name, view_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "view '{}' does not exist for entity '{}'",
                    view_logical_name, entity_logical_name
                ))
            })
    }

    /// Deletes one runtime record in app scope.
    pub async fn delete_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Delete,
        )
        .await?;

        self.runtime_record_service
            .delete_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }
}
