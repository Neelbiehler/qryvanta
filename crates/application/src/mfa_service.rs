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
