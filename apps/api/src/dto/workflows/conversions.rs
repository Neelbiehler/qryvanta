use qryvanta_application::{WorkflowRun, WorkflowRunAttempt, WorkflowRunStepTrace};
use qryvanta_core::AppError;
use qryvanta_domain::{
    WorkflowAction, WorkflowConditionOperator, WorkflowDefinition, WorkflowStep, WorkflowTrigger,
};

use serde_json::Value;

use super::types::{
    SaveWorkflowRequest, WorkflowConditionOperatorDto, WorkflowResponse,
    WorkflowRunAttemptResponse, WorkflowRunResponse, WorkflowRunStepTraceResponse, WorkflowStepDto,
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
            _ => {
                return Err(AppError::Validation(format!(
                    "unknown workflow trigger_type '{}'",
                    value.trigger_type
                )));
            }
        };

        let steps = value.steps.map(|workflow_steps| {
            workflow_steps
                .into_iter()
                .map(WorkflowStep::from)
                .collect::<Vec<WorkflowStep>>()
        });

        let action = if let Some(action_type) = value.action_type {
            workflow_action_from_transport(
                action_type.as_str(),
                value.action_entity_logical_name,
                value.action_payload,
            )?
        } else {
            let Some(first_action) = steps
                .as_ref()
                .and_then(|workflow_steps| first_action_from_steps(workflow_steps.as_slice()))
            else {
                return Err(AppError::Validation(
                    "workflow requires either action_type/action_payload or executable steps"
                        .to_owned(),
                ));
            };

            first_action
        };

        Ok(qryvanta_application::SaveWorkflowInput {
            logical_name: value.logical_name,
            display_name: value.display_name,
            description: value.description,
            trigger,
            action,
            steps,
            max_attempts: value.max_attempts.unwrap_or(3),
            is_enabled: value.is_enabled.unwrap_or(true),
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
        };

        let (action_type, action_entity_logical_name, action_payload) = match value.action() {
            WorkflowAction::LogMessage { message } => (
                "log_message".to_owned(),
                None,
                serde_json::json!({"message": message}),
            ),
            WorkflowAction::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => (
                "create_runtime_record".to_owned(),
                Some(entity_logical_name.clone()),
                data.clone(),
            ),
        };

        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            description: value.description().map(ToOwned::to_owned),
            trigger_type,
            trigger_entity_logical_name,
            action_type,
            action_entity_logical_name,
            action_payload,
            steps: value
                .effective_steps()
                .into_iter()
                .map(WorkflowStepDto::from)
                .collect(),
            max_attempts: value.max_attempts(),
            is_enabled: value.is_enabled(),
        }
    }
}

impl From<WorkflowRun> for WorkflowRunResponse {
    fn from(value: WorkflowRun) -> Self {
        Self {
            run_id: value.run_id,
            workflow_logical_name: value.workflow_logical_name,
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

fn workflow_action_from_transport(
    action_type: &str,
    action_entity_logical_name: Option<String>,
    action_payload: Option<Value>,
) -> Result<WorkflowAction, AppError> {
    match action_type {
        "log_message" => {
            let message = action_payload
                .as_ref()
                .and_then(Value::as_object)
                .and_then(|payload| payload.get("message"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    AppError::Validation(
                        "log_message action_payload must include string field 'message'".to_owned(),
                    )
                })?;

            Ok(WorkflowAction::LogMessage {
                message: message.to_owned(),
            })
        }
        "create_runtime_record" => Ok(WorkflowAction::CreateRuntimeRecord {
            entity_logical_name: action_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "action_entity_logical_name is required for create_runtime_record".to_owned(),
                )
            })?,
            data: action_payload.unwrap_or_else(|| serde_json::json!({})),
        }),
        _ => Err(AppError::Validation(format!(
            "unknown workflow action_type '{}'",
            action_type
        ))),
    }
}

fn first_action_from_steps(steps: &[WorkflowStep]) -> Option<WorkflowAction> {
    for step in steps {
        match step {
            WorkflowStep::LogMessage { message } => {
                return Some(WorkflowAction::LogMessage {
                    message: message.clone(),
                });
            }
            WorkflowStep::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => {
                return Some(WorkflowAction::CreateRuntimeRecord {
                    entity_logical_name: entity_logical_name.clone(),
                    data: data.clone(),
                });
            }
            WorkflowStep::Condition {
                then_steps,
                else_steps,
                ..
            } => {
                if let Some(action) = first_action_from_steps(then_steps.as_slice()) {
                    return Some(action);
                }

                if let Some(action) = first_action_from_steps(else_steps.as_slice()) {
                    return Some(action);
                }
            }
        }
    }

    None
}
