use super::*;

impl PostgresWorkflowRepository {
    pub(super) async fn create_run_impl(
        &self,
        tenant_id: TenantId,
        input: CreateWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"
            INSERT INTO workflow_execution_runs (
                tenant_id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                started_at
            )
            VALUES ($1, $2, $3, $4, $5, 'running', 0, now())
            RETURNING
                id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                dead_letter_reason,
                started_at,
                finished_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.workflow_logical_name)
        .bind(input.trigger_type)
        .bind(input.trigger_entity_logical_name)
        .bind(input.trigger_payload)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to create workflow run for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        workflow_run_from_row(row)
    }

    pub(super) async fn append_run_attempt_impl(
        &self,
        tenant_id: TenantId,
        attempt: WorkflowRunAttempt,
    ) -> AppResult<()> {
        let run_id = uuid::Uuid::parse_str(attempt.run_id.as_str()).map_err(|error| {
            AppError::Validation(format!(
                "invalid workflow run id '{}': {error}",
                attempt.run_id
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO workflow_execution_attempts (
                run_id,
                tenant_id,
                attempt_number,
                status,
                error_message,
                executed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(run_id)
        .bind(tenant_id.as_uuid())
        .bind(attempt.attempt_number)
        .bind(attempt.status.as_str())
        .bind(attempt.error_message)
        .bind(attempt.executed_at)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to append workflow run attempt for run '{}' tenant '{}': {error}",
                attempt.run_id, tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn complete_run_impl(
        &self,
        tenant_id: TenantId,
        input: CompleteWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        let run_id = uuid::Uuid::parse_str(input.run_id.as_str()).map_err(|error| {
            AppError::Validation(format!(
                "invalid workflow run id '{}': {error}",
                input.run_id
            ))
        })?;

        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"
            UPDATE workflow_execution_runs
            SET
                status = $3,
                attempts = $4,
                dead_letter_reason = $5,
                finished_at = now()
            WHERE tenant_id = $1 AND id = $2
            RETURNING
                id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                dead_letter_reason,
                started_at,
                finished_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .bind(input.status.as_str())
        .bind(input.attempts)
        .bind(input.dead_letter_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to complete workflow run '{}' for tenant '{}': {error}",
                run_id, tenant_id
            ))
        })?;

        workflow_run_from_row(row)
    }

    pub(super) async fn list_runs_impl(
        &self,
        tenant_id: TenantId,
        query: WorkflowRunListQuery,
    ) -> AppResult<Vec<WorkflowRun>> {
        let rows = sqlx::query_as::<_, WorkflowRunRow>(
            r#"
            SELECT
                id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                dead_letter_reason,
                started_at,
                finished_at
            FROM workflow_execution_runs
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR workflow_logical_name = $2)
            ORDER BY started_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(query.workflow_logical_name)
        .bind(i64::try_from(query.limit).map_err(|error| {
            AppError::Validation(format!("invalid workflow run list limit: {error}"))
        })?)
        .bind(i64::try_from(query.offset).map_err(|error| {
            AppError::Validation(format!("invalid workflow run list offset: {error}"))
        })?)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list workflow runs for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter().map(workflow_run_from_row).collect()
    }

    pub(super) async fn list_run_attempts_impl(
        &self,
        tenant_id: TenantId,
        run_id: &str,
    ) -> AppResult<Vec<WorkflowRunAttempt>> {
        let run_uuid = uuid::Uuid::parse_str(run_id).map_err(|error| {
            AppError::Validation(format!("invalid workflow run id '{}': {error}", run_id))
        })?;

        let rows = sqlx::query_as::<_, WorkflowRunAttemptRow>(
            r#"
            SELECT run_id, attempt_number, status, error_message, executed_at
            FROM workflow_execution_attempts
            WHERE tenant_id = $1 AND run_id = $2
            ORDER BY attempt_number
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list workflow run attempts for run '{}' tenant '{}': {error}",
                run_id, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(workflow_run_attempt_from_row)
            .collect()
    }
}
