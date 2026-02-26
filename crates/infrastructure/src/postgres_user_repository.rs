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

mod account;
mod lookup;
mod mfa;

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserRecord>> {
        self.find_by_email_impl(email).await
    }

    async fn find_by_id(&self, user_id: UserId) -> AppResult<Option<UserRecord>> {
        self.find_by_id_impl(user_id).await
    }

    async fn create(
        &self,
        email: &str,
        password_hash: Option<&str>,
        email_verified: bool,
    ) -> AppResult<UserId> {
        self.create_impl(email, password_hash, email_verified).await
    }

    async fn update_password(&self, user_id: UserId, password_hash: &str) -> AppResult<()> {
        self.update_password_impl(user_id, password_hash).await
    }

    async fn record_failed_login(&self, user_id: UserId) -> AppResult<()> {
        self.record_failed_login_impl(user_id).await
    }

    async fn reset_failed_logins(&self, user_id: UserId) -> AppResult<()> {
        self.reset_failed_logins_impl(user_id).await
    }

    async fn mark_email_verified(&self, user_id: UserId) -> AppResult<()> {
        self.mark_email_verified_impl(user_id).await
    }

    async fn update_display_name(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        display_name: &str,
    ) -> AppResult<()> {
        self.update_display_name_impl(user_id, tenant_id, display_name)
            .await
    }

    async fn update_email(&self, user_id: UserId, new_email: &str) -> AppResult<()> {
        self.update_email_impl(user_id, new_email).await
    }

    async fn enable_totp(
        &self,
        user_id: UserId,
        totp_secret_enc: &[u8],
        recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()> {
        self.enable_totp_impl(user_id, totp_secret_enc, recovery_codes_hash)
            .await
    }

    async fn disable_totp(&self, user_id: UserId) -> AppResult<()> {
        self.disable_totp_impl(user_id).await
    }

    async fn update_recovery_codes(
        &self,
        user_id: UserId,
        recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()> {
        self.update_recovery_codes_impl(user_id, recovery_codes_hash)
            .await
    }

    async fn find_by_subject(&self, subject: &str) -> AppResult<Option<UserRecord>> {
        self.find_by_subject_impl(subject).await
    }
}

fn email_conflict_or_internal(error: sqlx::Error, operation: &str) -> AppError {
    if let sqlx::Error::Database(ref database_error) = error
        && database_error.code().as_deref() == Some("23505")
    {
        return AppError::Conflict("an account with this email already exists".to_owned());
    }

    AppError::Internal(format!("failed to {operation}: {error}"))
}
