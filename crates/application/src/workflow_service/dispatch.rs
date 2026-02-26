use super::*;

impl WorkflowService {
    /// Executes a workflow by logical name using manual trigger context.
    pub async fn execute_workflow(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        trigger_payload: Value,
    ) -> AppResult<WorkflowRun> {
        self.require_workflow_manage(actor).await?;

        let workflow = self
            .repository
            .find_workflow(actor.tenant_id(), workflow_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow '{}' does not exist for tenant '{}'",
                    workflow_logical_name,
                    actor.tenant_id()
                ))
            })?;

        if !workflow.is_enabled() {
            return Err(AppError::Conflict(format!(
                "workflow '{}' is disabled",
                workflow.logical_name().as_str()
            )));
        }

        match self.execution_mode {
            WorkflowExecutionMode::Inline => {
                self.execute_workflow_definition(actor, &workflow, trigger_payload)
                    .await
            }
            WorkflowExecutionMode::Queued => {
                self.enqueue_workflow_definition(actor, &workflow, trigger_payload)
                    .await
            }
        }
    }

    /// Dispatches runtime record created trigger across enabled workflows.
    pub async fn dispatch_runtime_record_created(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<usize> {
        let trigger = WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name: entity_logical_name.to_owned(),
        };
        let workflows = self
            .repository
            .list_enabled_workflows_for_trigger(actor.tenant_id(), &trigger)
            .await?;

        if workflows.is_empty() {
            return Ok(0);
        }

        let workflow_actor = UserIdentity::new(
            "workflow-runtime",
            "workflow-runtime",
            None,
            actor.tenant_id(),
        );

        let mut executed = 0;
        for workflow in workflows {
            let payload = serde_json::json!({
                "entity_logical_name": entity_logical_name,
                "record_id": record_id,
                "triggered_by": actor.subject(),
            });

            let result = match self.execution_mode {
                WorkflowExecutionMode::Inline => {
                    self.execute_workflow_definition(&workflow_actor, &workflow, payload)
                        .await
                }
                WorkflowExecutionMode::Queued => {
                    self.enqueue_workflow_definition(&workflow_actor, &workflow, payload)
                        .await
                }
            };

            if result.is_ok() {
                executed += 1;
            }
        }

        Ok(executed)
    }
}
