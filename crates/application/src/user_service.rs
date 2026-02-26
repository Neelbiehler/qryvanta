//! User management ports and application service.
//!
//! Owns user lifecycle operations: registration, authentication,
//! password changes, and account lockout. Follows OWASP guidelines for
//! generic error messages and constant-time responses.

use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{RegistrationMode, UserId};

use crate::{AuthEventService, TenantRepository};

/// User record returned by repository queries.
#[derive(Debug, Clone)]
pub struct UserRecord {
    /// Unique user identifier.
    pub id: UserId,
    /// Canonical email address.
    pub email: String,
    /// Whether the email address has been verified.
    pub email_verified: bool,
    /// Argon2id password hash, or `None` for passkey-only accounts.
    pub password_hash: Option<String>,
    /// Whether TOTP MFA is enabled.
    pub totp_enabled: bool,
    /// Encrypted TOTP secret, if enrolled.
    pub totp_secret_enc: Option<Vec<u8>>,
    /// Hashed recovery codes as JSON array, if enrolled.
    pub recovery_codes_hash: Option<serde_json::Value>,
    /// Number of consecutive failed login attempts.
    pub failed_login_count: i32,
    /// Account is locked until this time, if set.
    pub locked_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// Repository port for user persistence.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Finds a user by email (case-insensitive).
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserRecord>>;

    /// Finds a user by their unique identifier.
    async fn find_by_id(&self, user_id: UserId) -> AppResult<Option<UserRecord>>;

    /// Creates a new user record. Returns the assigned user ID.
    async fn create(
        &self,
        email: &str,
        password_hash: Option<&str>,
        email_verified: bool,
    ) -> AppResult<UserId>;

    /// Updates the password hash for a user.
    async fn update_password(&self, user_id: UserId, password_hash: &str) -> AppResult<()>;

    /// Increments the failed login counter and optionally locks the account.
    async fn record_failed_login(&self, user_id: UserId) -> AppResult<()>;

    /// Resets the failed login counter and removes any account lock.
    async fn reset_failed_logins(&self, user_id: UserId) -> AppResult<()>;

    /// Marks the user's email as verified.
    async fn mark_email_verified(&self, user_id: UserId) -> AppResult<()>;

    /// Updates the user's display name in their tenant membership.
    async fn update_display_name(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        display_name: &str,
    ) -> AppResult<()>;

    /// Updates the user's email address.
    async fn update_email(&self, user_id: UserId, new_email: &str) -> AppResult<()>;

    /// Stores encrypted TOTP secret and hashed recovery codes.
    async fn enable_totp(
        &self,
        user_id: UserId,
        totp_secret_enc: &[u8],
        recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()>;

    /// Disables TOTP and clears recovery codes.
    async fn disable_totp(&self, user_id: UserId) -> AppResult<()>;

    /// Updates the hashed recovery codes.
    async fn update_recovery_codes(
        &self,
        user_id: UserId,
        recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()>;

    /// Finds a user by their legacy subject string (for migration compatibility).
    async fn find_by_subject(&self, subject: &str) -> AppResult<Option<UserRecord>>;
}

/// Port for password hashing operations. Keeps domain/application free of
/// direct cryptographic library coupling.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hashes a plaintext password using Argon2id.
    fn hash_password(&self, password: &str) -> AppResult<String>;

    /// Verifies a plaintext password against a stored hash.
    /// Must run in constant time regardless of validity.
    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
}

/// Result of a login attempt.
#[derive(Debug)]
pub enum AuthOutcome {
    /// Authentication succeeded. Session can be established.
    Authenticated(UserRecord),
    /// Password was correct but TOTP verification is required.
    MfaRequired {
        /// The user ID awaiting MFA.
        user_id: UserId,
    },
    /// Authentication failed. Generic message prevents enumeration.
    Failed,
}

/// Parameters for user registration.
pub struct RegisterParams {
    /// Email address for the new account.
    pub email: String,
    /// Plaintext password (validated against OWASP rules).
    pub password: String,
    /// Display name for tenant membership.
    pub display_name: String,
    /// Current registration mode for the tenant.
    pub registration_mode: RegistrationMode,
    /// Preferred tenant to join (if any).
    pub preferred_tenant_id: Option<TenantId>,
    /// IP address from the request (for audit logging).
    pub ip_address: Option<String>,
    /// User-Agent header from the request (for audit logging).
    pub user_agent: Option<String>,
}

/// Application service for user authentication and registration.
#[derive(Clone)]
pub struct UserService {
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
    tenant_repository: Arc<dyn TenantRepository>,
    auth_event_service: AuthEventService,
}

impl UserService {
    /// Creates a new user service.
    #[must_use]
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
        tenant_repository: Arc<dyn TenantRepository>,
        auth_event_service: AuthEventService,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
            tenant_repository,
            auth_event_service,
        }
    }

    /// Returns a reference to the password hasher for use by other services.
    #[must_use]
    pub fn password_hasher(&self) -> &Arc<dyn PasswordHasher> {
        &self.password_hasher
    }

    /// Returns a reference to the user repository for use by other services.
    #[must_use]
    pub fn user_repository(&self) -> &Arc<dyn UserRepository> {
        &self.user_repository
    }
}

mod login;
mod password;
mod registration;
mod retrieval;
