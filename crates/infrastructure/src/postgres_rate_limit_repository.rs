//! PostgreSQL-backed rate limit repository using the `auth_rate_limits` table.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use qryvanta_application::{AttemptInfo, RateLimitRepository};
use qryvanta_core::{AppError, AppResult};

/// PostgreSQL implementation of the rate limit repository port.
#[derive(Clone)]
pub struct PostgresRateLimitRepository {
    pool: PgPool,
}

impl PostgresRateLimitRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RateLimitRepository for PostgresRateLimitRepository {
    async fn record_attempt(
        &self,
        key: &str,
        window_duration_seconds: i64,
    ) -> AppResult<AttemptInfo> {
        // UPSERT: insert a new row or increment the counter.
        // If the existing window has expired, reset the counter and window start.
        let row = sqlx::query_as::<_, AttemptRow>(
            r#"
            INSERT INTO auth_rate_limits (key, window_started_at, attempt_count)
            VALUES ($1, now(), 1)
            ON CONFLICT (key) DO UPDATE
            SET
                attempt_count = CASE
                    WHEN auth_rate_limits.window_started_at + make_interval(secs => $2::float8) < now()
                    THEN 1
                    ELSE auth_rate_limits.attempt_count + 1
                END,
                window_started_at = CASE
                    WHEN auth_rate_limits.window_started_at + make_interval(secs => $2::float8) < now()
                    THEN now()
                    ELSE auth_rate_limits.window_started_at
                END
            RETURNING attempt_count, window_started_at
            "#,
        )
        .bind(key)
        .bind(window_duration_seconds as f64)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to record rate limit attempt: {error}"))
        })?;

        Ok(AttemptInfo {
            attempt_count: row.attempt_count,
            window_started_at: row.window_started_at,
        })
    }

    async fn cleanup_expired(&self, before: DateTime<Utc>) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM auth_rate_limits
            WHERE window_started_at < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to cleanup expired rate limits: {error}"))
        })?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AttemptRow {
    attempt_count: i32,
    window_started_at: DateTime<Utc>,
}
