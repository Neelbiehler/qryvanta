//! User management ports and application service.
//!
//! Owns user lifecycle operations: registration, authentication,
//! password changes, and account lockout. Follows OWASP guidelines for
//! generic error messages and constant-time responses.

use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{EmailAddress, RegistrationMode, UserId, validate_password};

use crate::{AuthEvent, AuthEventService, TenantRepository};

// ---------------------------------------------------------------------------
// Ports
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Authentication outcome
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

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

    /// Registers a new user with email and password.
    ///
    /// Only allowed when the tenant registration mode is `Open` or when
    /// called from an invite acceptance flow (caller is responsible for
    /// that check).
    pub async fn register(&self, params: RegisterParams) -> AppResult<UserId> {
        if params.registration_mode == RegistrationMode::InviteOnly {
            return Err(AppError::Forbidden(
                "registration is currently invite-only".to_owned(),
            ));
        }

        let email_address = EmailAddress::new(&params.email)?;
        validate_password(&params.password, false)?;

        // Check for existing user -- always hash to prevent timing attacks.
        let existing = self
            .user_repository
            .find_by_email(email_address.as_str())
            .await?;

        if existing.is_some() {
            // OWASP: do not reveal that the account exists.
            // Still hash the password to prevent timing side-channels.
            let _ = self.password_hasher.hash_password(&params.password);
            return Err(AppError::Conflict(
                "a link to activate your account has been emailed to the address provided"
                    .to_owned(),
            ));
        }

        let password_hash = self.password_hasher.hash_password(&params.password)?;
        let user_id = self
            .user_repository
            .create(email_address.as_str(), Some(&password_hash), false)
            .await?;

        // Create tenant membership for the new user.
        let tenant_id = self
            .tenant_repository
            .ensure_membership_for_subject(
                &user_id.to_string(),
                &params.display_name,
                Some(email_address.as_str()),
                params.preferred_tenant_id,
            )
            .await?;

        // Link membership to user_id -- the tenant repository uses subject strings,
        // so we pass user_id as the subject for new users.
        let _ = tenant_id;

        self.auth_event_service
            .record_event(AuthEvent {
                subject: Some(user_id.to_string()),
                event_type: "registration".to_owned(),
                outcome: "success".to_owned(),
                ip_address: params.ip_address,
                user_agent: params.user_agent,
            })
            .await?;

        Ok(user_id)
    }

    /// Authenticates a user with email and password.
    ///
    /// Returns `AuthOutcome::Failed` with a generic message for any failure
    /// (unknown email, wrong password, locked account) to prevent enumeration.
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthOutcome> {
        let user = self.user_repository.find_by_email(email).await?;

        let Some(user) = user else {
            // OWASP: always hash to prevent timing attacks even when user not found.
            let _ = self.password_hasher.hash_password(password);
            return Ok(AuthOutcome::Failed);
        };

        // Check account lockout.
        if let Some(locked_until) = user.locked_until
            && chrono::Utc::now() < locked_until
        {
            // Still locked -- don't reveal this; just say failed.
            let _ = self.password_hasher.hash_password(password);

            self.auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.id.to_string()),
                    event_type: "login_attempt".to_owned(),
                    outcome: "account_locked".to_owned(),
                    ip_address,
                    user_agent,
                })
                .await?;

            return Ok(AuthOutcome::Failed);
        }

        let Some(ref stored_hash) = user.password_hash else {
            // Passkey-only user trying password login -- fail generically.
            let _ = self.password_hasher.hash_password(password);
            return Ok(AuthOutcome::Failed);
        };

        let password_valid = self
            .password_hasher
            .verify_password(password, stored_hash)?;

        if !password_valid {
            self.user_repository.record_failed_login(user.id).await?;

            self.auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.id.to_string()),
                    event_type: "login_attempt".to_owned(),
                    outcome: "invalid_password".to_owned(),
                    ip_address,
                    user_agent,
                })
                .await?;

            return Ok(AuthOutcome::Failed);
        }

        // Password correct -- reset failed login counter.
        self.user_repository.reset_failed_logins(user.id).await?;

        // Check if MFA is required.
        if user.totp_enabled {
            self.auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.id.to_string()),
                    event_type: "login_attempt".to_owned(),
                    outcome: "mfa_required".to_owned(),
                    ip_address,
                    user_agent,
                })
                .await?;

            return Ok(AuthOutcome::MfaRequired { user_id: user.id });
        }

        self.auth_event_service
            .record_event(AuthEvent {
                subject: Some(user.id.to_string()),
                event_type: "login_attempt".to_owned(),
                outcome: "success".to_owned(),
                ip_address,
                user_agent,
            })
            .await?;

        Ok(AuthOutcome::Authenticated(user))
    }

    /// Changes the password for an authenticated user.
    ///
    /// Requires the current password for verification (OWASP Authentication:
    /// change password feature).
    pub async fn change_password(
        &self,
        user_id: UserId,
        current_password: &str,
        new_password: &str,
    ) -> AppResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        let Some(ref stored_hash) = user.password_hash else {
            return Err(AppError::Validation(
                "no password is set on this account".to_owned(),
            ));
        };

        let current_valid = self
            .password_hasher
            .verify_password(current_password, stored_hash)?;

        if !current_valid {
            return Err(AppError::Unauthorized(
                "current password is incorrect".to_owned(),
            ));
        }

        validate_password(new_password, user.totp_enabled)?;

        let new_hash = self.password_hasher.hash_password(new_password)?;
        self.user_repository
            .update_password(user_id, &new_hash)
            .await
    }

    /// Returns a user record by ID, if it exists.
    pub async fn find_by_id(&self, user_id: UserId) -> AppResult<Option<UserRecord>> {
        self.user_repository.find_by_id(user_id).await
    }

    /// Returns a user record by email, if it exists.
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<UserRecord>> {
        self.user_repository.find_by_email(email).await
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
