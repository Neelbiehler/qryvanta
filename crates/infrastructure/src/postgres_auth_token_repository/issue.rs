use qryvanta_core::AppError;

use super::*;

impl PostgresAuthTokenRepository {
    pub(super) async fn create_token_impl(
        &self,
        user_id: Option<UserId>,
        email: &str,
        token_hash: &str,
        token_type: AuthTokenType,
        expires_at: chrono::DateTime<chrono::Utc>,
        metadata: Option<&serde_json::Value>,
    ) -> AppResult<uuid::Uuid> {
        let id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            INSERT INTO auth_tokens (user_id, email, token_hash, token_type, expires_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(user_id.map(|uid| uid.as_uuid()))
        .bind(email)
        .bind(token_hash)
        .bind(token_type.as_str())
        .bind(expires_at)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to create auth token: {error}")))?;

        Ok(id)
    }
}
