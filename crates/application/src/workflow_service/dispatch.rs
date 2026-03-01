use super::*;

impl WorkflowService {
    async fn dispatch_trigger(
        &self,
        actor: &UserIdentity,
        trigger: WorkflowTrigger,
        mut payload: Value,
    ) -> AppResult<usize> {
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

        if let Some(payload_object) = payload.as_object_mut() {
            payload_object
                .entry("triggered_by".to_owned())
                .or_insert_with(|| Value::String(actor.subject().to_owned()));
        }

        let mut executed = 0;
        for workflow in workflows {
            let result = match self.execution_mode {
                WorkflowExecutionMode::Inline => {
                    self.execute_workflow_definition(&workflow_actor, &workflow, payload.clone())
                        .await
                }
                WorkflowExecutionMode::Queued => {
                    self.enqueue_workflow_definition(&workflow_actor, &workflow, payload.clone())
                        .await
                }
            };

            if result.is_ok() {
                executed += 1;
            }
        }

        Ok(executed)
    }

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
        record_data: &Value,
    ) -> AppResult<usize> {
        let mut payload = serde_json::json!({
            "entity_logical_name": entity_logical_name,
            "record_id": record_id,
            "id": record_id,
            "record": record_data,
            "data": record_data,
            "event": "created",
        });

        if let Some(payload_object) = payload.as_object_mut()
            && let Some(record_object) = record_data.as_object()
        {
            for (key, value) in record_object {
                payload_object
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: entity_logical_name.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches runtime record updated trigger across enabled workflows.
    pub async fn dispatch_runtime_record_updated(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        previous_data: Option<&Value>,
        current_data: &Value,
    ) -> AppResult<usize> {
        let payload = serde_json::json!({
            "entity_logical_name": entity_logical_name,
            "record_id": record_id,
            "id": record_id,
            "event": "updated",
            "previous": previous_data,
            "record": current_data,
            "data": current_data,
        });

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::RuntimeRecordUpdated {
                entity_logical_name: entity_logical_name.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches runtime record deleted trigger across enabled workflows.
    pub async fn dispatch_runtime_record_deleted(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        deleted_data: Option<&Value>,
    ) -> AppResult<usize> {
        let payload = serde_json::json!({
            "entity_logical_name": entity_logical_name,
            "record_id": record_id,
            "id": record_id,
            "event": "deleted",
            "record": deleted_data,
            "data": deleted_data,
        });

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::RuntimeRecordDeleted {
                entity_logical_name: entity_logical_name.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches schedule tick trigger across enabled workflows.
    pub async fn dispatch_schedule_tick(
        &self,
        actor: &UserIdentity,
        schedule_key: &str,
        payload: Option<Value>,
    ) -> AppResult<usize> {
        let event_payload = serde_json::json!({
            "schedule_key": schedule_key,
            "event": "schedule_tick",
            "data": payload,
        });

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::ScheduleTick {
                schedule_key: schedule_key.to_owned(),
            },
            event_payload,
        )
        .await
    }

    /// Retries one workflow step for an existing run.
    pub async fn retry_run_step(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        run_id: &str,
        step_path: &str,
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

        let run = self
            .repository
            .find_run(actor.tenant_id(), run_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow run '{}' does not exist for tenant '{}'",
                    run_id,
                    actor.tenant_id()
                ))
            })?;

        if run.workflow_logical_name != workflow.logical_name().as_str() {
            return Err(AppError::Validation(format!(
                "run '{}' does not belong to workflow '{}'",
                run_id, workflow_logical_name
            )));
        }

        self.retry_step_for_run(actor, &workflow, &run, step_path)
            .await
    }
}
