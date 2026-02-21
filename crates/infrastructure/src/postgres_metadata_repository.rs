use std::str::FromStr;

use async_trait::async_trait;
use qryvanta_application::{
    MetadataRepository, RecordListQuery, RuntimeRecordQuery, UniqueFieldValue,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    EntityDefinition, EntityFieldDefinition, FieldType, PublishedEntitySchema, RuntimeRecord,
};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

/// PostgreSQL-backed metadata repository.
#[derive(Clone)]
pub struct PostgresMetadataRepository {
    pool: PgPool,
}

impl PostgresMetadataRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct EntityRow {
    logical_name: String,
    display_name: String,
}

#[derive(Debug, FromRow)]
struct FieldRow {
    entity_logical_name: String,
    logical_name: String,
    display_name: String,
    field_type: String,
    is_required: bool,
    is_unique: bool,
    default_value: Option<Value>,
    relation_target_entity: Option<String>,
}

#[derive(Debug, FromRow)]
struct PublishedSchemaRow {
    version: i32,
    schema_json: Value,
}

#[derive(Debug, FromRow)]
struct LatestSchemaRow {
    schema_json: Value,
}

#[derive(Debug, FromRow)]
struct RuntimeRecordRow {
    id: Uuid,
    entity_logical_name: String,
    data: Value,
}

#[async_trait]
impl MetadataRepository for PostgresMetadataRepository {
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO entity_definitions (tenant_id, logical_name, display_name)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity.logical_name().as_str())
        .bind(entity.display_name().as_str())
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

    async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
        let rows = sqlx::query_as::<_, EntityRow>(
            r#"
            SELECT logical_name, display_name
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
                EntityDefinition::new(row.logical_name, row.display_name).map_err(|error| {
                    AppError::Internal(format!(
                        "persisted entity definition is invalid for tenant '{}': {error}",
                        tenant_id
                    ))
                })
            })
            .collect()
    }

    async fn find_entity(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<EntityDefinition>> {
        let row = sqlx::query_as::<_, EntityRow>(
            r#"
            SELECT logical_name, display_name
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

        row.map(|row| EntityDefinition::new(row.logical_name, row.display_name))
            .transpose()
    }

    async fn save_field(&self, tenant_id: TenantId, field: EntityFieldDefinition) -> AppResult<()> {
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
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
            ON CONFLICT (tenant_id, entity_logical_name, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                field_type = EXCLUDED.field_type,
                is_required = EXCLUDED.is_required,
                is_unique = EXCLUDED.is_unique,
                default_value = EXCLUDED.default_value,
                relation_target_entity = EXCLUDED.relation_target_entity,
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

    async fn list_fields(
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
                relation_target_entity
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
                EntityFieldDefinition::new(
                    row.entity_logical_name,
                    row.logical_name,
                    row.display_name,
                    field_type,
                    row.is_required,
                    row.is_unique,
                    row.default_value,
                    row.relation_target_entity,
                )
            })
            .collect()
    }

    async fn publish_entity_schema(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
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

        let schema = PublishedEntitySchema::new(entity.clone(), next_version, fields)?;
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

    async fn latest_published_schema(
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

    async fn create_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start runtime record create transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        let created = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            INSERT INTO runtime_records (tenant_id, entity_logical_name, data)
            VALUES ($1, $2, $3)
            RETURNING id, entity_logical_name, data
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(&data)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to create runtime record for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        for unique_value in unique_values {
            let result = sqlx::query(
                r#"
                INSERT INTO runtime_record_unique_values (
                    tenant_id,
                    entity_logical_name,
                    field_logical_name,
                    field_value_hash,
                    record_id
                )
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(entity_logical_name)
            .bind(unique_value.field_logical_name.as_str())
            .bind(unique_value.field_value_hash.as_str())
            .bind(created.id)
            .execute(&mut *transaction)
            .await;

            if let Err(error) = result {
                if let sqlx::Error::Database(database_error) = &error
                    && database_error.code().as_deref() == Some("23505")
                {
                    return Err(AppError::Conflict(format!(
                        "unique constraint violated for field '{}'",
                        unique_value.field_logical_name
                    )));
                }

                return Err(AppError::Internal(format!(
                    "failed to index unique value for field '{}' on entity '{}' in tenant '{}': {error}",
                    unique_value.field_logical_name, entity_logical_name, tenant_id
                )));
            }
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record create transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        RuntimeRecord::new(
            created.id.to_string(),
            created.entity_logical_name,
            created.data,
        )
    }

    async fn update_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        let record_uuid = Uuid::parse_str(record_id).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime record id '{}': {error}",
                record_id
            ))
        })?;

        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start runtime record update transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        let updated = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            UPDATE runtime_records
            SET data = $4,
                updated_at = now()
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            RETURNING id, entity_logical_name, data
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .bind(&data)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to update runtime record '{}' for entity '{}' in tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            ))
        })?;

        sqlx::query(
            r#"
            DELETE FROM runtime_record_unique_values
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND record_id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to clear unique field index for runtime record '{}' in entity '{}' and tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?;

        for unique_value in unique_values {
            let result = sqlx::query(
                r#"
                INSERT INTO runtime_record_unique_values (
                    tenant_id,
                    entity_logical_name,
                    field_logical_name,
                    field_value_hash,
                    record_id
                )
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(entity_logical_name)
            .bind(unique_value.field_logical_name.as_str())
            .bind(unique_value.field_value_hash.as_str())
            .bind(record_uuid)
            .execute(&mut *transaction)
            .await;

            if let Err(error) = result {
                if let sqlx::Error::Database(database_error) = &error
                    && database_error.code().as_deref() == Some("23505")
                {
                    return Err(AppError::Conflict(format!(
                        "unique constraint violated for field '{}'",
                        unique_value.field_logical_name
                    )));
                }

                return Err(AppError::Internal(format!(
                    "failed to index unique value for field '{}' on entity '{}' in tenant '{}': {error}",
                    unique_value.field_logical_name, entity_logical_name, tenant_id
                )));
            }
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record update transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        RuntimeRecord::new(
            updated.id.to_string(),
            updated.entity_logical_name,
            updated.data,
        )
    }

    async fn list_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let limit = i64::try_from(query.limit).map_err(|error| {
            AppError::Validation(format!("invalid runtime record list limit: {error}"))
        })?;
        let offset = i64::try_from(query.offset).map_err(|error| {
            AppError::Validation(format!("invalid runtime record list offset: {error}"))
        })?;

        let rows = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            SELECT id, entity_logical_name, data
            FROM runtime_records
            WHERE tenant_id = $1 AND entity_logical_name = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list runtime records for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| RuntimeRecord::new(row.id.to_string(), row.entity_logical_name, row.data))
            .collect()
    }

    async fn query_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let limit = i64::try_from(query.limit).map_err(|error| {
            AppError::Validation(format!("invalid runtime record query limit: {error}"))
        })?;
        let offset = i64::try_from(query.offset).map_err(|error| {
            AppError::Validation(format!("invalid runtime record query offset: {error}"))
        })?;

        let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
            "SELECT id, entity_logical_name, data FROM runtime_records WHERE tenant_id = ",
        );
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND entity_logical_name = ");
        builder.push_bind(entity_logical_name);

        for filter in query.filters {
            builder.push(" AND data -> ");
            builder.push_bind(filter.field_logical_name);
            builder.push(" = ");
            builder.push_bind(filter.field_value);
        }

        builder.push(" ORDER BY created_at DESC LIMIT ");
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let rows = builder
            .build_query_as::<RuntimeRecordRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to query runtime records for entity '{}' in tenant '{}': {error}",
                    entity_logical_name, tenant_id
                ))
            })?;

        rows.into_iter()
            .map(|row| RuntimeRecord::new(row.id.to_string(), row.entity_logical_name, row.data))
            .collect()
    }

    async fn find_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<Option<RuntimeRecord>> {
        let record_uuid = Uuid::parse_str(record_id).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime record id '{}': {error}",
                record_id
            ))
        })?;

        let row = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            SELECT id, entity_logical_name, data
            FROM runtime_records
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find runtime record '{}' for entity '{}' in tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?;

        row.map(|value| {
            RuntimeRecord::new(value.id.to_string(), value.entity_logical_name, value.data)
        })
        .transpose()
    }

    async fn delete_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        let record_uuid = Uuid::parse_str(record_id).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime record id '{}': {error}",
                record_id
            ))
        })?;

        let deleted = sqlx::query(
            r#"
            DELETE FROM runtime_records
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to delete runtime record '{}' for entity '{}' in tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?;

        if deleted.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            )));
        }

        Ok(())
    }

    async fn runtime_record_exists(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<bool> {
        let record_uuid = Uuid::parse_str(record_id).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime record id '{}': {error}",
                record_id
            ))
        })?;

        sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM runtime_records
                WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            )
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to check runtime record existence for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })
    }

    async fn has_relation_reference(
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

#[cfg(test)]
mod tests {
    use qryvanta_application::{
        MetadataRepository, RecordListQuery, RuntimeRecordFilter, RuntimeRecordQuery,
    };
    use qryvanta_core::{AppError, TenantId};
    use qryvanta_domain::{EntityDefinition, EntityFieldDefinition, FieldType};
    use serde_json::json;
    use sqlx::PgPool;
    use sqlx::migrate::Migrator;
    use sqlx::postgres::PgPoolOptions;

    use super::PostgresMetadataRepository;

    static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

    async fn test_pool() -> Option<PgPool> {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            return None;
        };

        let pool = match PgPoolOptions::new()
            .max_connections(2)
            .connect(database_url.as_str())
            .await
        {
            Ok(pool) => pool,
            Err(error) => panic!("failed to connect to DATABASE_URL in test: {error}"),
        };

        if let Err(error) = MIGRATOR.run(&pool).await {
            panic!("failed to run migrations for postgres metadata tests: {error}");
        }

        Some(pool)
    }

    async fn ensure_tenant(pool: &PgPool, tenant_id: TenantId, name: &str) {
        let insert = sqlx::query(
            r#"
            INSERT INTO tenants (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(name)
        .execute(pool)
        .await;

        assert!(insert.is_ok());
    }

    #[tokio::test]
    async fn runtime_record_queries_are_tenant_scoped() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let repository = PostgresMetadataRepository::new(pool.clone());
        let left_tenant = TenantId::new();
        let right_tenant = TenantId::new();

        ensure_tenant(&pool, left_tenant, "Left Tenant").await;
        ensure_tenant(&pool, right_tenant, "Right Tenant").await;

        let left_entity = EntityDefinition::new("contact", "Contact");
        assert!(left_entity.is_ok());
        let right_entity = EntityDefinition::new("contact", "Contact");
        assert!(right_entity.is_ok());

        assert!(
            repository
                .save_entity(left_tenant, left_entity.unwrap_or_else(|_| unreachable!()))
                .await
                .is_ok()
        );
        assert!(
            repository
                .save_entity(
                    right_tenant,
                    right_entity.unwrap_or_else(|_| unreachable!())
                )
                .await
                .is_ok()
        );

        let left_record = repository
            .create_runtime_record(left_tenant, "contact", json!({"name": "Alice"}), Vec::new())
            .await;
        assert!(left_record.is_ok());
        let left_record = left_record.unwrap_or_else(|_| unreachable!());

        let right_listed = repository
            .list_runtime_records(
                right_tenant,
                "contact",
                RecordListQuery {
                    limit: 50,
                    offset: 0,
                },
            )
            .await;
        assert!(right_listed.is_ok());
        assert!(right_listed.unwrap_or_default().is_empty());

        let right_queried = repository
            .query_runtime_records(
                right_tenant,
                "contact",
                RuntimeRecordQuery {
                    limit: 50,
                    offset: 0,
                    filters: vec![RuntimeRecordFilter {
                        field_logical_name: "name".to_owned(),
                        field_value: json!("Alice"),
                    }],
                },
            )
            .await;
        assert!(right_queried.is_ok());
        assert!(right_queried.unwrap_or_default().is_empty());

        let right_found = repository
            .find_runtime_record(right_tenant, "contact", left_record.record_id().as_str())
            .await;
        assert!(right_found.is_ok());
        assert!(right_found.unwrap_or_default().is_none());

        let right_exists = repository
            .runtime_record_exists(right_tenant, "contact", left_record.record_id().as_str())
            .await;
        assert!(right_exists.is_ok());
        assert!(!right_exists.unwrap_or(true));

        let right_delete = repository
            .delete_runtime_record(right_tenant, "contact", left_record.record_id().as_str())
            .await;
        assert!(matches!(right_delete, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn query_runtime_records_filters_and_paginates() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let repository = PostgresMetadataRepository::new(pool.clone());
        let tenant_id = TenantId::new();
        ensure_tenant(&pool, tenant_id, "Query Tenant").await;

        let entity = EntityDefinition::new("contact", "Contact");
        assert!(entity.is_ok());
        assert!(
            repository
                .save_entity(tenant_id, entity.unwrap_or_else(|_| unreachable!()))
                .await
                .is_ok()
        );

        assert!(
            repository
                .create_runtime_record(
                    tenant_id,
                    "contact",
                    json!({"name": "Alice", "active": true}),
                    Vec::new(),
                )
                .await
                .is_ok()
        );
        assert!(
            repository
                .create_runtime_record(
                    tenant_id,
                    "contact",
                    json!({"name": "Bob", "active": false}),
                    Vec::new(),
                )
                .await
                .is_ok()
        );
        assert!(
            repository
                .create_runtime_record(
                    tenant_id,
                    "contact",
                    json!({"name": "Carol", "active": true}),
                    Vec::new(),
                )
                .await
                .is_ok()
        );

        let queried = repository
            .query_runtime_records(
                tenant_id,
                "contact",
                RuntimeRecordQuery {
                    limit: 1,
                    offset: 1,
                    filters: vec![RuntimeRecordFilter {
                        field_logical_name: "active".to_owned(),
                        field_value: json!(true),
                    }],
                },
            )
            .await;
        assert!(queried.is_ok());
        let queried = queried.unwrap_or_default();

        assert_eq!(queried.len(), 1);
        assert_eq!(
            queried[0]
                .data()
                .as_object()
                .and_then(|value| value.get("active")),
            Some(&json!(true))
        );
    }

    #[tokio::test]
    async fn relation_reference_check_does_not_leak_across_tenants() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let repository = PostgresMetadataRepository::new(pool.clone());
        let left_tenant = TenantId::new();
        let right_tenant = TenantId::new();

        ensure_tenant(&pool, left_tenant, "Left Tenant").await;
        ensure_tenant(&pool, right_tenant, "Right Tenant").await;

        let left_contact =
            EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
        let left_deal = EntityDefinition::new("deal", "Deal").unwrap_or_else(|_| unreachable!());
        let right_contact =
            EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
        let right_deal = EntityDefinition::new("deal", "Deal").unwrap_or_else(|_| unreachable!());

        assert!(
            repository
                .save_entity(left_tenant, left_contact)
                .await
                .is_ok()
        );
        assert!(
            repository
                .save_entity(left_tenant, left_deal.clone())
                .await
                .is_ok()
        );
        assert!(
            repository
                .save_entity(right_tenant, right_contact)
                .await
                .is_ok()
        );
        assert!(
            repository
                .save_entity(right_tenant, right_deal.clone())
                .await
                .is_ok()
        );

        let left_relation_field = EntityFieldDefinition::new(
            "deal",
            "owner_contact_id",
            "Owner",
            FieldType::Relation,
            true,
            false,
            None,
            Some("contact".to_owned()),
        )
        .unwrap_or_else(|_| unreachable!());
        let right_relation_field = EntityFieldDefinition::new(
            "deal",
            "owner_contact_id",
            "Owner",
            FieldType::Relation,
            true,
            false,
            None,
            Some("contact".to_owned()),
        )
        .unwrap_or_else(|_| unreachable!());

        assert!(
            repository
                .save_field(left_tenant, left_relation_field)
                .await
                .is_ok()
        );
        assert!(
            repository
                .save_field(right_tenant, right_relation_field)
                .await
                .is_ok()
        );

        let left_deal_fields = repository.list_fields(left_tenant, "deal").await;
        assert!(left_deal_fields.is_ok());
        assert!(
            repository
                .publish_entity_schema(
                    left_tenant,
                    left_deal,
                    left_deal_fields.unwrap_or_default(),
                    "alice",
                )
                .await
                .is_ok()
        );

        let right_deal_fields = repository.list_fields(right_tenant, "deal").await;
        assert!(right_deal_fields.is_ok());
        assert!(
            repository
                .publish_entity_schema(
                    right_tenant,
                    right_deal,
                    right_deal_fields.unwrap_or_default(),
                    "alice",
                )
                .await
                .is_ok()
        );

        let left_contact_record = repository
            .create_runtime_record(left_tenant, "contact", json!({"name": "Alice"}), Vec::new())
            .await;
        assert!(left_contact_record.is_ok());
        let left_contact_record = left_contact_record.unwrap_or_else(|_| unreachable!());

        assert!(
            repository
                .create_runtime_record(
                    right_tenant,
                    "deal",
                    json!({"owner_contact_id": left_contact_record.record_id().as_str()}),
                    Vec::new(),
                )
                .await
                .is_ok()
        );

        let cross_tenant_reference = repository
            .has_relation_reference(
                left_tenant,
                "contact",
                left_contact_record.record_id().as_str(),
            )
            .await;
        assert!(cross_tenant_reference.is_ok());
        assert!(!cross_tenant_reference.unwrap_or(true));

        assert!(
            repository
                .create_runtime_record(
                    left_tenant,
                    "deal",
                    json!({"owner_contact_id": left_contact_record.record_id().as_str()}),
                    Vec::new(),
                )
                .await
                .is_ok()
        );

        let in_tenant_reference = repository
            .has_relation_reference(
                left_tenant,
                "contact",
                left_contact_record.record_id().as_str(),
            )
            .await;
        assert!(in_tenant_reference.is_ok());
        assert!(in_tenant_reference.unwrap_or(false));
    }
}
