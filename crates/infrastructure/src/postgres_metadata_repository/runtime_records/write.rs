use super::*;

impl PostgresMetadataRepository {
    pub(in super::super) async fn create_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        created_by_subject: &str,
    ) -> AppResult<RuntimeRecord> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start runtime record create transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        let created = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            INSERT INTO runtime_records (tenant_id, entity_logical_name, data, created_by_subject)
            VALUES ($1, $2, $3, $4)
            RETURNING id, entity_logical_name, data
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(&data)
        .bind(created_by_subject)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to create runtime record for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        index_unique_values(
            &mut transaction,
            tenant_id,
            entity_logical_name,
            created.id,
            &unique_values,
        )
        .await?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record create transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        runtime_record_from_row(created)
    }

    pub(in super::super) async fn update_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        let record_uuid = parse_runtime_record_uuid(record_id)?;

        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start runtime record update transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        let updated = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            UPDATE runtime_records
            SET data = $4,
                updated_at = now()
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            RETURNING id, entity_logical_name, data
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .bind(&data)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to update runtime record '{}' for entity '{}' in tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            ))
        })?;

        sqlx::query(
            r#"
            DELETE FROM runtime_record_unique_values
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND record_id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to clear unique field index for runtime record '{}' in entity '{}' and tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?;

        index_unique_values(
            &mut transaction,
            tenant_id,
            entity_logical_name,
            record_uuid,
            &unique_values,
        )
        .await?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record update transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        runtime_record_from_row(updated)
    }
}
