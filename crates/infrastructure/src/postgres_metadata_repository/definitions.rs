use super::*;

impl PostgresMetadataRepository {
    pub(super) async fn save_entity_impl(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO entity_definitions (
                tenant_id,
                logical_name,
                display_name,
                description,
                plural_display_name,
                icon
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity.logical_name().as_str())
        .bind(entity.display_name().as_str())
        .bind(entity.description())
        .bind(entity.plural_display_name().map(|value| value.as_str()))
        .bind(entity.icon())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                if let sqlx::Error::Database(database_error) = &error
                    && database_error.code().as_deref() == Some("23505")
                {
                    return Err(AppError::Conflict(format!(
                        "entity '{}' already exists for tenant '{}'",
                        entity.logical_name().as_str(),
                        tenant_id
                    )));
                }

                Err(AppError::Internal(format!(
                    "failed to save entity definition: {error}"
                )))
            }
        }
    }

    pub(super) async fn list_entities_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<Vec<EntityDefinition>> {
        let rows = sqlx::query_as::<_, EntityRow>(
            r#"
            SELECT logical_name, display_name, description, plural_display_name, icon
            FROM entity_definitions
            WHERE tenant_id = $1
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to list entity definitions: {error}"))
        })?;

        rows.into_iter()
            .map(|row| {
                EntityDefinition::new_with_details(
                    row.logical_name,
                    row.display_name,
                    row.description,
                    row.plural_display_name,
                    row.icon,
                )
                .map_err(|error| {
                    AppError::Internal(format!(
                        "persisted entity definition is invalid for tenant '{}': {error}",
                        tenant_id
                    ))
                })
            })
            .collect()
    }

    pub(super) async fn find_entity_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<EntityDefinition>> {
        let row = sqlx::query_as::<_, EntityRow>(
            r#"
            SELECT logical_name, display_name, description, plural_display_name, icon
            FROM entity_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find entity definition '{}' for tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;

        row.map(|row| {
            EntityDefinition::new_with_details(
                row.logical_name,
                row.display_name,
                row.description,
                row.plural_display_name,
                row.icon,
            )
        })
        .transpose()
    }

    pub(super) async fn update_entity_impl(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
    ) -> AppResult<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE entity_definitions
            SET display_name = $3,
                description = $4,
                plural_display_name = $5,
                icon = $6
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity.logical_name().as_str())
        .bind(entity.display_name().as_str())
        .bind(entity.description())
        .bind(entity.plural_display_name().map(|value| value.as_str()))
        .bind(entity.icon())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to update entity definition: {error}"))
        })?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "entity '{}' does not exist for tenant '{}'",
                entity.logical_name().as_str(),
                tenant_id
            )));
        }

        Ok(())
    }

    pub(super) async fn save_field_impl(
        &self,
        tenant_id: TenantId,
        field: EntityFieldDefinition,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO entity_fields (
                tenant_id,
                entity_logical_name,
                logical_name,
                display_name,
                field_type,
                is_required,
                is_unique,
                default_value,
                relation_target_entity,
                option_set_logical_name,
                description,
                calculation_expression,
                max_length,
                min_value,
                max_value,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
            ON CONFLICT (tenant_id, entity_logical_name, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                field_type = EXCLUDED.field_type,
                is_required = EXCLUDED.is_required,
                is_unique = EXCLUDED.is_unique,
                default_value = EXCLUDED.default_value,
                relation_target_entity = EXCLUDED.relation_target_entity,
                option_set_logical_name = EXCLUDED.option_set_logical_name,
                description = EXCLUDED.description,
                calculation_expression = EXCLUDED.calculation_expression,
                max_length = EXCLUDED.max_length,
                min_value = EXCLUDED.min_value,
                max_value = EXCLUDED.max_value,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(field.entity_logical_name().as_str())
        .bind(field.logical_name().as_str())
        .bind(field.display_name().as_str())
        .bind(field.field_type().as_str())
        .bind(field.is_required())
        .bind(field.is_unique())
        .bind(field.default_value())
        .bind(field.relation_target_entity().map(|value| value.as_str()))
        .bind(field.option_set_logical_name().map(|value| value.as_str()))
        .bind(field.description())
        .bind(field.calculation_expression())
        .bind(field.max_length())
        .bind(field.min_value())
        .bind(field.max_value())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save field '{}' for entity '{}' in tenant '{}': {error}",
                field.logical_name().as_str(),
                field.entity_logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_fields_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>> {
        let rows = sqlx::query_as::<_, FieldRow>(
            r#"
            SELECT
                entity_logical_name,
                logical_name,
                display_name,
                field_type,
                is_required,
                is_unique,
                default_value,
                relation_target_entity,
                option_set_logical_name,
                description,
                calculation_expression,
                max_length,
                min_value,
                max_value
            FROM entity_fields
            WHERE tenant_id = $1 AND entity_logical_name = $2
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list fields for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                let field_type = FieldType::from_str(row.field_type.as_str())?;
                EntityFieldDefinition::new_with_details_and_calculation(
                    row.entity_logical_name,
                    row.logical_name,
                    row.display_name,
                    field_type,
                    row.is_required,
                    row.is_unique,
                    row.default_value,
                    row.relation_target_entity,
                    row.option_set_logical_name,
                    row.description,
                    row.calculation_expression,
                    row.max_length,
                    row.min_value,
                    row.max_value,
                )
            })
            .collect()
    }

    pub(super) async fn find_field_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<Option<EntityFieldDefinition>> {
        let row = sqlx::query_as::<_, FieldRow>(
            r#"
            SELECT
                entity_logical_name,
                logical_name,
                display_name,
                field_type,
                is_required,
                is_unique,
                default_value,
                relation_target_entity,
                option_set_logical_name,
                description,
                calculation_expression,
                max_length,
                min_value,
                max_value
            FROM entity_fields
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(field_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find field '{}.{}' in tenant '{}': {error}",
                entity_logical_name, field_logical_name, tenant_id
            ))
        })?;

        row.map(|row| {
            let field_type = FieldType::from_str(row.field_type.as_str())?;
            EntityFieldDefinition::new_with_details_and_calculation(
                row.entity_logical_name,
                row.logical_name,
                row.display_name,
                field_type,
                row.is_required,
                row.is_unique,
                row.default_value,
                row.relation_target_entity,
                row.option_set_logical_name,
                row.description,
                row.calculation_expression,
                row.max_length,
                row.min_value,
                row.max_value,
            )
        })
        .transpose()
    }

    pub(super) async fn delete_field_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM entity_fields
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(field_logical_name)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to delete field '{}.{}' in tenant '{}': {error}",
                entity_logical_name, field_logical_name, tenant_id
            ))
        })?;

        if result.rows_affected() == 0 {
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
        sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM entity_published_versions
                WHERE tenant_id = $1
                  AND entity_logical_name = $2
                  AND schema_json @> jsonb_build_object(
                    'fields',
                    jsonb_build_array(jsonb_build_object('logical_name', $3))
                  )
            )
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(field_logical_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to evaluate published-field usage for '{}.{}' in tenant '{}': {error}",
                entity_logical_name, field_logical_name, tenant_id
            ))
        })
    }
}
