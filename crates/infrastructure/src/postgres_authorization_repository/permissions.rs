use std::str::FromStr;

use qryvanta_core::AppError;

use super::*;

impl PostgresAuthorizationRepository {
    pub(super) async fn list_permissions_for_subject_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT DISTINCT grants.permission
            FROM rbac_subject_roles AS subject_roles
            INNER JOIN rbac_role_grants AS grants
                ON grants.role_id = subject_roles.role_id
            WHERE subject_roles.tenant_id = $1
                AND subject_roles.subject = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to load permissions: {error}")))?;

        rows.into_iter()
            .map(|row| {
                Permission::from_str(row.permission.as_str()).map_err(|error| {
                    AppError::Internal(format!(
                        "failed to decode permission '{}' for tenant '{}': {error}",
                        row.permission, tenant_id
                    ))
                })
            })
            .collect()
    }
}
