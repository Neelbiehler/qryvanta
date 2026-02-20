use async_trait::async_trait;
use qryvanta_application::MetadataRepository;
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::EntityDefinition;
use sqlx::{FromRow, PgPool};

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
}
