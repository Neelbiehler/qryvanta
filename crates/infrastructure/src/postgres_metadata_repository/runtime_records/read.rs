use super::*;

impl PostgresMetadataRepository {
    pub(in super::super) async fn list_runtime_records_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let limit = i64::try_from(query.limit).map_err(|error| {
            AppError::Validation(format!("invalid runtime record list limit: {error}"))
        })?;
        let offset = i64::try_from(query.offset).map_err(|error| {
            AppError::Validation(format!("invalid runtime record list offset: {error}"))
        })?;

        let started_at = std::time::Instant::now();
        let rows_result = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            SELECT id, entity_logical_name, data
            FROM runtime_records
            WHERE tenant_id = $1
              AND entity_logical_name = $2
              AND ($3::TEXT IS NULL OR created_by_subject = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(query.owner_subject.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *transaction)
        .await;

        warn_if_runtime_query_slow(
            "runtime_records.list",
            tenant_id,
            entity_logical_name,
            started_at,
        );

        let rows = rows_result.map_err(|error| {
            AppError::Internal(format!(
                "failed to list runtime records for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record list transaction: {error}"
            ))
        })?;

        rows.into_iter().map(runtime_record_from_row).collect()
    }

    pub(in super::super) async fn find_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<Option<RuntimeRecord>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let record_uuid = parse_runtime_record_uuid(record_id)?;

        let row = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            SELECT id, entity_logical_name, data
            FROM runtime_records
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find runtime record '{}' for entity '{}' in tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record lookup transaction: {error}"
            ))
        })?;

        row.map(runtime_record_from_row).transpose()
    }

    pub(in super::super) async fn delete_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        workflow_event: Option<RuntimeRecordWorkflowEventInput>,
    ) -> AppResult<()> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let record_uuid = parse_runtime_record_uuid(record_id)?;

        let deleted = sqlx::query(
            r#"
            DELETE FROM runtime_records
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to delete runtime record '{}' for entity '{}' in tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?;

        if deleted.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            )));
        }

        super::write::enqueue_runtime_record_workflow_event(
            &mut transaction,
            tenant_id,
            entity_logical_name,
            record_id,
            workflow_event,
        )
        .await?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record delete transaction: {error}"
            ))
        })?;

        Ok(())
    }

    pub(in super::super) async fn runtime_record_exists_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<bool> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let record_uuid = parse_runtime_record_uuid(record_id)?;

        let exists = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM runtime_records
                WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            )
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to check runtime record existence for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record existence transaction: {error}"
            ))
        })?;

        Ok(exists)
    }

    pub(in super::super) async fn runtime_record_owned_by_subject_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let record_uuid = parse_runtime_record_uuid(record_id)?;

        let is_owned = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM runtime_records
                WHERE tenant_id = $1
                  AND entity_logical_name = $2
                  AND id = $3
                  AND created_by_subject = $4
            )
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .bind(subject)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to evaluate runtime record ownership for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record ownership transaction: {error}"
            ))
        })?;

        Ok(is_owned)
    }
}
