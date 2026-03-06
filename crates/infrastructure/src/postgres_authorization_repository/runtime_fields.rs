use qryvanta_core::AppError;

use super::*;

impl PostgresAuthorizationRepository {
    pub(super) async fn list_runtime_field_grants_for_subject_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
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
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load runtime field grants for subject '{}' in tenant '{}': {error}",
                subject, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped runtime field grant lookup transaction: {error}"
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
