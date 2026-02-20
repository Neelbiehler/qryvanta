use async_trait::async_trait;
use sqlx::PgPool;

use qryvanta_application::{AuditEvent, AuditRepository};
use qryvanta_core::{AppError, AppResult};

/// PostgreSQL-backed append-only audit repository.
#[derive(Clone)]
pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_log_entries (
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(event.tenant_id.as_uuid())
        .bind(event.subject)
        .bind(event.action.as_str())
        .bind(event.resource_type)
        .bind(event.resource_id)
        .bind(event.detail)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to append audit event: {error}")))?;

        Ok(())
    }
}
