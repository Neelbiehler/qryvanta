use async_trait::async_trait;
use qryvanta_application::TenantRepository;
use qryvanta_core::{AppError, AppResult, TenantId};
use sqlx::PgPool;

/// PostgreSQL-backed tenant membership repository.
#[derive(Clone)]
pub struct PostgresTenantRepository {
    pool: PgPool,
}

impl PostgresTenantRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>> {
        let tenant_id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            SELECT tenant_id
            FROM tenant_memberships
            WHERE subject = $1
            LIMIT 1
            "#,
        )
        .bind(subject)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to resolve tenant membership: {error}"))
        })?;

        Ok(tenant_id.map(TenantId::from_uuid))
    }

    async fn create_membership(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tenant_memberships (tenant_id, subject, display_name, email)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (tenant_id, subject) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(display_name)
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to create membership: {error}")))?;

        Ok(())
    }
}
