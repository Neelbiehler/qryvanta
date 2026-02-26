use qryvanta_core::AppError;
use qryvanta_domain::UserId;

use super::recovery_codes::hash_single_code;
use super::*;

impl MfaService {
    /// Verifies a TOTP code for an authenticated MFA challenge.
    pub async fn verify_totp(&self, user_id: UserId, code: &str) -> AppResult<bool> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        if !user.totp_enabled {
            return Err(AppError::Validation(
                "TOTP is not enabled for this account".to_owned(),
            ));
        }

        let Some(ref encrypted_secret) = user.totp_secret_enc else {
            return Err(AppError::Internal(
                "TOTP enabled but secret is missing".to_owned(),
            ));
        };

        let secret_bytes = self.secret_encryptor.decrypt(encrypted_secret)?;
        self.totp_provider.verify_code(&secret_bytes, code)
    }

    /// Verifies a recovery code and marks it as used.
    pub async fn verify_recovery_code(&self, user_id: UserId, code: &str) -> AppResult<bool> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        let Some(ref codes_json) = user.recovery_codes_hash else {
            return Ok(false);
        };

        let hashed_codes: Vec<String> =
            serde_json::from_value(codes_json.clone()).map_err(|error| {
                AppError::Internal(format!("failed to parse recovery codes: {error}"))
            })?;

        let code_hash = hash_single_code(code);

        let mut matched = false;
        let mut remaining_codes: Vec<String> = Vec::new();

        for stored_hash in &hashed_codes {
            if !matched && *stored_hash == code_hash {
                matched = true;
                // Don't add to remaining -- it's consumed.
            } else {
                remaining_codes.push(stored_hash.clone());
            }
        }

        if matched {
            let updated_json = serde_json::to_value(&remaining_codes).map_err(|error| {
                AppError::Internal(format!("failed to serialize recovery codes: {error}"))
            })?;

            self.user_repository
                .update_recovery_codes(user_id, &updated_json)
                .await?;
        }

        Ok(matched)
    }
}
