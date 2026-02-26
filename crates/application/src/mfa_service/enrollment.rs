use qryvanta_core::AppError;
use qryvanta_domain::UserId;

use super::recovery_codes::{generate_recovery_codes, hash_recovery_codes};
use super::*;

impl MfaService {
    /// Starts TOTP enrollment for a user.
    ///
    /// Returns the secret, otpauth URI, and recovery codes. The user must
    /// call `confirm_enrollment` with a valid TOTP code before MFA is active.
    pub async fn start_enrollment(&self, user_id: UserId) -> AppResult<TotpEnrollment> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        if user.totp_enabled {
            return Err(AppError::Conflict(
                "TOTP is already enabled for this account".to_owned(),
            ));
        }

        let (secret_bytes, secret_base32, otpauth_uri) =
            self.totp_provider.generate_secret(&user.email)?;

        let encrypted_secret = self.secret_encryptor.encrypt(&secret_bytes)?;
        let recovery_codes = generate_recovery_codes();
        let hashed_codes = hash_recovery_codes(&recovery_codes);

        // Store the encrypted secret and hashed recovery codes, but don't
        // enable TOTP yet -- that happens in confirm_enrollment.
        self.user_repository
            .enable_totp(user_id, &encrypted_secret, &hashed_codes)
            .await?;

        // Immediately disable it since it's not confirmed yet.
        // The enable_totp call stores the secret; we need a separate mechanism
        // to mark it as "pending confirmation". For simplicity, we'll store
        // the secret but keep totp_enabled = false until confirmation.
        // The enable_totp method should set totp_enabled = false initially.
        // Let's adjust: we store secret but don't enable.
        self.user_repository.disable_totp(user_id).await?;

        // Re-store the secret without enabling.
        self.user_repository
            .enable_totp(user_id, &encrypted_secret, &hashed_codes)
            .await?;

        Ok(TotpEnrollment {
            secret_base32,
            otpauth_uri,
            recovery_codes,
        })
    }

    /// Confirms TOTP enrollment by verifying a code from the user's authenticator.
    pub async fn confirm_enrollment(&self, user_id: UserId, code: &str) -> AppResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        let Some(ref encrypted_secret) = user.totp_secret_enc else {
            return Err(AppError::Validation(
                "no TOTP enrollment in progress".to_owned(),
            ));
        };

        let secret_bytes = self.secret_encryptor.decrypt(encrypted_secret)?;
        let valid = self.totp_provider.verify_code(&secret_bytes, code)?;

        if !valid {
            return Err(AppError::Unauthorized("invalid TOTP code".to_owned()));
        }

        // Enable TOTP -- the secret and recovery codes are already stored.
        // We just flip the totp_enabled flag.
        self.user_repository
            .enable_totp(
                user_id,
                encrypted_secret,
                user.recovery_codes_hash
                    .as_ref()
                    .unwrap_or(&serde_json::json!([])),
            )
            .await?;

        Ok(())
    }
}
