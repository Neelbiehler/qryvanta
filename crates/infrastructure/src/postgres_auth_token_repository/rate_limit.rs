use qryvanta_core::AppError;

use super::*;

impl PostgresAuthTokenRepository {
    pub(super) async fn count_recent_tokens_impl(
        &self,
        email: &str,
        token_type: AuthTokenType,
        since: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM auth_tokens
            WHERE LOWER(email) = LOWER($1)
              AND token_type = $2
              AND created_at >= $3
            "#,
        )
        .bind(email)
        .bind(token_type.as_str())
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to count recent tokens: {error}")))?;

        Ok(count)
    }
}
