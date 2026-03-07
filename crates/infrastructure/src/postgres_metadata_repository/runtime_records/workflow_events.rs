use super::*;

impl PostgresMetadataRepository {
    pub(in super::super) async fn claim_runtime_record_workflow_events_impl(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedRuntimeRecordWorkflowEvent>> {
        let mut transaction = begin_workflow_worker_transaction(&self.pool).await?;

        let claim_rows = sqlx::query_as::<_, RuntimeRecordWorkflowEventRow>(
            r#"
            WITH candidate_events AS (
                SELECT id
                FROM workflow_runtime_trigger_events
                WHERE (
                        status = 'pending'
                        OR (status = 'leased' AND lease_expires_at < now())
                      )
                  AND ($4::UUID IS NULL OR tenant_id = $4)
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            ),
            leased_events AS (
                UPDATE workflow_runtime_trigger_events events
                SET
                    status = 'leased',
                    leased_by = $2,
                    lease_token = gen_random_uuid()::TEXT,
                    lease_expires_at = now() + make_interval(secs => $3::INT),
                    updated_at = now()
                FROM candidate_events
                WHERE events.id = candidate_events.id
                RETURNING
                    events.id,
                    events.tenant_id,
                    events.trigger_type,
                    events.entity_logical_name,
                    events.record_id,
                    events.emitted_by_subject,
                    events.payload,
                    events.lease_token
            )
            SELECT
                id,
                tenant_id,
                trigger_type,
                entity_logical_name,
                record_id,
                emitted_by_subject,
                payload,
                lease_token
            FROM leased_events
            ORDER BY id
            "#,
        )
        .bind(i64::try_from(limit).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime workflow event claim limit: {error}"
            ))
        })?)
        .bind(worker_id)
        .bind(i32::try_from(lease_seconds).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime workflow event lease_seconds: {error}"
            ))
        })?)
        .bind(tenant_filter.map(|value| value.as_uuid()))
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to claim runtime workflow events for worker '{worker_id}': {error}"
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime workflow event claim transaction: {error}"
            ))
        })?;

        claim_rows
            .into_iter()
            .map(runtime_record_workflow_event_from_row)
            .collect()
    }

    pub(in super::super) async fn complete_runtime_record_workflow_event_impl(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        let event_uuid = Uuid::parse_str(event_id).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime workflow event id '{event_id}': {error}"
            ))
        })?;
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

        let result = sqlx::query(
            r#"
            UPDATE workflow_runtime_trigger_events
            SET
                status = 'completed',
                leased_by = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                updated_at = now(),
                processed_at = now()
            WHERE tenant_id = $1
              AND id = $2
              AND leased_by = $3
              AND lease_token = $4
              AND status = 'leased'
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(event_uuid)
        .bind(worker_id)
        .bind(lease_token)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to complete runtime workflow event '{event_id}' for tenant '{tenant_id}' worker '{worker_id}': {error}"
            ))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "runtime workflow event '{event_id}' is not currently leased by worker '{worker_id}' with matching lease token"
            )));
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime workflow event completion transaction: {error}"
            ))
        })?;

        Ok(())
    }

    pub(in super::super) async fn release_runtime_record_workflow_event_impl(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        let event_uuid = Uuid::parse_str(event_id).map_err(|error| {
            AppError::Validation(format!(
                "invalid runtime workflow event id '{event_id}': {error}"
            ))
        })?;
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

        let result = sqlx::query(
            r#"
            UPDATE workflow_runtime_trigger_events
            SET
                status = 'pending',
                leased_by = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                attempt_count = attempt_count + 1,
                last_error = $5,
                updated_at = now()
            WHERE tenant_id = $1
              AND id = $2
              AND leased_by = $3
              AND lease_token = $4
              AND status = 'leased'
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(event_uuid)
        .bind(worker_id)
        .bind(lease_token)
        .bind(error_message)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to release runtime workflow event '{event_id}' for tenant '{tenant_id}' worker '{worker_id}': {error}"
            ))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "runtime workflow event '{event_id}' is not currently leased by worker '{worker_id}' with matching lease token"
            )));
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime workflow event release transaction: {error}"
            ))
        })?;

        Ok(())
    }
}
