use qryvanta_core::AppError;

use super::*;

impl PostgresAuthorizationRepository {
    pub(super) async fn list_runtime_field_grants_for_subject_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        let rows = sqlx::query_as::<_, RuntimeFieldGrantRow>(
            r#"
            SELECT field_logical_name, can_read, can_write
            FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND subject = $2
              AND entity_logical_name = $3
            ORDER BY field_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load runtime field grants for subject '{}' in tenant '{}': {error}",
                subject, tenant_id
            ))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| RuntimeFieldGrant {
                field_logical_name: row.field_logical_name,
                can_read: row.can_read,
                can_write: row.can_write,
            })
            .collect())
    }
}
