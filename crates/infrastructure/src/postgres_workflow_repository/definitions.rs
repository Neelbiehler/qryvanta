use super::*;

impl PostgresWorkflowRepository {
    pub(super) async fn save_workflow_impl(
        &self,
        tenant_id: TenantId,
        workflow: WorkflowDefinition,
    ) -> AppResult<()> {
        let (trigger_type, trigger_entity) = workflow_trigger_parts(workflow.trigger());
        let (action_type, action_entity, action_payload) = workflow_action_parts(workflow.action());
        let action_steps = workflow_steps_to_json(workflow.steps())?;

        sqlx::query(
            r#"
            INSERT INTO workflow_definitions (
                tenant_id,
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, now())
            ON CONFLICT (tenant_id, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                description = EXCLUDED.description,
                trigger_type = EXCLUDED.trigger_type,
                trigger_entity_logical_name = EXCLUDED.trigger_entity_logical_name,
                action_type = EXCLUDED.action_type,
                action_entity_logical_name = EXCLUDED.action_entity_logical_name,
                action_payload = EXCLUDED.action_payload,
                action_steps = EXCLUDED.action_steps,
                max_attempts = EXCLUDED.max_attempts,
                is_enabled = EXCLUDED.is_enabled,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(workflow.logical_name().as_str())
        .bind(workflow.display_name().as_str())
        .bind(workflow.description())
        .bind(trigger_type)
        .bind(trigger_entity)
        .bind(action_type)
        .bind(action_entity)
        .bind(action_payload)
        .bind(action_steps)
        .bind(i16::try_from(workflow.max_attempts()).map_err(|error| {
            AppError::Validation(format!("invalid workflow max_attempts value: {error}"))
        })?)
        .bind(workflow.is_enabled())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save workflow '{}' for tenant '{}': {error}",
                workflow.logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_workflows_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        let rows = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled
            FROM workflow_definitions
            WHERE tenant_id = $1
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list workflows for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter().map(workflow_definition_from_row).collect()
    }

    pub(super) async fn find_workflow_impl(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled
            FROM workflow_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find workflow '{}' for tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;

        row.map(workflow_definition_from_row).transpose()
    }

    pub(super) async fn list_enabled_workflows_for_trigger_impl(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        let (trigger_type, trigger_entity) = workflow_trigger_parts(trigger);

        let rows = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled
            FROM workflow_definitions
            WHERE tenant_id = $1
              AND is_enabled = true
              AND trigger_type = $2
              AND (
                    (trigger_entity_logical_name IS NULL AND $3::TEXT IS NULL)
                    OR trigger_entity_logical_name = $3
                  )
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(trigger_type)
        .bind(trigger_entity)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list trigger workflows for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter().map(workflow_definition_from_row).collect()
    }
}
