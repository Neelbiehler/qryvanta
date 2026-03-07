use super::*;

impl PostgresWorkflowRepository {
    pub(super) async fn save_workflow_impl(
        &self,
        tenant_id: TenantId,
        workflow: WorkflowDefinition,
    ) -> AppResult<()> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let (trigger_type, trigger_entity) = workflow_trigger_parts(workflow.trigger());
        let steps = workflow_steps_to_json(workflow.steps())?;

        let result = sqlx::query(
            r#"
            INSERT INTO workflow_definitions (
                tenant_id,
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            ON CONFLICT (tenant_id, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                description = EXCLUDED.description,
                trigger_type = EXCLUDED.trigger_type,
                trigger_entity_logical_name = EXCLUDED.trigger_entity_logical_name,
                steps = EXCLUDED.steps,
                max_attempts = EXCLUDED.max_attempts,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(workflow.logical_name().as_str())
        .bind(workflow.display_name().as_str())
        .bind(workflow.description())
        .bind(trigger_type)
        .bind(trigger_entity)
        .bind(steps)
        .bind(i16::try_from(workflow.max_attempts()).map_err(|error| {
            AppError::Validation(format!("invalid workflow max_attempts value: {error}"))
        })?)
        .execute(&mut *transaction)
        .await;

        match result {
            Ok(_) => {
                transaction.commit().await.map_err(|error| {
                    AppError::Internal(format!(
                        "failed to commit tenant-scoped workflow save transaction: {error}"
                    ))
                })?;
                Ok(())
            }
            Err(error) => Err(AppError::Internal(format!(
                "failed to save workflow '{}' for tenant '{}': {error}",
                workflow.logical_name().as_str(),
                tenant_id
            ))),
        }
    }

    pub(super) async fn list_workflows_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let rows = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                lifecycle_state,
                current_published_version
            FROM workflow_definitions
            WHERE tenant_id = $1
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list workflows for tenant '{}': {error}",
                tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow list transaction: {error}"
            ))
        })?;

        rows.into_iter().map(workflow_definition_from_row).collect()
    }

    pub(super) async fn find_workflow_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                lifecycle_state,
                current_published_version
            FROM workflow_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find workflow '{}' for tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow find transaction: {error}"
            ))
        })?;

        row.map(workflow_definition_from_row).transpose()
    }

    pub(super) async fn find_published_workflow_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                versions.logical_name,
                versions.display_name,
                versions.description,
                versions.trigger_type,
                versions.trigger_entity_logical_name,
                versions.steps,
                versions.max_attempts,
                definitions.lifecycle_state,
                definitions.current_published_version
            FROM workflow_definitions definitions
            INNER JOIN workflow_published_versions versions
                ON versions.tenant_id = definitions.tenant_id
               AND versions.logical_name = definitions.logical_name
               AND versions.version = definitions.current_published_version
            WHERE definitions.tenant_id = $1
              AND definitions.logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find published workflow '{}' for tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped published workflow find transaction: {error}"
            ))
        })?;

        row.map(workflow_definition_from_row).transpose()
    }

    pub(super) async fn find_published_workflow_version_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        version: i32,
    ) -> AppResult<Option<WorkflowDefinition>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                versions.logical_name,
                versions.display_name,
                versions.description,
                versions.trigger_type,
                versions.trigger_entity_logical_name,
                versions.steps,
                versions.max_attempts,
                CASE
                    WHEN definitions.current_published_version = versions.version
                        THEN definitions.lifecycle_state
                    ELSE 'disabled'
                END AS lifecycle_state,
                versions.version AS current_published_version
            FROM workflow_published_versions versions
            INNER JOIN workflow_definitions definitions
                ON definitions.tenant_id = versions.tenant_id
               AND definitions.logical_name = versions.logical_name
            WHERE versions.tenant_id = $1
              AND versions.logical_name = $2
              AND versions.version = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .bind(version)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find published workflow '{}@v{}' for tenant '{}': {error}",
                logical_name, version, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped published workflow version find transaction: {error}"
            ))
        })?;

        row.map(workflow_definition_from_row).transpose()
    }

    pub(super) async fn publish_workflow_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        published_by: &str,
    ) -> AppResult<WorkflowDefinition> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let draft = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                lifecycle_state,
                current_published_version
            FROM workflow_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load workflow '{}' for publish tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "workflow '{}' does not exist for tenant '{}'",
                logical_name, tenant_id
            ))
        })?;

        let next_version = draft.current_published_version.unwrap_or(0) + 1;

        sqlx::query(
            r#"
            INSERT INTO workflow_published_versions (
                tenant_id,
                logical_name,
                version,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                published_by_subject,
                published_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .bind(next_version)
        .bind(draft.display_name)
        .bind(draft.description)
        .bind(draft.trigger_type)
        .bind(draft.trigger_entity_logical_name)
        .bind(draft.steps)
        .bind(draft.max_attempts)
        .bind(published_by)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to persist workflow '{}' published version {} for tenant '{}': {error}",
                logical_name, next_version, tenant_id
            ))
        })?;

        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            UPDATE workflow_definitions
            SET
                lifecycle_state = 'published',
                current_published_version = $3,
                updated_at = now()
            WHERE tenant_id = $1 AND logical_name = $2
            RETURNING
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                lifecycle_state,
                current_published_version
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .bind(next_version)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to mark workflow '{}' published for tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow publish transaction: {error}"
            ))
        })?;

        workflow_definition_from_row(row)
    }

    pub(super) async fn disable_workflow_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<WorkflowDefinition> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let existing = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                lifecycle_state,
                current_published_version
            FROM workflow_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load workflow '{}' for disable tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "workflow '{}' does not exist for tenant '{}'",
                logical_name, tenant_id
            ))
        })?;

        if existing.current_published_version.is_none() {
            return Err(AppError::Conflict(format!(
                "workflow '{}' does not have a published version to disable",
                logical_name
            )));
        }

        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            UPDATE workflow_definitions
            SET
                lifecycle_state = 'disabled',
                updated_at = now()
            WHERE tenant_id = $1 AND logical_name = $2
            RETURNING
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                steps,
                max_attempts,
                lifecycle_state,
                current_published_version
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to disable workflow '{}' for tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow disable transaction: {error}"
            ))
        })?;

        workflow_definition_from_row(row)
    }

    pub(super) async fn list_enabled_workflows_for_trigger_impl(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let (trigger_type, trigger_entity) = workflow_trigger_parts(trigger);

        let rows = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                versions.logical_name,
                versions.display_name,
                versions.description,
                versions.trigger_type,
                versions.trigger_entity_logical_name,
                versions.steps,
                versions.max_attempts,
                definitions.lifecycle_state,
                definitions.current_published_version
            FROM workflow_definitions definitions
            INNER JOIN workflow_published_versions versions
                ON versions.tenant_id = definitions.tenant_id
               AND versions.logical_name = definitions.logical_name
               AND versions.version = definitions.current_published_version
            WHERE definitions.tenant_id = $1
              AND definitions.lifecycle_state = 'published'
              AND versions.trigger_type = $2
              AND (
                    (versions.trigger_entity_logical_name IS NULL AND $3::TEXT IS NULL)
                    OR versions.trigger_entity_logical_name = $3
                  )
            ORDER BY versions.logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(trigger_type)
        .bind(trigger_entity)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list trigger workflows for tenant '{}': {error}",
                tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow trigger list transaction: {error}"
            ))
        })?;

        rows.into_iter().map(workflow_definition_from_row).collect()
    }
}
