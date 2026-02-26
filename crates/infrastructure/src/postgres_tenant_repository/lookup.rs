use qryvanta_core::AppError;

use super::*;

impl PostgresTenantRepository {
    pub(super) async fn find_tenant_for_subject_impl(
        &self,
        subject: &str,
    ) -> AppResult<Option<TenantId>> {
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

    pub(super) async fn registration_mode_for_tenant_impl(
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
}
