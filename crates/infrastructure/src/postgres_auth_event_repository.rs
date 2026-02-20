use async_trait::async_trait;
use sqlx::PgPool;

use qryvanta_application::{AuthEvent, AuthEventRepository};
use qryvanta_core::{AppError, AppResult};

/// PostgreSQL-backed repository for authentication events.
#[derive(Clone)]
pub struct PostgresAuthEventRepository {
    pool: PgPool,
}

impl PostgresAuthEventRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthEventRepository for PostgresAuthEventRepository {
    async fn append_event(&self, event: AuthEvent) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO auth_events (
                subject,
                event_type,
                outcome,
                ip_address,
                user_agent
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(event.subject)
        .bind(event.event_type)
        .bind(event.outcome)
        .bind(event.ip_address)
        .bind(event.user_agent)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to append auth event: {error}")))?;

        Ok(())
    }
}
