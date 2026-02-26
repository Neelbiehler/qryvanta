use super::*;

impl PostgresUserRepository {
    pub(super) async fn enable_totp_impl(
        &self,
        user_id: UserId,
        totp_secret_enc: &[u8],
        recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET totp_secret_enc = $2,
                recovery_codes_hash = $3,
                totp_enabled = TRUE,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(totp_secret_enc)
        .bind(recovery_codes_hash)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to enable TOTP: {error}")))?;

        Ok(())
    }

    pub(super) async fn disable_totp_impl(&self, user_id: UserId) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET totp_enabled = FALSE, totp_secret_enc = NULL,
                recovery_codes_hash = NULL, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to disable TOTP: {error}")))?;

        Ok(())
    }

    pub(super) async fn update_recovery_codes_impl(
        &self,
        user_id: UserId,
        recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET recovery_codes_hash = $2, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(recovery_codes_hash)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to update recovery codes: {error}")))?;

        Ok(())
    }
}
