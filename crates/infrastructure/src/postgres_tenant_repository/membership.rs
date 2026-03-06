use qryvanta_core::AppError;

use super::*;

impl PostgresTenantRepository {
    pub(super) async fn create_membership_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

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
            AppError::Internal(format!(
                "failed to commit tenant-scoped membership create transaction: {error}"
            ))
        })?;

        Ok(())
    }

    pub(super) async fn ensure_membership_for_subject_impl(
        &self,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId> {
        if let Some(tenant_id) = self.find_tenant_for_subject_impl(subject).await? {
            return Ok(tenant_id);
        }

        let tenant_id = preferred_tenant_id.unwrap_or_default();
        let tenant_name = format!("{display_name} Workspace");

        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to begin tenant bootstrap transaction: {error}"
            ))
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

        stamp_tenant_context(&mut *transaction, tenant_id).await?;

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
            AppError::Internal(format!(
                "failed to commit tenant bootstrap transaction: {error}"
            ))
        })?;

        self.find_tenant_for_subject_impl(subject)
            .await?
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "subject '{subject}' membership was not persisted after bootstrap"
                ))
            })
    }
}
