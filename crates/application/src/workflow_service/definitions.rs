use super::*;

impl WorkflowService {
    /// Saves one workflow definition.
    pub async fn save_workflow(
        &self,
        actor: &UserIdentity,
        input: SaveWorkflowInput,
    ) -> AppResult<WorkflowDefinition> {
        self.require_workflow_manage(actor).await?;

        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: input.logical_name,
            display_name: input.display_name,
            description: input.description,
            trigger: input.trigger,
            action: input.action,
            steps: input.steps,
            max_attempts: input.max_attempts,
            is_enabled: input.is_enabled,
        })?;

        self.repository
            .save_workflow(actor.tenant_id(), workflow.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::WorkflowSaved,
                resource_type: "workflow_definition".to_owned(),
                resource_id: workflow.logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "saved workflow '{}' trigger '{}' action '{}' with {} step(s)",
                    workflow.logical_name().as_str(),
                    workflow.trigger().trigger_type(),
                    workflow.action().action_type(),
                    workflow.effective_steps().len()
                )),
            })
            .await?;

        Ok(workflow)
    }

    /// Lists workflow definitions.
    pub async fn list_workflows(&self, actor: &UserIdentity) -> AppResult<Vec<WorkflowDefinition>> {
        self.require_workflow_read(actor).await?;
        self.repository.list_workflows(actor.tenant_id()).await
    }

    /// Lists workflow runs for operational traceability.
    pub async fn list_runs(
        &self,
        actor: &UserIdentity,
        query: WorkflowRunListQuery,
    ) -> AppResult<Vec<WorkflowRun>> {
        self.require_workflow_read(actor).await?;
        self.repository.list_runs(actor.tenant_id(), query).await
    }

    /// Lists workflow run attempts for one run.
    pub async fn list_run_attempts(
        &self,
        actor: &UserIdentity,
        run_id: &str,
    ) -> AppResult<Vec<WorkflowRunAttempt>> {
        self.require_workflow_read(actor).await?;
        self.repository
            .list_run_attempts(actor.tenant_id(), run_id)
            .await
    }

    pub(super) async fn require_workflow_manage(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await
    }

    pub(super) async fn require_workflow_read(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await
    }
}
