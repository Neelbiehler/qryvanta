use qryvanta_application::{WorkflowRun, WorkflowRunAttempt};
use qryvanta_core::AppError;
use qryvanta_domain::{
    WorkflowAction, WorkflowConditionOperator, WorkflowDefinition, WorkflowStep, WorkflowTrigger,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// Condition operators exposed through workflow DTOs.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workflow-condition-operator-dto.ts"
)]
pub enum WorkflowConditionOperatorDto {
    Equals,
    NotEquals,
    Exists,
}

/// One workflow canvas step shape used for API transport.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workflow-step-dto.ts"
)]
pub enum WorkflowStepDto {
    LogMessage {
        message: String,
    },
    CreateRuntimeRecord {
        entity_logical_name: String,
        #[ts(type = "Record<string, unknown>")]
        data: Value,
    },
    Condition {
        field_path: String,
        operator: WorkflowConditionOperatorDto,
        #[ts(type = "unknown | null")]
        value: Option<Value>,
        then_label: Option<String>,
        else_label: Option<String>,
        then_steps: Vec<WorkflowStepDto>,
        else_steps: Vec<WorkflowStepDto>,
    },
}

/// Incoming payload for workflow create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/save-workflow-request.ts"
)]
pub struct SaveWorkflowRequest {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_entity_logical_name: Option<String>,
    pub action_type: Option<String>,
    pub action_entity_logical_name: Option<String>,
    #[ts(type = "Record<string, unknown> | null")]
    pub action_payload: Option<Value>,
    pub steps: Option<Vec<WorkflowStepDto>>,
    pub max_attempts: Option<u16>,
    pub is_enabled: Option<bool>,
}

/// Incoming payload for manual workflow execution.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/execute-workflow-request.ts"
)]
pub struct ExecuteWorkflowRequest {
    #[ts(type = "Record<string, unknown>")]
    pub trigger_payload: Value,
}

/// API representation of one workflow definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workflow-response.ts"
)]
pub struct WorkflowResponse {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_entity_logical_name: Option<String>,
    pub action_type: String,
    pub action_entity_logical_name: Option<String>,
    #[ts(type = "Record<string, unknown>")]
    pub action_payload: Value,
    pub steps: Vec<WorkflowStepDto>,
    pub max_attempts: u16,
    pub is_enabled: bool,
}

/// API representation of one workflow run.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workflow-run-response.ts"
)]
pub struct WorkflowRunResponse {
    pub run_id: String,
    pub workflow_logical_name: String,
    pub trigger_type: String,
    pub trigger_entity_logical_name: Option<String>,
    #[ts(type = "Record<string, unknown>")]
    pub trigger_payload: Value,
    pub status: String,
    pub attempts: i32,
    pub dead_letter_reason: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

/// API representation of one workflow run attempt.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workflow-run-attempt-response.ts"
)]
pub struct WorkflowRunAttemptResponse {
    pub run_id: String,
    pub attempt_number: i32,
    pub status: String,
    pub error_message: Option<String>,
    pub executed_at: String,
}

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
