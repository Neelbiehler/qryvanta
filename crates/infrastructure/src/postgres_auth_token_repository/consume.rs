use qryvanta_core::AppError;

use super::*;

impl PostgresAuthTokenRepository {
    pub(super) async fn consume_valid_token_impl(
        &self,
        token_hash: &str,
        token_type: AuthTokenType,
    ) -> AppResult<Option<AuthTokenRecord>> {
        let row = sqlx::query_as::<_, TokenRow>(
            r#"
            UPDATE auth_tokens
            SET used_at = now()
            WHERE token_hash = $1
              AND token_type = $2
              AND used_at IS NULL
              AND expires_at > now()
            RETURNING id, user_id, email, token_hash, token_type, expires_at, used_at, metadata
            "#,
        )
        .bind(token_hash)
        .bind(token_type.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to consume auth token: {error}")))?;

        Ok(row.map(AuthTokenRecord::from))
    }
}
