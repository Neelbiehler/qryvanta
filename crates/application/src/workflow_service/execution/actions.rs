use super::*;
use crate::workflow_ports::{WorkflowActionDispatchRequest, WorkflowActionDispatchType};
use serde_json::Value;

impl WorkflowService {
    async fn dispatch_external_action(
        &self,
        dispatch_type: WorkflowActionDispatchType,
        payload: Value,
        context: WorkflowExecutionContext<'_>,
        step_path: &str,
        step_type: &str,
    ) -> AppResult<()> {
        let Some(dispatcher) = self.action_dispatcher.clone() else {
            return Err(AppError::Validation(format!(
                "workflow action '{step_type}' requires configured integration dispatcher"
            )));
        };

        let request = WorkflowActionDispatchRequest {
            dispatch_type,
            run_id: context.run_id.to_owned(),
            step_path: step_path.to_owned(),
            idempotency_key: format!("{}:{}", context.run_id, step_path),
            payload,
        };

        dispatcher.dispatch_action(request).await
    }

    pub(super) async fn execute_action(
        &self,
        actor: &UserIdentity,
        step: &WorkflowStep,
    ) -> AppResult<()> {
        match step {
            WorkflowStep::LogMessage { .. } => Ok(()),
            WorkflowStep::CreateRuntimeRecord {
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
            WorkflowStep::UpdateRuntimeRecord {
                entity_logical_name,
                record_id,
                data,
            } => {
                self.runtime_record_service
                    .update_runtime_record_unchecked(
                        actor,
                        entity_logical_name.as_str(),
                        record_id.as_str(),
                        data.clone(),
                    )
                    .await?;
                Ok(())
            }
            WorkflowStep::DeleteRuntimeRecord {
                entity_logical_name,
                record_id,
            } => {
                self.runtime_record_service
                    .delete_runtime_record_unchecked(
                        actor,
                        entity_logical_name.as_str(),
                        record_id.as_str(),
                    )
                    .await?;
                Ok(())
            }
            WorkflowStep::SendEmail { .. }
            | WorkflowStep::HttpRequest { .. }
            | WorkflowStep::Webhook { .. }
            | WorkflowStep::Delay { .. } => Err(AppError::Validation(
                "native integration steps require execution context".to_owned(),
            )),
            WorkflowStep::AssignOwner {
                entity_logical_name,
                record_id,
                owner_id,
                reason,
            } => {
                self.runtime_record_service
                    .create_runtime_record_unchecked(
                        actor,
                        "record_assignment",
                        serde_json::json!({
                            "source_record_id": record_id,
                            "source_entity": entity_logical_name,
                            "owner_id": owner_id,
                            "reason": reason,
                        }),
                    )
                    .await?;
                Ok(())
            }
            WorkflowStep::ApprovalRequest {
                entity_logical_name,
                record_id,
                request_type,
                requested_by,
                approver_id,
                reason,
                payload,
            } => {
                self.runtime_record_service
                    .create_runtime_record_unchecked(
                        actor,
                        "approval_request",
                        serde_json::json!({
                            "request_type": request_type,
                            "source_record_id": record_id,
                            "source_entity": entity_logical_name,
                            "requested_by": requested_by.as_deref().unwrap_or(actor.subject()),
                            "approver_id": approver_id,
                            "reason": reason,
                            "status": "pending",
                            "payload": payload,
                        }),
                    )
                    .await?;
                Ok(())
            }
            WorkflowStep::Condition { .. } => Err(AppError::Validation(
                "condition step cannot execute as an action".to_owned(),
            )),
        }
    }

    pub(super) async fn execute_resolved_step(
        &self,
        actor: &UserIdentity,
        step: &WorkflowStep,
        context: WorkflowExecutionContext<'_>,
        step_path: &str,
    ) -> AppResult<()> {
        match step {
            WorkflowStep::SendEmail {
                to,
                subject,
                body,
                html_body,
            } => {
                return self
                    .dispatch_external_action(
                        WorkflowActionDispatchType::Email,
                        serde_json::json!({
                            "to": to,
                            "subject": subject,
                            "body": body,
                            "html_body": html_body,
                        }),
                        context,
                        step_path,
                        "send_email",
                    )
                    .await;
            }
            WorkflowStep::HttpRequest {
                method,
                url,
                headers,
                header_secret_refs,
                body,
            } => {
                return self
                    .dispatch_external_action(
                        WorkflowActionDispatchType::HttpRequest,
                        serde_json::json!({
                            "method": method,
                            "url": url,
                            "headers": headers,
                            "header_secret_refs": header_secret_refs,
                            "body": body,
                        }),
                        context,
                        step_path,
                        "http_request",
                    )
                    .await;
            }
            WorkflowStep::Webhook {
                endpoint,
                event,
                headers,
                header_secret_refs,
                payload,
            } => {
                return self
                    .dispatch_external_action(
                        WorkflowActionDispatchType::Webhook,
                        serde_json::json!({
                            "endpoint": endpoint,
                            "event": event,
                            "headers": headers,
                            "header_secret_refs": header_secret_refs,
                            "payload": payload,
                        }),
                        context,
                        step_path,
                        "webhook",
                    )
                    .await;
            }
            WorkflowStep::Delay { duration_ms, .. } => {
                let Some(delay_service) = self.delay_service.clone() else {
                    return Err(AppError::Validation(
                        "workflow action 'delay' requires configured delay service".to_owned(),
                    ));
                };

                delay_service.sleep(*duration_ms).await?;
                return Ok(());
            }
            WorkflowStep::LogMessage { .. }
            | WorkflowStep::CreateRuntimeRecord { .. }
            | WorkflowStep::UpdateRuntimeRecord { .. }
            | WorkflowStep::DeleteRuntimeRecord { .. }
            | WorkflowStep::AssignOwner { .. }
            | WorkflowStep::ApprovalRequest { .. }
            | WorkflowStep::Condition { .. } => {}
        }

        self.execute_action(actor, step).await
    }
}
