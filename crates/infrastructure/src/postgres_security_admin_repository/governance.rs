use super::*;

impl PostgresSecurityAdminRepository {
    pub(super) async fn registration_mode_impl(
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

        RegistrationMode::parse(stored_mode.as_str()).map_err(|error| {
            AppError::Internal(format!(
                "invalid tenant registration mode '{}' for tenant '{}': {error}",
                stored_mode, tenant_id
            ))
        })
    }

    pub(super) async fn set_registration_mode_impl(
        &self,
        tenant_id: TenantId,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode> {
        let stored_mode = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE tenants
            SET registration_mode = $2
            WHERE id = $1
            RETURNING registration_mode
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(registration_mode.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to update tenant registration mode: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        RegistrationMode::parse(stored_mode.as_str()).map_err(|error| {
            AppError::Internal(format!(
                "invalid tenant registration mode '{}' for tenant '{}': {error}",
                stored_mode, tenant_id
            ))
        })
    }

    pub(super) async fn audit_retention_policy_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<AuditRetentionPolicy> {
        let retention_days = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT audit_retention_days
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve tenant audit retention policy: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        Ok(AuditRetentionPolicy {
            retention_days: u16::try_from(retention_days).map_err(|_| {
                AppError::Internal(format!(
                    "invalid stored audit retention_days '{}' for tenant '{}'",
                    retention_days, tenant_id
                ))
            })?,
        })
    }

    pub(super) async fn set_audit_retention_policy_impl(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<AuditRetentionPolicy> {
        let stored_days = sqlx::query_scalar::<_, i32>(
            r#"
            UPDATE tenants
            SET audit_retention_days = $2
            WHERE id = $1
            RETURNING audit_retention_days
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(i32::from(retention_days))
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to update tenant audit retention policy: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        Ok(AuditRetentionPolicy {
            retention_days: u16::try_from(stored_days).map_err(|_| {
                AppError::Internal(format!(
                    "invalid stored audit retention_days '{}' for tenant '{}'",
                    stored_days, tenant_id
                ))
            })?,
        })
    }
}
