use qryvanta_core::AppError;

use super::*;

impl PostgresAuthTokenRepository {
    pub(super) async fn invalidate_tokens_for_user_impl(
        &self,
        user_id: UserId,
        token_type: AuthTokenType,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE auth_tokens
            SET used_at = now()
            WHERE user_id = $1
              AND token_type = $2
              AND used_at IS NULL
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(token_type.as_str())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to invalidate tokens: {error}")))?;

        Ok(())
    }
}
