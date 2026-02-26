use super::*;

impl PostgresMetadataRepository {
    pub(super) async fn publish_entity_schema_impl(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
        option_sets: Vec<OptionSetDefinition>,
        published_by: &str,
    ) -> AppResult<PublishedEntitySchema> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start metadata publish transaction for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        let next_version: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(MAX(version), 0) + 1
            FROM entity_published_versions
            WHERE tenant_id = $1 AND entity_logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity.logical_name().as_str())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to compute next published schema version for entity '{}' in tenant '{}': {error}",
                entity.logical_name().as_str(),
                tenant_id
            ))
        })?;

        let schema = PublishedEntitySchema::new(entity.clone(), next_version, fields, option_sets)?;
        let schema_json = serde_json::to_value(&schema).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize published schema for entity '{}' in tenant '{}': {error}",
                entity.logical_name().as_str(),
                tenant_id
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO entity_published_versions (
                tenant_id,
                entity_logical_name,
                version,
                schema_json,
                published_by_subject
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity.logical_name().as_str())
        .bind(next_version)
        .bind(schema_json)
        .bind(published_by)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to persist published schema for entity '{}' in tenant '{}': {error}",
                entity.logical_name().as_str(),
                tenant_id
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit metadata publish transaction for entity '{}' in tenant '{}': {error}",
                entity.logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(schema)
    }

    pub(super) async fn latest_published_schema_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        let row = sqlx::query_as::<_, PublishedSchemaRow>(
            r#"
            SELECT version, schema_json
            FROM entity_published_versions
            WHERE tenant_id = $1 AND entity_logical_name = $2
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load latest published schema for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        let schema: PublishedEntitySchema =
            serde_json::from_value(row.schema_json).map_err(|error| {
                AppError::Internal(format!(
                    "persisted published schema is invalid for entity '{}' in tenant '{}': {error}",
                    entity_logical_name, tenant_id
                ))
            })?;

        if schema.version() != row.version {
            return Err(AppError::Internal(format!(
                "persisted published schema version mismatch for entity '{}' in tenant '{}'",
                entity_logical_name, tenant_id
            )));
        }

        Ok(Some(schema))
    }

    pub(super) async fn save_published_form_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        forms: &[FormDefinition],
    ) -> AppResult<()> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start form snapshot transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        sqlx::query(
            r#"
            DELETE FROM entity_form_published_versions
            WHERE tenant_id = $1
              AND entity_logical_name = $2
              AND published_schema_version = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(published_schema_version)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to clear form snapshots for entity '{}' version {} in tenant '{}': {error}",
                entity_logical_name, published_schema_version, tenant_id
            ))
        })?;

        for form in forms {
            let definition_json = serde_json::to_value(form).map_err(|error| {
                AppError::Internal(format!(
                    "failed to serialize published form snapshot '{}.{}': {error}",
                    entity_logical_name,
                    form.logical_name().as_str()
                ))
            })?;

            sqlx::query(
                r#"
                INSERT INTO entity_form_published_versions (
                    tenant_id,
                    entity_logical_name,
                    published_schema_version,
                    logical_name,
                    definition_json
                )
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(entity_logical_name)
            .bind(published_schema_version)
            .bind(form.logical_name().as_str())
            .bind(definition_json)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to save published form snapshot '{}.{}' in tenant '{}': {error}",
                    entity_logical_name,
                    form.logical_name().as_str(),
                    tenant_id
                ))
            })?;
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit form snapshot transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn save_published_view_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        published_schema_version: i32,
        views: &[ViewDefinition],
    ) -> AppResult<()> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start view snapshot transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        sqlx::query(
            r#"
            DELETE FROM entity_view_published_versions
            WHERE tenant_id = $1
              AND entity_logical_name = $2
              AND published_schema_version = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(published_schema_version)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to clear view snapshots for entity '{}' version {} in tenant '{}': {error}",
                entity_logical_name, published_schema_version, tenant_id
            ))
        })?;

        for view in views {
            let definition_json = serde_json::to_value(view).map_err(|error| {
                AppError::Internal(format!(
                    "failed to serialize published view snapshot '{}.{}': {error}",
                    entity_logical_name,
                    view.logical_name().as_str()
                ))
            })?;

            sqlx::query(
                r#"
                INSERT INTO entity_view_published_versions (
                    tenant_id,
                    entity_logical_name,
                    published_schema_version,
                    logical_name,
                    definition_json
                )
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(entity_logical_name)
            .bind(published_schema_version)
            .bind(view.logical_name().as_str())
            .bind(definition_json)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to save published view snapshot '{}.{}' in tenant '{}': {error}",
                    entity_logical_name,
                    view.logical_name().as_str(),
                    tenant_id
                ))
            })?;
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit view snapshot transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_latest_published_form_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        let rows = sqlx::query_as::<_, FormRow>(
            r#"
            SELECT definition_json
            FROM entity_form_published_versions
            WHERE tenant_id = $1
              AND entity_logical_name = $2
              AND published_schema_version = (
                  SELECT MAX(published_schema_version)
                  FROM entity_form_published_versions
                  WHERE tenant_id = $1 AND entity_logical_name = $2
              )
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list latest published form snapshots for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<FormDefinition>(row.definition_json).map_err(|error| {
                    AppError::Internal(format!(
                        "persisted published form snapshot is invalid for entity '{}' in tenant '{}': {error}",
                        entity_logical_name, tenant_id
                    ))
                })
            })
            .collect()
    }

    pub(super) async fn list_latest_published_view_snapshots_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        let rows = sqlx::query_as::<_, ViewRow>(
            r#"
            SELECT definition_json
            FROM entity_view_published_versions
            WHERE tenant_id = $1
              AND entity_logical_name = $2
              AND published_schema_version = (
                  SELECT MAX(published_schema_version)
                  FROM entity_view_published_versions
                  WHERE tenant_id = $1 AND entity_logical_name = $2
              )
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list latest published view snapshots for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<ViewDefinition>(row.definition_json).map_err(|error| {
                    AppError::Internal(format!(
                        "persisted published view snapshot is invalid for entity '{}' in tenant '{}': {error}",
                        entity_logical_name, tenant_id
                    ))
                })
            })
            .collect()
    }
}
