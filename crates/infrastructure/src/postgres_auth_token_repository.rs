//! PostgreSQL-backed auth token repository.

use async_trait::async_trait;
use sqlx::PgPool;

use qryvanta_application::{AuthTokenRecord, AuthTokenRepository};
use qryvanta_core::{AppError, AppResult};
use qryvanta_domain::{AuthTokenType, UserId};

/// PostgreSQL implementation of the auth token repository port.
#[derive(Clone)]
pub struct PostgresAuthTokenRepository {
    pool: PgPool,
}

impl PostgresAuthTokenRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthTokenRepository for PostgresAuthTokenRepository {
    async fn create_token(
        &self,
        user_id: Option<UserId>,
        email: &str,
        token_hash: &str,
        token_type: AuthTokenType,
        expires_at: chrono::DateTime<chrono::Utc>,
        metadata: Option<&serde_json::Value>,
    ) -> AppResult<uuid::Uuid> {
        let id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            INSERT INTO auth_tokens (user_id, email, token_hash, token_type, expires_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(user_id.map(|uid| uid.as_uuid()))
        .bind(email)
        .bind(token_hash)
        .bind(token_type.as_str())
        .bind(expires_at)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to create auth token: {error}")))?;

        Ok(id)
    }

    async fn consume_valid_token(
        &self,
        token_hash: &str,
        token_type: AuthTokenType,
    ) -> AppResult<Option<AuthTokenRecord>> {
        let row = sqlx::query_as::<_, TokenRow>(
            r#"
            UPDATE auth_tokens
            SET used_at = now()
            WHERE token_hash = $1
              AND token_type = $2
              AND used_at IS NULL
              AND expires_at > now()
            RETURNING id, user_id, email, token_hash, token_type, expires_at, used_at, metadata
            "#,
        )
        .bind(token_hash)
        .bind(token_type.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to consume auth token: {error}")))?;

        Ok(row.map(AuthTokenRecord::from))
    }

    async fn invalidate_tokens_for_user(
        &self,
        user_id: UserId,
        token_type: AuthTokenType,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE auth_tokens
            SET used_at = now()
            WHERE user_id = $1
              AND token_type = $2
              AND used_at IS NULL
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(token_type.as_str())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to invalidate tokens: {error}")))?;

        Ok(())
    }

    async fn count_recent_tokens(
        &self,
        email: &str,
        token_type: AuthTokenType,
        since: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM auth_tokens
            WHERE LOWER(email) = LOWER($1)
              AND token_type = $2
              AND created_at >= $3
            "#,
        )
        .bind(email)
        .bind(token_type.as_str())
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to count recent tokens: {error}")))?;

        Ok(count)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TokenRow {
    id: uuid::Uuid,
    user_id: Option<uuid::Uuid>,
    email: String,
    token_hash: String,
    token_type: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    used_at: Option<chrono::DateTime<chrono::Utc>>,
    metadata: Option<serde_json::Value>,
}

impl From<TokenRow> for AuthTokenRecord {
    fn from(row: TokenRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id.map(UserId::from_uuid),
            email: row.email,
            token_hash: row.token_hash,
            token_type: row.token_type,
            expires_at: row.expires_at,
            used_at: row.used_at,
            metadata: row.metadata,
        }
    }
}
