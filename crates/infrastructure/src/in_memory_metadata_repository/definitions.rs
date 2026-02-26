use super::*;

impl InMemoryMetadataRepository {
    pub(super) async fn save_entity_impl(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
    ) -> AppResult<()> {
        let key = (tenant_id, entity.logical_name().as_str().to_owned());
        let mut entities = self.entities.write().await;

        if entities.contains_key(&key) {
            return Err(AppError::Conflict(format!(
                "entity '{}' already exists for tenant '{}'",
                key.1, key.0
            )));
        }

        entities.insert(key, entity);
        Ok(())
    }

    pub(super) async fn list_entities_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<Vec<EntityDefinition>> {
        let entities = self.entities.read().await;

        let mut values: Vec<EntityDefinition> = entities
            .iter()
            .filter_map(|((stored_tenant_id, _), entity)| {
                (stored_tenant_id == &tenant_id).then_some(entity.clone())
            })
            .collect();
        values.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });

        Ok(values)
    }

    pub(super) async fn find_entity_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<EntityDefinition>> {
        Ok(self
            .entities
            .read()
            .await
            .get(&(tenant_id, logical_name.to_owned()))
            .cloned())
    }

    pub(super) async fn update_entity_impl(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
    ) -> AppResult<()> {
        let key = (tenant_id, entity.logical_name().as_str().to_owned());
        let mut entities = self.entities.write().await;

        if !entities.contains_key(&key) {
            return Err(AppError::NotFound(format!(
                "entity '{}' does not exist for tenant '{}'",
                key.1, key.0
            )));
        }

        entities.insert(key, entity);
        Ok(())
    }

    pub(super) async fn save_field_impl(
        &self,
        tenant_id: TenantId,
        field: EntityFieldDefinition,
    ) -> AppResult<()> {
        self.fields.write().await.insert(
            (
                tenant_id,
                field.entity_logical_name().as_str().to_owned(),
                field.logical_name().as_str().to_owned(),
            ),
            field,
        );

        Ok(())
    }

    pub(super) async fn list_fields_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>> {
        let fields = self.fields.read().await;
        let mut listed: Vec<EntityFieldDefinition> = fields
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), field)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
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

    pub(super) async fn find_field_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<Option<EntityFieldDefinition>> {
        Ok(self
            .fields
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                field_logical_name.to_owned(),
            ))
            .cloned())
    }

    pub(super) async fn delete_field_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.fields.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            field_logical_name.to_owned(),
        ));

        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "field '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, field_logical_name, tenant_id
            )));
        }

        Ok(())
    }

    pub(super) async fn field_exists_in_published_schema_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<bool> {
        let published = self.published_schemas.read().await;
        let Some(versions) = published.get(&(tenant_id, entity_logical_name.to_owned())) else {
            return Ok(false);
        };

        Ok(versions.iter().any(|schema| {
            schema
                .fields()
                .iter()
                .any(|field| field.logical_name().as_str() == field_logical_name)
        }))
    }
}
