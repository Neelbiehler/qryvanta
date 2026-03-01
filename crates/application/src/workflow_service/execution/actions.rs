use super::*;
use crate::workflow_ports::{WorkflowActionDispatchRequest, WorkflowActionDispatchType};

impl WorkflowService {
    pub(super) async fn execute_action(
        &self,
        actor: &UserIdentity,
        action: &WorkflowAction,
    ) -> AppResult<()> {
        match action {
            WorkflowAction::LogMessage { .. } => Ok(()),
            WorkflowAction::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => {
                self.runtime_record_service
                    .create_runtime_record_unchecked(
                        actor,
                        entity_logical_name.as_str(),
                        data.clone(),
                    )
                    .await?;
                Ok(())
            }
        }
    }

    pub(super) async fn execute_resolved_action(
        &self,
        actor: &UserIdentity,
        action: &WorkflowAction,
        context: WorkflowExecutionContext<'_>,
        step_path: &str,
    ) -> AppResult<()> {
        if let WorkflowAction::CreateRuntimeRecord {
            entity_logical_name,
            data,
        } = action
            && let Some(dispatch_type) = Self::integration_dispatch_type(entity_logical_name)
        {
            let Some(dispatcher) = self.action_dispatcher.clone() else {
                return Err(AppError::Validation(format!(
                    "workflow action '{}' requires configured integration dispatcher",
                    entity_logical_name
                )));
            };

            let request = WorkflowActionDispatchRequest {
                dispatch_type,
                run_id: context.run_id.to_owned(),
                step_path: step_path.to_owned(),
                idempotency_key: format!("{}:{}", context.run_id, step_path),
                payload: data.clone(),
            };

            dispatcher.dispatch_action(request).await?;
            return Ok(());
        }

        self.execute_action(actor, action).await
    }

    pub(super) fn integration_dispatch_type(
        entity_logical_name: &str,
    ) -> Option<WorkflowActionDispatchType> {
        match entity_logical_name {
            "integration_http_request" => Some(WorkflowActionDispatchType::HttpRequest),
            "webhook_dispatch" => Some(WorkflowActionDispatchType::Webhook),
            "email_outbox" => Some(WorkflowActionDispatchType::Email),
            _ => None,
        }
    }
}
