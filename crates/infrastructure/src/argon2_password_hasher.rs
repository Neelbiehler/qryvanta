//! Argon2id password hasher implementation.
//!
//! Uses OWASP-recommended Argon2id parameters:
//! m=19456 (19 MiB), t=2, p=1.

use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use qryvanta_application::PasswordHasher as PasswordHasherPort;
use qryvanta_core::{AppError, AppResult};

/// Argon2id password hasher with OWASP-recommended parameters.
#[derive(Clone)]
pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Argon2PasswordHasher {
    /// Creates a new Argon2id hasher with recommended parameters.
    #[must_use]
    pub fn new() -> Self {
        // OWASP Password Storage: Argon2id with m=19456, t=2, p=1.
        let params = Params::new(19456, 2, 1, None).unwrap_or_else(|_| Params::default());

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Self { argon2 }
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasherPort for Argon2PasswordHasher {
    fn hash_password(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| AppError::Internal(format!("failed to hash password: {error}")))?;

        Ok(hash.to_string())
    }

    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash).map_err(|error| {
            AppError::Internal(format!("failed to parse password hash: {error}"))
        })?;

        match self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
        {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(error) => Err(AppError::Internal(format!(
                "password verification failed: {error}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qryvanta_application::PasswordHasher as PasswordHasherPort;
    use qryvanta_core::AppResult;

    #[test]
    fn hash_and_verify_correct_password() -> AppResult<()> {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash_password("my-secret-password")?;
        assert!(hasher.verify_password("my-secret-password", &hash)?);
        Ok(())
    }

    #[test]
    fn verify_wrong_password_returns_false() -> AppResult<()> {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash_password("correct-password")?;
        assert!(!hasher.verify_password("wrong-password", &hash)?);
        Ok(())
    }
}
