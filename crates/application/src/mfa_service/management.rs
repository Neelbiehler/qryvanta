use qryvanta_core::AppError;
use qryvanta_domain::UserId;

use super::recovery_codes::{generate_recovery_codes, hash_recovery_codes};
use super::*;

impl MfaService {
    /// Disables TOTP for a user. Requires password re-authentication.
    pub async fn disable_totp(&self, user_id: UserId, password: &str) -> AppResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        let Some(ref stored_hash) = user.password_hash else {
            return Err(AppError::Validation(
                "password re-authentication required".to_owned(),
            ));
        };

        let valid = self
            .password_hasher
            .verify_password(password, stored_hash)?;

        if !valid {
            return Err(AppError::Unauthorized("incorrect password".to_owned()));
        }

        self.user_repository.disable_totp(user_id).await
    }

    /// Regenerates recovery codes. Requires password re-authentication.
    pub async fn regenerate_recovery_codes(
        &self,
        user_id: UserId,
        password: &str,
    ) -> AppResult<Vec<String>> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        let Some(ref stored_hash) = user.password_hash else {
            return Err(AppError::Validation(
                "password re-authentication required".to_owned(),
            ));
        };

        let valid = self
            .password_hasher
            .verify_password(password, stored_hash)?;

        if !valid {
            return Err(AppError::Unauthorized("incorrect password".to_owned()));
        }

        let codes = generate_recovery_codes();
        let hashed = hash_recovery_codes(&codes);

        self.user_repository
            .update_recovery_codes(user_id, &hashed)
            .await?;

        Ok(codes)
    }
}
