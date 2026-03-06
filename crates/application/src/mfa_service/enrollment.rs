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

        self.user_repository
            .begin_totp_enrollment(user_id, &encrypted_secret, &hashed_codes)
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

        let Some(ref encrypted_secret) = user.totp_pending_secret_enc else {
            return Err(AppError::Validation(
                "no TOTP enrollment in progress".to_owned(),
            ));
        };

        let secret_bytes = self.secret_encryptor.decrypt(encrypted_secret)?;
        let valid = self.totp_provider.verify_code(&secret_bytes, code)?;

        if !valid {
            return Err(AppError::Unauthorized("invalid TOTP code".to_owned()));
        }

        self.user_repository
            .confirm_totp_enrollment(user_id)
            .await?;

        Ok(())
    }
}
