//! Auth token management for password resets, email verification, and invites.
//!
//! Tokens are cryptographically random, stored as SHA-256 hashes, single-use,
//! and time-limited per OWASP Forgot Password Cheat Sheet.

use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::AppResult;
use qryvanta_domain::{AuthTokenType, UserId};

/// Auth token record as persisted in the database.
#[derive(Debug, Clone)]
pub struct AuthTokenRecord {
    /// Token identifier.
    pub id: uuid::Uuid,
    /// User ID the token belongs to, if applicable.
    pub user_id: Option<UserId>,
    /// Email address the token was sent to.
    pub email: String,
    /// SHA-256 hash of the token value.
    pub token_hash: String,
    /// Type discriminator.
    pub token_type: String,
    /// Expiration timestamp.
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// When the token was consumed, if ever.
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Optional metadata (e.g. invite role, tenant ID).
    pub metadata: Option<serde_json::Value>,
}

/// Repository port for auth token persistence.
#[async_trait]
pub trait AuthTokenRepository: Send + Sync {
    /// Stores a new auth token.
    async fn create_token(
        &self,
        user_id: Option<UserId>,
        email: &str,
        token_hash: &str,
        token_type: AuthTokenType,
        expires_at: chrono::DateTime<chrono::Utc>,
        metadata: Option<&serde_json::Value>,
    ) -> AppResult<uuid::Uuid>;

    /// Atomically consumes a valid token by its hash and returns the record.
    ///
    /// Consumption succeeds only when the token is unexpired and unused.
    /// When consumed, `used_at` is set in the same database statement to
    /// prevent replay races.
    async fn consume_valid_token(
        &self,
        token_hash: &str,
        token_type: AuthTokenType,
    ) -> AppResult<Option<AuthTokenRecord>>;

    /// Invalidates all unused tokens of a given type for a user.
    async fn invalidate_tokens_for_user(
        &self,
        user_id: UserId,
        token_type: AuthTokenType,
    ) -> AppResult<()>;

    /// Counts tokens created in a time window for rate limiting.
    async fn count_recent_tokens(
        &self,
        email: &str,
        token_type: AuthTokenType,
        since: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<i64>;
}

/// Port for sending emails. Infrastructure provides SMTP or console implementations.
#[async_trait]
pub trait EmailService: Send + Sync {
    /// Sends a plain-text or HTML email.
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        html_body: Option<&str>,
    ) -> AppResult<()>;
}

/// Application service for managing auth tokens and related email flows.
#[derive(Clone)]
pub struct AuthTokenService {
    token_repository: Arc<dyn AuthTokenRepository>,
    email_service: Arc<dyn EmailService>,
    frontend_url: String,
}

impl AuthTokenService {
    /// Creates a new auth token service.
    #[must_use]
    pub fn new(
        token_repository: Arc<dyn AuthTokenRepository>,
        email_service: Arc<dyn EmailService>,
        frontend_url: String,
    ) -> Self {
        Self {
            token_repository,
            email_service,
            frontend_url,
        }
    }

    /// Returns a reference to the token repository.
    #[must_use]
    pub fn token_repository(&self) -> &Arc<dyn AuthTokenRepository> {
        &self.token_repository
    }
}

mod consume;
mod email_verification;
mod invite;
mod password_reset;
mod token_crypto;

#[cfg(test)]
mod tests;
