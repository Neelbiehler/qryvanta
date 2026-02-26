use super::*;

impl InMemoryMetadataRepository {
    pub(super) async fn publish_entity_schema_impl(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
        option_sets: Vec<OptionSetDefinition>,
        _published_by: &str,
    ) -> AppResult<PublishedEntitySchema> {
        let mut published_schemas = self.published_schemas.write().await;
        let versions = published_schemas
            .entry((tenant_id, entity.logical_name().as_str().to_owned()))
            .or_default();

        let version = versions
            .last()
            .map(|schema| schema.version() + 1)
            .unwrap_or(1);
        let schema = PublishedEntitySchema::new(entity, version, fields, option_sets)?;
        versions.push(schema.clone());

        Ok(schema)
    }

    pub(super) async fn latest_published_schema_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        Ok(self
            .published_schemas
            .read()
            .await
            .get(&(tenant_id, entity_logical_name.to_owned()))
            .and_then(|versions| versions.last().cloned()))
    }

    pub(super) async fn save_published_form_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        forms: &[FormDefinition],
    ) -> AppResult<()> {
        self.published_form_snapshots.write().await.insert(
            (
                tenant_id,
                entity_logical_name.to_owned(),
                published_schema_version,
            ),
            forms.to_vec(),
        );
        Ok(())
    }

    pub(super) async fn save_published_view_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        views: &[ViewDefinition],
    ) -> AppResult<()> {
        self.published_view_snapshots.write().await.insert(
            (
                tenant_id,
                entity_logical_name.to_owned(),
                published_schema_version,
            ),
            views.to_vec(),
        );
        Ok(())
    }

    pub(super) async fn list_latest_published_form_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        let snapshots = self.published_form_snapshots.read().await;
        let latest_version = snapshots
            .keys()
            .filter_map(|(stored_tenant, stored_entity, version)| {
                (stored_tenant == &tenant_id && stored_entity == entity_logical_name)
                    .then_some(*version)
            })
            .max();

        let Some(version) = latest_version else {
            return Ok(Vec::new());
        };

        Ok(snapshots
            .get(&(tenant_id, entity_logical_name.to_owned(), version))
            .cloned()
            .unwrap_or_default())
    }

    pub(super) async fn list_latest_published_view_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        let snapshots = self.published_view_snapshots.read().await;
        let latest_version = snapshots
            .keys()
            .filter_map(|(stored_tenant, stored_entity, version)| {
                (stored_tenant == &tenant_id && stored_entity == entity_logical_name)
                    .then_some(*version)
            })
            .max();

        let Some(version) = latest_version else {
            return Ok(Vec::new());
        };

        Ok(snapshots
            .get(&(tenant_id, entity_logical_name.to_owned(), version))
            .cloned()
            .unwrap_or_default())
    }
}
