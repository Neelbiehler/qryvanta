use async_trait::async_trait;
use qryvanta_application::TenantRepository;
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::RegistrationMode;
use sqlx::PgPool;

use crate::postgres_security_admin_repository::assign_owner_role_grants;

/// PostgreSQL-backed tenant membership repository.
#[derive(Clone)]
pub struct PostgresTenantRepository {
    pool: PgPool,
}

impl PostgresTenantRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>> {
        let tenant_id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            SELECT tenant_id
            FROM tenant_memberships
            WHERE subject = $1
            LIMIT 1
            "#,
        )
        .bind(subject)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to resolve tenant membership: {error}"))
        })?;

        Ok(tenant_id.map(TenantId::from_uuid))
    }

    async fn registration_mode_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<RegistrationMode> {
        let stored_mode = sqlx::query_scalar::<_, String>(
            r#"
            SELECT registration_mode
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve tenant registration mode: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        RegistrationMode::parse(stored_mode.as_str())
    }

    async fn create_membership(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        sqlx::query(
            r#"
            INSERT INTO tenant_memberships (tenant_id, subject, display_name, email)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (tenant_id, subject) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(display_name)
        .bind(email)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to create membership: {error}")))?;

        assign_owner_role_grants(&mut transaction, tenant_id, subject).await?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        Ok(())
    }

    async fn ensure_membership_for_subject(
        &self,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId> {
        if let Some(tenant_id) = self.find_tenant_for_subject(subject).await? {
            return Ok(tenant_id);
        }

        let tenant_id = preferred_tenant_id.unwrap_or_default();
        let tenant_name = format!("{display_name} Workspace");

        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        sqlx::query(
            r#"
            INSERT INTO tenants (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(tenant_name)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to ensure tenant exists: {error}")))?;

        sqlx::query(
            r#"
            INSERT INTO tenant_memberships (tenant_id, subject, display_name, email)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (tenant_id, subject) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(display_name)
        .bind(email)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to ensure tenant membership exists: {error}"
            ))
        })?;

        assign_owner_role_grants(&mut transaction, tenant_id, subject).await?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        self.find_tenant_for_subject(subject).await?.ok_or_else(|| {
            AppError::Internal(format!(
                "subject '{subject}' membership was not persisted after bootstrap"
            ))
        })
    }
}
