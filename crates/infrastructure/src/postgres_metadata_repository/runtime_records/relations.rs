use super::*;

impl PostgresMetadataRepository {
    pub(in super::super) async fn has_relation_reference_impl(
        &self,
        tenant_id: TenantId,
        target_entity_logical_name: &str,
        target_record_id: &str,
    ) -> AppResult<bool> {
        let latest_schemas = sqlx::query_as::<_, LatestSchemaRow>(
            r#"
            SELECT DISTINCT ON (entity_logical_name) schema_json
            FROM entity_published_versions
            WHERE tenant_id = $1
            ORDER BY entity_logical_name, version DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list latest published schemas for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        for row in latest_schemas {
            let schema: PublishedEntitySchema =
                serde_json::from_value(row.schema_json).map_err(|error| {
                    AppError::Internal(format!(
                        "persisted published schema is invalid for tenant '{}': {error}",
                        tenant_id
                    ))
                })?;

            let relation_field_names: Vec<String> = schema
                .fields()
                .iter()
                .filter(|field| {
                    field.field_type() == FieldType::Relation
                        && field
                            .relation_target_entity()
                            .map(|target| target.as_str() == target_entity_logical_name)
                            .unwrap_or(false)
                })
                .map(|field| field.logical_name().as_str().to_owned())
                .collect();

            if relation_field_names.is_empty() {
                continue;
            }

            for field_name in relation_field_names {
                let exists = sqlx::query_scalar::<_, bool>(
                    r#"
                    SELECT EXISTS (
                        SELECT 1
                        FROM runtime_records
                        WHERE tenant_id = $1
                          AND entity_logical_name = $2
                          AND data ->> $3 = $4
                    )
                    "#,
                )
                .bind(tenant_id.as_uuid())
                .bind(schema.entity().logical_name().as_str())
                .bind(field_name.as_str())
                .bind(target_record_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|error| {
                    AppError::Internal(format!(
                        "failed to evaluate relation reference for field '{}' in entity '{}' and tenant '{}': {error}",
                        field_name,
                        schema.entity().logical_name().as_str(),
                        tenant_id
                    ))
                })?;

                if exists {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
