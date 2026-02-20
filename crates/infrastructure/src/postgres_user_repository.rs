//! PostgreSQL-backed user repository.

use async_trait::async_trait;
use sqlx::PgPool;

use qryvanta_application::{UserRecord, UserRepository};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::UserId;

/// PostgreSQL implementation of the user repository port.
#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    email_verified: bool,
    password_hash: Option<String>,
    totp_enabled: bool,
    totp_secret_enc: Option<Vec<u8>>,
    recovery_codes_hash: Option<serde_json::Value>,
    failed_login_count: i32,
    locked_until: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<UserRow> for UserRecord {
    fn from(row: UserRow) -> Self {
        Self {
            id: UserId::from_uuid(row.id),
            email: row.email,
            email_verified: row.email_verified,
            password_hash: row.password_hash,
            totp_enabled: row.totp_enabled,
            totp_secret_enc: row.totp_secret_enc,
            recovery_codes_hash: row.recovery_codes_hash,
            failed_login_count: row.failed_login_count,
            locked_until: row.locked_until,
        }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserRecord>> {
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

    async fn find_by_id(&self, user_id: UserId) -> AppResult<Option<UserRecord>> {
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

    async fn create(
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
        .map_err(|error| {
            if let sqlx::Error::Database(ref database_error) = error
                && database_error.code().as_deref() == Some("23505")
            {
                return AppError::Conflict("an account with this email already exists".to_owned());
            }
            AppError::Internal(format!("failed to create user: {error}"))
        })?;

        Ok(UserId::from_uuid(id))
    }

    async fn update_password(&self, user_id: UserId, password_hash: &str) -> AppResult<()> {
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

    async fn record_failed_login(&self, user_id: UserId) -> AppResult<()> {
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
        .map_err(|error| {
            AppError::Internal(format!("failed to record failed login: {error}"))
        })?;

        Ok(())
    }

    async fn reset_failed_logins(&self, user_id: UserId) -> AppResult<()> {
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

    async fn mark_email_verified(&self, user_id: UserId) -> AppResult<()> {
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

    async fn update_display_name(
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

    async fn update_email(&self, user_id: UserId, new_email: &str) -> AppResult<()> {
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
        .map_err(|error| {
            if let sqlx::Error::Database(ref database_error) = error
                && database_error.code().as_deref() == Some("23505")
            {
                return AppError::Conflict("an account with this email already exists".to_owned());
            }
            AppError::Internal(format!("failed to update email: {error}"))
        })?;

        Ok(())
    }

    async fn enable_totp(
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

    async fn disable_totp(&self, user_id: UserId) -> AppResult<()> {
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

    async fn update_recovery_codes(
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

    async fn find_by_subject(&self, subject: &str) -> AppResult<Option<UserRecord>> {
        // Try parsing as UUID first (new-style user_id as subject).
        if let Ok(uuid) = uuid::Uuid::parse_str(subject) {
            return self.find_by_id(UserId::from_uuid(uuid)).await;
        }

        // Fall back to email lookup (legacy subjects might be emails).
        self.find_by_email(subject).await
    }
}
