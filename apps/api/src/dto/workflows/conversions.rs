use qryvanta_application::{
    WorkflowRun, WorkflowRunAttempt, WorkflowRunReplay, WorkflowRunReplayTimelineEvent,
    WorkflowRunStepTrace,
};
use qryvanta_core::AppError;
use qryvanta_domain::{
    WorkflowConditionOperator, WorkflowDefinition, WorkflowLifecycleState, WorkflowStep,
    WorkflowTrigger,
};

use super::types::{
    SaveWorkflowRequest, WorkflowConditionOperatorDto, WorkflowResponse,
    WorkflowRunAttemptResponse, WorkflowRunReplayResponse, WorkflowRunReplayTimelineEventResponse,
    WorkflowRunResponse, WorkflowRunStepTraceResponse, WorkflowStepDto,
};

impl TryFrom<SaveWorkflowRequest> for qryvanta_application::SaveWorkflowInput {
    type Error = qryvanta_core::AppError;

    fn try_from(value: SaveWorkflowRequest) -> Result<Self, Self::Error> {
        let trigger = match value.trigger_type.as_str() {
            "manual" => WorkflowTrigger::Manual,
            "runtime_record_created" => WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for runtime_record_created"
                            .to_owned(),
                    )
                })?,
            },
            "runtime_record_updated" => WorkflowTrigger::RuntimeRecordUpdated {
                entity_logical_name: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for runtime_record_updated"
                            .to_owned(),
                    )
                })?,
            },
            "runtime_record_deleted" => WorkflowTrigger::RuntimeRecordDeleted {
                entity_logical_name: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for runtime_record_deleted"
                            .to_owned(),
                    )
                })?,
            },
            "schedule_tick" => WorkflowTrigger::ScheduleTick {
                schedule_key: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for schedule_tick".to_owned(),
                    )
                })?,
            },
            "webhook_received" => WorkflowTrigger::WebhookReceived {
                webhook_key: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for webhook_received".to_owned(),
                    )
                })?,
            },
            "form_submitted" => WorkflowTrigger::FormSubmitted {
                form_key: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for form_submitted".to_owned(),
                    )
                })?,
            },
            "inbound_email_received" => WorkflowTrigger::InboundEmailReceived {
                mailbox_key: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for inbound_email_received"
                            .to_owned(),
                    )
                })?,
            },
            "approval_event_received" => WorkflowTrigger::ApprovalEventReceived {
                approval_key: value.trigger_entity_logical_name.ok_or_else(|| {
                    AppError::Validation(
                        "trigger_entity_logical_name is required for approval_event_received"
                            .to_owned(),
                    )
                })?,
            },
            _ => {
                return Err(AppError::Validation(format!(
                    "unknown workflow trigger_type '{}'",
                    value.trigger_type
                )));
            }
        };

        let steps = value
            .steps
            .into_iter()
            .map(WorkflowStep::from)
            .collect::<Vec<WorkflowStep>>();

        Ok(qryvanta_application::SaveWorkflowInput {
            logical_name: value.logical_name,
            display_name: value.display_name,
            description: value.description,
            trigger,
            steps,
            max_attempts: value.max_attempts.unwrap_or(3),
            is_enabled: true,
        })
    }
}

impl From<WorkflowDefinition> for WorkflowResponse {
    fn from(value: WorkflowDefinition) -> Self {
        let (trigger_type, trigger_entity_logical_name) = match value.trigger() {
            WorkflowTrigger::Manual => ("manual".to_owned(), None),
            WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name,
            } => (
                "runtime_record_created".to_owned(),
                Some(entity_logical_name.clone()),
            ),
            WorkflowTrigger::RuntimeRecordUpdated {
                entity_logical_name,
            } => (
                "runtime_record_updated".to_owned(),
                Some(entity_logical_name.clone()),
            ),
            WorkflowTrigger::RuntimeRecordDeleted {
                entity_logical_name,
            } => (
                "runtime_record_deleted".to_owned(),
                Some(entity_logical_name.clone()),
            ),
            WorkflowTrigger::ScheduleTick { schedule_key } => {
                ("schedule_tick".to_owned(), Some(schedule_key.clone()))
            }
            WorkflowTrigger::WebhookReceived { webhook_key } => {
                ("webhook_received".to_owned(), Some(webhook_key.clone()))
            }
            WorkflowTrigger::FormSubmitted { form_key } => {
                ("form_submitted".to_owned(), Some(form_key.clone()))
            }
            WorkflowTrigger::InboundEmailReceived { mailbox_key } => (
                "inbound_email_received".to_owned(),
                Some(mailbox_key.clone()),
            ),
            WorkflowTrigger::ApprovalEventReceived { approval_key } => (
                "approval_event_received".to_owned(),
                Some(approval_key.clone()),
            ),
        };

        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            description: value.description().map(ToOwned::to_owned),
            trigger_type,
            trigger_entity_logical_name,
            steps: value
                .steps()
                .iter()
                .cloned()
                .map(WorkflowStepDto::from)
                .collect(),
            max_attempts: value.max_attempts(),
            lifecycle_state: workflow_lifecycle_state_str(value.lifecycle_state()).to_owned(),
            published_version: value.published_version(),
            is_enabled: value.is_enabled(),
        }
    }
}

impl From<WorkflowRun> for WorkflowRunResponse {
    fn from(value: WorkflowRun) -> Self {
        Self {
            run_id: value.run_id,
            workflow_logical_name: value.workflow_logical_name,
            workflow_version: value.workflow_version,
            trigger_type: value.trigger_type,
            trigger_entity_logical_name: value.trigger_entity_logical_name,
            trigger_payload: value.trigger_payload,
            status: value.status.as_str().to_owned(),
            attempts: value.attempts,
            dead_letter_reason: value.dead_letter_reason,
            started_at: value.started_at.to_rfc3339(),
            finished_at: value.finished_at.map(|timestamp| timestamp.to_rfc3339()),
        }
    }
}

fn workflow_lifecycle_state_str(state: WorkflowLifecycleState) -> &'static str {
    match state {
        WorkflowLifecycleState::Draft => "draft",
        WorkflowLifecycleState::Published => "published",
        WorkflowLifecycleState::Disabled => "disabled",
    }
}

impl From<WorkflowRunAttempt> for WorkflowRunAttemptResponse {
    fn from(value: WorkflowRunAttempt) -> Self {
        Self {
            run_id: value.run_id,
            attempt_number: value.attempt_number,
            status: value.status.as_str().to_owned(),
            error_message: value.error_message,
            executed_at: value.executed_at.to_rfc3339(),
            step_traces: value
                .step_traces
                .into_iter()
                .map(WorkflowRunStepTraceResponse::from)
                .collect(),
        }
    }
}

impl From<WorkflowRunStepTrace> for WorkflowRunStepTraceResponse {
    fn from(value: WorkflowRunStepTrace) -> Self {
        Self {
            step_path: value.step_path,
            step_type: value.step_type,
            status: value.status,
            input_payload: value.input_payload,
            output_payload: value.output_payload,
            error_message: value.error_message,
            duration_ms: value.duration_ms,
        }
    }
}

impl From<WorkflowRunReplayTimelineEvent> for WorkflowRunReplayTimelineEventResponse {
    fn from(value: WorkflowRunReplayTimelineEvent) -> Self {
        Self {
            sequence: value.sequence,
            attempt_number: value.attempt_number,
            attempt_status: value.attempt_status.as_str().to_owned(),
            attempt_executed_at: value.attempt_executed_at.to_rfc3339(),
            step_path: value.step_path,
            step_type: value.step_type,
            status: value.status,
            input_payload: value.input_payload,
            output_payload: value.output_payload,
            error_message: value.error_message,
            duration_ms: value.duration_ms,
        }
    }
}

impl From<WorkflowRunReplay> for WorkflowRunReplayResponse {
    fn from(value: WorkflowRunReplay) -> Self {
        Self {
            run: WorkflowRunResponse::from(value.run),
            attempts: value
                .attempts
                .into_iter()
                .map(WorkflowRunAttemptResponse::from)
                .collect(),
            timeline: value
                .timeline
                .into_iter()
                .map(WorkflowRunReplayTimelineEventResponse::from)
                .collect(),
            checksum_sha256: value.checksum_sha256,
        }
    }
}

impl From<WorkflowConditionOperatorDto> for WorkflowConditionOperator {
    fn from(value: WorkflowConditionOperatorDto) -> Self {
        match value {
            WorkflowConditionOperatorDto::Equals => Self::Equals,
            WorkflowConditionOperatorDto::NotEquals => Self::NotEquals,
            WorkflowConditionOperatorDto::Exists => Self::Exists,
        }
    }
}

impl From<WorkflowConditionOperator> for WorkflowConditionOperatorDto {
    fn from(value: WorkflowConditionOperator) -> Self {
        match value {
            WorkflowConditionOperator::Equals => Self::Equals,
            WorkflowConditionOperator::NotEquals => Self::NotEquals,
            WorkflowConditionOperator::Exists => Self::Exists,
        }
    }
}

impl From<WorkflowStepDto> for WorkflowStep {
    fn from(value: WorkflowStepDto) -> Self {
        match value {
            WorkflowStepDto::LogMessage { message } => Self::LogMessage { message },
            WorkflowStepDto::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => Self::CreateRuntimeRecord {
                entity_logical_name,
                data,
            },
            WorkflowStepDto::UpdateRuntimeRecord {
                entity_logical_name,
                record_id,
                data,
            } => Self::UpdateRuntimeRecord {
                entity_logical_name,
                record_id,
                data,
            },
            WorkflowStepDto::DeleteRuntimeRecord {
                entity_logical_name,
                record_id,
            } => Self::DeleteRuntimeRecord {
                entity_logical_name,
                record_id,
            },
            WorkflowStepDto::SendEmail {
                to,
                subject,
                body,
                html_body,
            } => Self::SendEmail {
                to,
                subject,
                body,
                html_body,
            },
            WorkflowStepDto::HttpRequest {
                method,
                url,
                headers,
                header_secret_refs,
                body,
            } => Self::HttpRequest {
                method,
                url,
                headers,
                header_secret_refs,
                body,
            },
            WorkflowStepDto::Webhook {
                endpoint,
                event,
                headers,
                header_secret_refs,
                payload,
            } => Self::Webhook {
                endpoint,
                event,
                headers,
                header_secret_refs,
                payload,
            },
            WorkflowStepDto::AssignOwner {
                entity_logical_name,
                record_id,
                owner_id,
                reason,
            } => Self::AssignOwner {
                entity_logical_name,
                record_id,
                owner_id,
                reason,
            },
            WorkflowStepDto::ApprovalRequest {
                entity_logical_name,
                record_id,
                request_type,
                requested_by,
                approver_id,
                reason,
                payload,
            } => Self::ApprovalRequest {
                entity_logical_name,
                record_id,
                request_type,
                requested_by,
                approver_id,
                reason,
                payload,
            },
            WorkflowStepDto::Delay {
                duration_ms,
                reason,
            } => Self::Delay {
                duration_ms,
                reason,
            },
            WorkflowStepDto::Condition {
                field_path,
                operator,
                value,
                then_label,
                else_label,
                then_steps,
                else_steps,
            } => Self::Condition {
                field_path,
                operator: WorkflowConditionOperator::from(operator),
                value,
                then_label,
                else_label,
                then_steps: then_steps.into_iter().map(Self::from).collect(),
                else_steps: else_steps.into_iter().map(Self::from).collect(),
            },
        }
    }
}

impl From<WorkflowStep> for WorkflowStepDto {
    fn from(value: WorkflowStep) -> Self {
        match value {
            WorkflowStep::LogMessage { message } => Self::LogMessage { message },
            WorkflowStep::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => Self::CreateRuntimeRecord {
                entity_logical_name,
                data,
            },
            WorkflowStep::UpdateRuntimeRecord {
                entity_logical_name,
                record_id,
                data,
            } => Self::UpdateRuntimeRecord {
                entity_logical_name,
                record_id,
                data,
            },
            WorkflowStep::DeleteRuntimeRecord {
                entity_logical_name,
                record_id,
            } => Self::DeleteRuntimeRecord {
                entity_logical_name,
                record_id,
            },
            WorkflowStep::SendEmail {
                to,
                subject,
                body,
                html_body,
            } => Self::SendEmail {
                to,
                subject,
                body,
                html_body,
            },
            WorkflowStep::HttpRequest {
                method,
                url,
                headers,
                header_secret_refs,
                body,
            } => Self::HttpRequest {
                method,
                url,
                headers,
                header_secret_refs,
                body,
            },
            WorkflowStep::Webhook {
                endpoint,
                event,
                headers,
                header_secret_refs,
                payload,
            } => Self::Webhook {
                endpoint,
                event,
                headers,
                header_secret_refs,
                payload,
            },
            WorkflowStep::AssignOwner {
                entity_logical_name,
                record_id,
                owner_id,
                reason,
            } => Self::AssignOwner {
                entity_logical_name,
                record_id,
                owner_id,
                reason,
            },
            WorkflowStep::ApprovalRequest {
                entity_logical_name,
                record_id,
                request_type,
                requested_by,
                approver_id,
                reason,
                payload,
            } => Self::ApprovalRequest {
                entity_logical_name,
                record_id,
                request_type,
                requested_by,
                approver_id,
                reason,
                payload,
            },
            WorkflowStep::Delay {
                duration_ms,
                reason,
            } => Self::Delay {
                duration_ms,
                reason,
            },
            WorkflowStep::Condition {
                field_path,
                operator,
                value,
                then_label,
                else_label,
                then_steps,
                else_steps,
            } => Self::Condition {
                field_path,
                operator: WorkflowConditionOperatorDto::from(operator),
                value,
                then_label,
                else_label,
                then_steps: then_steps.into_iter().map(Self::from).collect(),
                else_steps: else_steps.into_iter().map(Self::from).collect(),
            },
        }
    }
}
