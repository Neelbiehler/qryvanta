use super::*;

impl PostgresSecurityAdminRepository {
    pub(super) async fn save_runtime_field_permissions_impl(
        &self,
        tenant_id: TenantId,
        input: SaveRuntimeFieldPermissionsInput,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        sqlx::query(
            r#"
            DELETE FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND subject = $2
              AND entity_logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.subject.as_str())
        .bind(input.entity_logical_name.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to clear runtime field permissions for subject '{}' and entity '{}': {error}",
                input.subject, input.entity_logical_name
            ))
        })?;

        for field in &input.fields {
            sqlx::query(
                r#"
                INSERT INTO runtime_subject_field_permissions (
                    tenant_id,
                    subject,
                    entity_logical_name,
                    field_logical_name,
                    can_read,
                    can_write
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (tenant_id, subject, entity_logical_name, field_logical_name)
                DO UPDATE
                SET can_read = EXCLUDED.can_read,
                    can_write = EXCLUDED.can_write,
                    updated_at = now()
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(input.subject.as_str())
            .bind(input.entity_logical_name.as_str())
            .bind(field.field_logical_name.as_str())
            .bind(field.can_read)
            .bind(field.can_write)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to save runtime field permission for field '{}': {error}",
                    field.field_logical_name
                ))
            })?;
        }

        let rows = sqlx::query_as::<_, RuntimeFieldPermissionRow>(
            r#"
            SELECT
                subject,
                entity_logical_name,
                field_logical_name,
                can_read,
                can_write,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
            FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND subject = $2
              AND entity_logical_name = $3
            ORDER BY field_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.subject.as_str())
        .bind(input.entity_logical_name.as_str())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list saved runtime field permissions for subject '{}' and entity '{}': {error}",
                input.subject, input.entity_logical_name
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| RuntimeFieldPermissionEntry {
                subject: row.subject,
                entity_logical_name: row.entity_logical_name,
                field_logical_name: row.field_logical_name,
                can_read: row.can_read,
                can_write: row.can_write,
                updated_at: row.updated_at,
            })
            .collect())
    }

    pub(super) async fn list_runtime_field_permissions_impl(
        &self,
        tenant_id: TenantId,
        subject: Option<&str>,
        entity_logical_name: Option<&str>,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        let rows = sqlx::query_as::<_, RuntimeFieldPermissionRow>(
            r#"
            SELECT
                subject,
                entity_logical_name,
                field_logical_name,
                can_read,
                can_write,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
            FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR subject = $2)
              AND ($3::TEXT IS NULL OR entity_logical_name = $3)
            ORDER BY subject, entity_logical_name, field_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to list runtime field permissions: {error}"))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| RuntimeFieldPermissionEntry {
                subject: row.subject,
                entity_logical_name: row.entity_logical_name,
                field_logical_name: row.field_logical_name,
                can_read: row.can_read,
                can_write: row.can_write,
                updated_at: row.updated_at,
            })
            .collect())
    }
}
