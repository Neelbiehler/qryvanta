use super::*;

impl PostgresUserRepository {
    pub(super) async fn find_by_email_impl(&self, email: &str) -> AppResult<Option<UserRecord>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, email_verified, password_hash, totp_enabled,
                   totp_secret_enc, recovery_codes_hash, failed_login_count, locked_until
            FROM users
            WHERE LOWER(email) = LOWER($1)
            LIMIT 1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to find user by email: {error}")))?;

        Ok(row.map(UserRecord::from))
    }

    pub(super) async fn find_by_id_impl(&self, user_id: UserId) -> AppResult<Option<UserRecord>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, email_verified, password_hash, totp_enabled,
                   totp_secret_enc, recovery_codes_hash, failed_login_count, locked_until
            FROM users
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to find user by id: {error}")))?;

        Ok(row.map(UserRecord::from))
    }

    pub(super) async fn find_by_subject_impl(
        &self,
        subject: &str,
    ) -> AppResult<Option<UserRecord>> {
        // Try parsing as UUID first (new-style user_id as subject).
        if let Ok(uuid) = uuid::Uuid::parse_str(subject) {
            return self.find_by_id_impl(UserId::from_uuid(uuid)).await;
        }

        // Fall back to email lookup (legacy subjects might be emails).
        self.find_by_email_impl(subject).await
    }
}
