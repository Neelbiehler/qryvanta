//! MFA (TOTP) enrollment, verification, and recovery code management.
//!
//! Follows OWASP Multifactor Authentication Cheat Sheet:
//! - TOTP codes are 6-digit, 30-second window, +/-1 step tolerance.
//! - Recovery codes are single-use, stored hashed.
//! - Disabling MFA requires password re-authentication.

use std::sync::Arc;

use async_trait::async_trait;

use crate::user_service::{PasswordHasher, UserRepository};
use qryvanta_core::AppResult;

/// TOTP enrollment data returned to the user for QR code display.
#[derive(Debug, Clone)]
pub struct TotpEnrollment {
    /// Base32-encoded TOTP secret for manual entry.
    pub secret_base32: String,
    /// otpauth:// URI for QR code generation.
    pub otpauth_uri: String,
    /// Single-use recovery codes (plaintext, shown once).
    pub recovery_codes: Vec<String>,
}

/// Port for TOTP operations. Infrastructure provides the actual TOTP implementation.
#[async_trait]
pub trait TotpProvider: Send + Sync {
    /// Generates a new TOTP secret and returns (secret_bytes, base32_string, otpauth_uri).
    fn generate_secret(&self, email: &str) -> AppResult<(Vec<u8>, String, String)>;

    /// Verifies a TOTP code against a secret with +/-1 step tolerance.
    fn verify_code(&self, secret_bytes: &[u8], code: &str) -> AppResult<bool>;
}

/// Port for encrypting/decrypting TOTP secrets at rest.
#[async_trait]
pub trait SecretEncryptor: Send + Sync {
    /// Encrypts a TOTP secret for database storage.
    fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>>;

    /// Decrypts a stored TOTP secret.
    fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>>;
}

/// Application service for MFA operations.
#[derive(Clone)]
pub struct MfaService {
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
    totp_provider: Arc<dyn TotpProvider>,
    secret_encryptor: Arc<dyn SecretEncryptor>,
}

impl MfaService {
    /// Creates a new MFA service.
    #[must_use]
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
        totp_provider: Arc<dyn TotpProvider>,
        secret_encryptor: Arc<dyn SecretEncryptor>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
            totp_provider,
            secret_encryptor,
        }
    }
}

mod enrollment;
mod management;
mod recovery_codes;
mod verification;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::user_service::{PasswordHasher, UserRecord, UserRepository};
    use qryvanta_core::{AppError, AppResult, TenantId};
    use qryvanta_domain::UserId;

    use super::{MfaService, SecretEncryptor, TotpProvider};

    #[derive(Default)]
    struct FakeUserState {
        user: Option<UserRecord>,
    }

    #[derive(Clone)]
    struct FakeUserRepository {
        state: Arc<Mutex<FakeUserState>>,
    }

    impl FakeUserRepository {
        fn with_user(user: UserRecord) -> Self {
            Self {
                state: Arc::new(Mutex::new(FakeUserState { user: Some(user) })),
            }
        }

        fn snapshot(&self) -> UserRecord {
            self.state
                .lock()
                .unwrap_or_else(|_| unreachable!())
                .user
                .clone()
                .unwrap_or_else(|| unreachable!())
        }
    }

    #[async_trait]
    impl UserRepository for FakeUserRepository {
        async fn find_by_email(&self, _email: &str) -> AppResult<Option<UserRecord>> {
            Ok(self
                .state
                .lock()
                .unwrap_or_else(|_| unreachable!())
                .user
                .clone())
        }

        async fn find_by_id(&self, user_id: UserId) -> AppResult<Option<UserRecord>> {
            Ok(self
                .state
                .lock()
                .unwrap_or_else(|_| unreachable!())
                .user
                .clone()
                .filter(|user| user.id == user_id))
        }

        async fn create(
            &self,
            _email: &str,
            _password_hash: Option<&str>,
            _email_verified: bool,
        ) -> AppResult<UserId> {
            Err(AppError::Internal("unused in test".to_owned()))
        }

        async fn update_password(&self, _user_id: UserId, _password_hash: &str) -> AppResult<()> {
            Err(AppError::Internal("unused in test".to_owned()))
        }

        async fn revoke_sessions(&self, _user_id: UserId) -> AppResult<()> {
            Ok(())
        }

        async fn default_tenant_id(&self, _user_id: UserId) -> AppResult<Option<TenantId>> {
            Ok(None)
        }

        async fn set_default_tenant_id(
            &self,
            _user_id: UserId,
            _tenant_id: TenantId,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn record_failed_login(&self, _user_id: UserId) -> AppResult<()> {
            Err(AppError::Internal("unused in test".to_owned()))
        }

        async fn reset_failed_logins(&self, _user_id: UserId) -> AppResult<()> {
            Err(AppError::Internal("unused in test".to_owned()))
        }

        async fn mark_email_verified(&self, _user_id: UserId) -> AppResult<()> {
            Err(AppError::Internal("unused in test".to_owned()))
        }

        async fn update_display_name(
            &self,
            _user_id: UserId,
            _tenant_id: TenantId,
            _display_name: &str,
        ) -> AppResult<()> {
            Err(AppError::Internal("unused in test".to_owned()))
        }

        async fn update_email(&self, _user_id: UserId, _new_email: &str) -> AppResult<()> {
            Err(AppError::Internal("unused in test".to_owned()))
        }

        async fn enable_totp(
            &self,
            user_id: UserId,
            totp_secret_enc: &[u8],
            recovery_codes_hash: &serde_json::Value,
        ) -> AppResult<()> {
            let mut state = self.state.lock().unwrap_or_else(|_| unreachable!());
            let user = state
                .user
                .as_mut()
                .filter(|user| user.id == user_id)
                .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;
            user.totp_enabled = true;
            user.totp_secret_enc = Some(totp_secret_enc.to_vec());
            user.recovery_codes_hash = Some(recovery_codes_hash.clone());
            user.totp_pending_secret_enc = None;
            user.recovery_codes_pending_hash = None;
            Ok(())
        }

        async fn begin_totp_enrollment(
            &self,
            user_id: UserId,
            totp_secret_enc: &[u8],
            recovery_codes_hash: &serde_json::Value,
        ) -> AppResult<()> {
            let mut state = self.state.lock().unwrap_or_else(|_| unreachable!());
            let user = state
                .user
                .as_mut()
                .filter(|user| user.id == user_id)
                .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;
            user.totp_enabled = false;
            user.totp_pending_secret_enc = Some(totp_secret_enc.to_vec());
            user.recovery_codes_pending_hash = Some(recovery_codes_hash.clone());
            Ok(())
        }

        async fn confirm_totp_enrollment(&self, user_id: UserId) -> AppResult<()> {
            let mut state = self.state.lock().unwrap_or_else(|_| unreachable!());
            let user = state
                .user
                .as_mut()
                .filter(|user| user.id == user_id)
                .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;
            user.totp_enabled = true;
            user.totp_secret_enc = user.totp_pending_secret_enc.take();
            user.recovery_codes_hash = user.recovery_codes_pending_hash.take();
            Ok(())
        }

        async fn disable_totp(&self, user_id: UserId) -> AppResult<()> {
            let mut state = self.state.lock().unwrap_or_else(|_| unreachable!());
            let user = state
                .user
                .as_mut()
                .filter(|user| user.id == user_id)
                .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;
            user.totp_enabled = false;
            user.totp_secret_enc = None;
            user.recovery_codes_hash = None;
            user.totp_pending_secret_enc = None;
            user.recovery_codes_pending_hash = None;
            Ok(())
        }

        async fn update_recovery_codes(
            &self,
            user_id: UserId,
            recovery_codes_hash: &serde_json::Value,
        ) -> AppResult<()> {
            let mut state = self.state.lock().unwrap_or_else(|_| unreachable!());
            let user = state
                .user
                .as_mut()
                .filter(|user| user.id == user_id)
                .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;
            user.recovery_codes_hash = Some(recovery_codes_hash.clone());
            Ok(())
        }

        async fn find_by_subject(&self, _subject: &str) -> AppResult<Option<UserRecord>> {
            Ok(self
                .state
                .lock()
                .unwrap_or_else(|_| unreachable!())
                .user
                .clone())
        }
    }

    struct FakePasswordHasher;

    #[async_trait]
    impl PasswordHasher for FakePasswordHasher {
        fn hash_password(&self, password: &str) -> AppResult<String> {
            Ok(password.to_owned())
        }

        fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
            Ok(password == hash)
        }
    }

    struct FakeTotpProvider;

    #[async_trait]
    impl TotpProvider for FakeTotpProvider {
        fn generate_secret(&self, email: &str) -> AppResult<(Vec<u8>, String, String)> {
            Ok((
                format!("secret:{email}").into_bytes(),
                "BASE32SECRET".to_owned(),
                "otpauth://qryvanta/test".to_owned(),
            ))
        }

        fn verify_code(&self, _secret_bytes: &[u8], code: &str) -> AppResult<bool> {
            Ok(code == "123456")
        }
    }

    struct FakeSecretEncryptor;

    #[async_trait]
    impl SecretEncryptor for FakeSecretEncryptor {
        fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
            Ok(plaintext.to_vec())
        }

        fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>> {
            Ok(ciphertext.to_vec())
        }
    }

    fn build_service(repository: Arc<dyn UserRepository>) -> MfaService {
        MfaService::new(
            repository,
            Arc::new(FakePasswordHasher),
            Arc::new(FakeTotpProvider),
            Arc::new(FakeSecretEncryptor),
        )
    }

    fn sample_user() -> UserRecord {
        UserRecord {
            id: UserId::new(),
            email: "user@example.com".to_owned(),
            email_verified: true,
            password_hash: Some("password".to_owned()),
            totp_enabled: false,
            totp_secret_enc: None,
            recovery_codes_hash: None,
            totp_pending_secret_enc: None,
            recovery_codes_pending_hash: None,
            failed_login_count: 0,
            locked_until: None,
            password_changed_at: None,
            auth_sessions_revoked_after: None,
            default_tenant_id: None,
        }
    }

    #[tokio::test]
    async fn enrollment_stays_pending_until_confirmation() {
        let repository = FakeUserRepository::with_user(sample_user());
        let user_id = repository.snapshot().id;
        let service = build_service(Arc::new(repository.clone()));

        service
            .start_enrollment(user_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        let pending_user = repository.snapshot();
        assert!(!pending_user.totp_enabled);
        assert!(pending_user.totp_secret_enc.is_none());
        assert!(pending_user.recovery_codes_hash.is_none());
        assert!(pending_user.totp_pending_secret_enc.is_some());
        assert!(pending_user.recovery_codes_pending_hash.is_some());

        service
            .confirm_enrollment(user_id, "123456")
            .await
            .unwrap_or_else(|_| unreachable!());

        let confirmed_user = repository.snapshot();
        assert!(confirmed_user.totp_enabled);
        assert!(confirmed_user.totp_secret_enc.is_some());
        assert!(confirmed_user.recovery_codes_hash.is_some());
        assert!(confirmed_user.totp_pending_secret_enc.is_none());
        assert!(confirmed_user.recovery_codes_pending_hash.is_none());
    }

    #[tokio::test]
    async fn verify_totp_rejects_pending_enrollment() {
        let repository = FakeUserRepository::with_user(sample_user());
        let user_id = repository.snapshot().id;
        let service = build_service(Arc::new(repository.clone()));

        service
            .start_enrollment(user_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        let result = service.verify_totp(user_id, "123456").await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
