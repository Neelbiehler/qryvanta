//! Auth token management for password resets, email verification, and invites.
//!
//! Tokens are cryptographically random, stored as SHA-256 hashes, single-use,
//! and time-limited per OWASP Forgot Password Cheat Sheet.

use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppError, AppResult};
use qryvanta_domain::{AuthTokenType, EmailAddress, UserId};

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

    /// Issues a password reset token and sends the reset email.
    ///
    /// Always returns `Ok(())` regardless of whether the email exists,
    /// per OWASP Forgot Password: "If that email is in our system, we will
    /// send you an email to reset your password."
    pub async fn request_password_reset(
        &self,
        email: &str,
        user_id: Option<UserId>,
    ) -> AppResult<()> {
        let Ok(canonical_email) = EmailAddress::new(email) else {
            // Keep generic success response semantics for invalid inputs.
            return Ok(());
        };

        // Rate limit: max 3 reset requests per email per hour.
        let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);
        let recent_count = self
            .token_repository
            .count_recent_tokens(
                canonical_email.as_str(),
                AuthTokenType::PasswordReset,
                one_hour_ago,
            )
            .await?;

        if recent_count >= 3 {
            // Silently succeed to prevent enumeration.
            return Ok(());
        }

        let Some(uid) = user_id else {
            // User not found -- silently succeed.
            return Ok(());
        };

        // Invalidate any existing reset tokens for this user.
        self.token_repository
            .invalidate_tokens_for_user(uid, AuthTokenType::PasswordReset)
            .await?;

        let (raw_token, token_hash) = generate_token()?;

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        self.token_repository
            .create_token(
                Some(uid),
                canonical_email.as_str(),
                &token_hash,
                AuthTokenType::PasswordReset,
                expires_at,
                None,
            )
            .await?;

        let reset_url = format!("{}/reset-password?token={}", self.frontend_url, raw_token);

        let subject = "Reset your Qryvanta password";
        let text_body = format!(
            "You requested a password reset.\n\n\
             Click the link below to set a new password:\n{reset_url}\n\n\
             This link expires in 1 hour.\n\n\
             If you did not request this, you can safely ignore this email."
        );

        self.email_service
            .send_email(canonical_email.as_str(), subject, &text_body, None)
            .await?;

        Ok(())
    }

    /// Issues an email verification token and sends the verification email.
    pub async fn send_email_verification(&self, user_id: UserId, email: &str) -> AppResult<()> {
        let canonical_email = EmailAddress::new(email)?;

        // Invalidate previous verification tokens.
        self.token_repository
            .invalidate_tokens_for_user(user_id, AuthTokenType::EmailVerification)
            .await?;

        let (raw_token, token_hash) = generate_token()?;

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        self.token_repository
            .create_token(
                Some(user_id),
                canonical_email.as_str(),
                &token_hash,
                AuthTokenType::EmailVerification,
                expires_at,
                None,
            )
            .await?;

        let verify_url = format!("{}/verify-email?token={}", self.frontend_url, raw_token);

        let subject = "Verify your Qryvanta email address";
        let text_body = format!(
            "Welcome to Qryvanta!\n\n\
             Please verify your email address by clicking the link below:\n{verify_url}\n\n\
             This link expires in 24 hours."
        );

        self.email_service
            .send_email(canonical_email.as_str(), subject, &text_body, None)
            .await?;

        Ok(())
    }

    /// Issues an invite token and sends the invitation email.
    pub async fn send_invite(
        &self,
        email: &str,
        inviter_name: &str,
        tenant_name: &str,
        metadata: &serde_json::Value,
    ) -> AppResult<()> {
        let canonical_email = EmailAddress::new(email)?;

        let (raw_token, token_hash) = generate_token()?;

        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
        self.token_repository
            .create_token(
                None,
                canonical_email.as_str(),
                &token_hash,
                AuthTokenType::Invite,
                expires_at,
                Some(metadata),
            )
            .await?;

        let invite_url = format!("{}/accept-invite?token={}", self.frontend_url, raw_token);

        let subject = format!("{inviter_name} invited you to {tenant_name} on Qryvanta");
        let text_body = format!(
            "{inviter_name} has invited you to join {tenant_name} on Qryvanta.\n\n\
             Click the link below to accept the invitation:\n{invite_url}\n\n\
             This link expires in 7 days."
        );

        self.email_service
            .send_email(canonical_email.as_str(), &subject, &text_body, None)
            .await?;

        Ok(())
    }

    /// Atomically validates and consumes a token.
    pub async fn consume_valid_token(
        &self,
        raw_token: &str,
        token_type: AuthTokenType,
    ) -> AppResult<AuthTokenRecord> {
        let token_hash = hash_token(raw_token);

        let record = self
            .token_repository
            .consume_valid_token(&token_hash, token_type)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid or expired token".to_owned()))?;

        Ok(record)
    }

    /// Returns a reference to the token repository.
    #[must_use]
    pub fn token_repository(&self) -> &Arc<dyn AuthTokenRepository> {
        &self.token_repository
    }
}

/// Generates a cryptographically random token and its SHA-256 hash.
///
/// Returns `(raw_token_hex, sha256_hash_hex)`.
fn generate_token() -> AppResult<(String, String)> {
    use std::fmt::Write;

    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| AppError::Internal(format!("failed to generate auth token: {error}")))?;

    let raw_token = bytes
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        });

    let hash = hash_token(&raw_token);
    Ok((raw_token, hash))
}

/// Computes the SHA-256 hash of a token string for storage.
fn hash_token(raw_token: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;

    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let result = hasher.finalize();

    result
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use qryvanta_core::AppResult;
    use qryvanta_domain::AuthTokenType;

    use super::{AuthTokenRecord, AuthTokenRepository, AuthTokenService, EmailService};

    #[derive(Default)]
    struct TestTokenRepo {
        created: Mutex<Vec<(String, AuthTokenType, Option<serde_json::Value>)>>,
    }

    #[async_trait]
    impl AuthTokenRepository for TestTokenRepo {
        async fn create_token(
            &self,
            _user_id: Option<qryvanta_domain::UserId>,
            email: &str,
            _token_hash: &str,
            token_type: AuthTokenType,
            _expires_at: chrono::DateTime<chrono::Utc>,
            metadata: Option<&serde_json::Value>,
        ) -> AppResult<uuid::Uuid> {
            self.created
                .lock()
                .map_err(|error| {
                    qryvanta_core::AppError::Internal(format!("failed to lock repo state: {error}"))
                })?
                .push((email.to_owned(), token_type, metadata.cloned()));
            Ok(uuid::Uuid::new_v4())
        }

        async fn consume_valid_token(
            &self,
            _token_hash: &str,
            _token_type: AuthTokenType,
        ) -> AppResult<Option<AuthTokenRecord>> {
            Ok(None)
        }

        async fn invalidate_tokens_for_user(
            &self,
            _user_id: qryvanta_domain::UserId,
            _token_type: AuthTokenType,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn count_recent_tokens(
            &self,
            _email: &str,
            _token_type: AuthTokenType,
            _since: chrono::DateTime<chrono::Utc>,
        ) -> AppResult<i64> {
            Ok(0)
        }
    }

    #[derive(Default)]
    struct TestEmailService {
        sent: Mutex<Vec<(String, String)>>,
    }

    #[async_trait]
    impl EmailService for TestEmailService {
        async fn send_email(
            &self,
            to: &str,
            subject: &str,
            _text_body: &str,
            _html_body: Option<&str>,
        ) -> AppResult<()> {
            self.sent
                .lock()
                .map_err(|error| {
                    qryvanta_core::AppError::Internal(format!(
                        "failed to lock email service state: {error}"
                    ))
                })?
                .push((to.to_owned(), subject.to_owned()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn send_invite_persists_invite_token_and_sends_email() {
        let repo = Arc::new(TestTokenRepo::default());
        let email = Arc::new(TestEmailService::default());

        let service = AuthTokenService::new(
            repo.clone(),
            email.clone(),
            "http://localhost:3000".to_owned(),
        );

        let metadata = serde_json::json!({"tenant_id": "tenant-1", "invited_by": "alice"});
        let result = service
            .send_invite("new.user@example.com", "Alice", "Acme Workspace", &metadata)
            .await;

        assert!(result.is_ok());

        let created = repo
            .created
            .lock()
            .ok()
            .map(|guard| guard.len())
            .unwrap_or(0);
        assert_eq!(created, 1);

        let sent = email.sent.lock().ok().map(|guard| guard.len()).unwrap_or(0);
        assert_eq!(sent, 1);
    }
}
