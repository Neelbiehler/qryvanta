use qryvanta_core::AppError;

use super::*;

impl PostgresAuthorizationRepository {
    pub(super) async fn find_active_temporary_permission_grant_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>> {
        let row = sqlx::query_as::<_, TemporaryPermissionGrantRow>(
            r#"
            SELECT
                grants.id AS grant_id,
                grants.reason,
                to_char(grants.expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS expires_at
            FROM security_temporary_access_grants AS grants
            INNER JOIN security_temporary_access_grant_permissions AS permissions
                ON permissions.grant_id = grants.id
            WHERE grants.tenant_id = $1
              AND grants.subject = $2
              AND permissions.permission = $3
              AND grants.revoked_at IS NULL
              AND grants.expires_at > now()
            ORDER BY grants.expires_at DESC
            LIMIT 1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(permission.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve temporary permission grant for subject '{}' in tenant '{}': {error}",
                subject, tenant_id
            ))
        })?;

        Ok(row.map(|row| TemporaryPermissionGrant {
            grant_id: row.grant_id.to_string(),
            reason: row.reason,
            expires_at: row.expires_at,
        }))
    }
}
