use qryvanta_core::AppError;
use qryvanta_domain::{UserId, validate_password};

use super::*;

impl UserService {
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
}
