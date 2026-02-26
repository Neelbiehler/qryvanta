//! PostgreSQL-backed auth token repository.

use async_trait::async_trait;
use sqlx::PgPool;

use qryvanta_application::{AuthTokenRecord, AuthTokenRepository};
use qryvanta_core::AppResult;
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

mod consume;
mod invalidate;
mod issue;
mod rate_limit;

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
        self.create_token_impl(user_id, email, token_hash, token_type, expires_at, metadata)
            .await
    }

    async fn consume_valid_token(
        &self,
        token_hash: &str,
        token_type: AuthTokenType,
    ) -> AppResult<Option<AuthTokenRecord>> {
        self.consume_valid_token_impl(token_hash, token_type).await
    }

    async fn invalidate_tokens_for_user(
        &self,
        user_id: UserId,
        token_type: AuthTokenType,
    ) -> AppResult<()> {
        self.invalidate_tokens_for_user_impl(user_id, token_type)
            .await
    }

    async fn count_recent_tokens(
        &self,
        email: &str,
        token_type: AuthTokenType,
        since: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<i64> {
        self.count_recent_tokens_impl(email, token_type, since)
            .await
    }
}
