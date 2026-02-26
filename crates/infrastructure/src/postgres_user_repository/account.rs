use super::*;

impl PostgresUserRepository {
    pub(super) async fn create_impl(
        &self,
        email: &str,
        password_hash: Option<&str>,
        email_verified: bool,
    ) -> AppResult<UserId> {
        let id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            INSERT INTO users (email, password_hash, email_verified)
            VALUES (LOWER($1), $2, $3)
            RETURNING id
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(email_verified)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| email_conflict_or_internal(error, "create user"))?;

        Ok(UserId::from_uuid(id))
    }

    pub(super) async fn update_password_impl(
        &self,
        user_id: UserId,
        password_hash: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2, password_changed_at = now(), updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(password_hash)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to update password: {error}")))?;

        Ok(())
    }

    pub(super) async fn record_failed_login_impl(&self, user_id: UserId) -> AppResult<()> {
        // Exponential lockout: lock for 2^(n-3) seconds after n failures,
        // starting at the 3rd failure. Permanent lock after 10 failures.
        sqlx::query(
            r#"
            UPDATE users
            SET failed_login_count = failed_login_count + 1,
                locked_until = CASE
                    WHEN failed_login_count + 1 >= 10
                        THEN now() + interval '24 hours'
                    WHEN failed_login_count + 1 >= 3
                        THEN now() + make_interval(secs => power(2, LEAST(failed_login_count + 1 - 3, 10))::int)
                    ELSE NULL
                END,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to record failed login: {error}")))?;

        Ok(())
    }

    pub(super) async fn reset_failed_logins_impl(&self, user_id: UserId) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET failed_login_count = 0, locked_until = NULL, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to reset failed logins: {error}")))?;

        Ok(())
    }

    pub(super) async fn mark_email_verified_impl(&self, user_id: UserId) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = TRUE, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to mark email verified: {error}")))?;

        Ok(())
    }

    pub(super) async fn update_display_name_impl(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        display_name: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE tenant_memberships
            SET display_name = $3
            WHERE user_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(tenant_id.as_uuid())
        .bind(display_name)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to update display name: {error}")))?;

        Ok(())
    }

    pub(super) async fn update_email_impl(
        &self,
        user_id: UserId,
        new_email: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET email = LOWER($2), email_verified = FALSE, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(new_email)
        .execute(&self.pool)
        .await
        .map_err(|error| email_conflict_or_internal(error, "update email"))?;

        Ok(())
    }
}
