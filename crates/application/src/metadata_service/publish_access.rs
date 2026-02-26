use super::*;

impl MetadataService {
    /// Lists standalone forms without permission checks.
    pub async fn list_forms_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        self.repository
            .list_forms(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Finds a standalone form without permission checks.
    pub async fn find_form_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        self.repository
            .find_form(actor.tenant_id(), entity_logical_name, form_logical_name)
            .await
    }

    /// Lists standalone views without permission checks.
    pub async fn list_views_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        self.repository
            .list_views(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Finds a standalone view without permission checks.
    pub async fn find_view_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        self.repository
            .find_view(actor.tenant_id(), entity_logical_name, view_logical_name)
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

    /// Returns latest published form snapshots for an entity.
    pub async fn list_latest_published_form_snapshots(
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
            .list_latest_published_form_snapshots(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Returns latest published view snapshots for an entity.
    pub async fn list_latest_published_view_snapshots(
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
            .list_latest_published_view_snapshots(actor.tenant_id(), entity_logical_name)
            .await
    }
}
