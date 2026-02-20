//! TOTP provider implementation using the `totp-rs` crate.

use async_trait::async_trait;
use qryvanta_application::TotpProvider;
use qryvanta_core::{AppError, AppResult};
use totp_rs::{Algorithm, Secret, TOTP};

/// TOTP provider with RFC 6238 compliance.
#[derive(Clone)]
pub struct TotpRsProvider {
    issuer: String,
}

impl TotpRsProvider {
    /// Creates a new TOTP provider.
    #[must_use]
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
        }
    }
}

#[async_trait]
impl TotpProvider for TotpRsProvider {
    fn generate_secret(&self, email: &str) -> AppResult<(Vec<u8>, String, String)> {
        let secret = Secret::generate_secret();
        let secret_bytes = secret.to_bytes().map_err(|error| {
            AppError::Internal(format!("failed to generate TOTP secret: {error}"))
        })?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes.clone(),
            Some(self.issuer.clone()),
            email.to_owned(),
        )
        .map_err(|error| AppError::Internal(format!("failed to create TOTP instance: {error}")))?;

        let base32 = secret.to_encoded().to_string();
        let otpauth_uri = totp.get_url();

        Ok((secret_bytes, base32, otpauth_uri))
    }

    fn verify_code(&self, secret_bytes: &[u8], code: &str) -> AppResult<bool> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1, // skew: allows +/-1 time step
            30,
            secret_bytes.to_vec(),
            Some("Qryvanta".to_owned()),
            String::new(),
        )
        .map_err(|error| AppError::Internal(format!("failed to create TOTP instance: {error}")))?;

        let valid = totp
            .check_current(code)
            .map_err(|error| AppError::Internal(format!("failed to verify TOTP code: {error}")))?;

        Ok(valid)
    }
}
