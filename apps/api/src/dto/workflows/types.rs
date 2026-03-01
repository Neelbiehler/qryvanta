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

/// Incoming payload for dispatching a schedule tick trigger.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/dispatch-schedule-trigger-request.ts"
)]
pub struct DispatchScheduleTriggerRequest {
    pub schedule_key: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub payload: Option<Value>,
}

/// Incoming payload for retrying one step of an existing run.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/retry-workflow-step-request.ts"
)]
pub struct RetryWorkflowStepRequest {
    pub step_path: String,
    pub strategy: RetryWorkflowStepStrategyDto,
    pub backoff_ms: Option<u32>,
}

/// Retry strategy options for step-level retry actions.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/retry-workflow-step-strategy-dto.ts"
)]
pub enum RetryWorkflowStepStrategyDto {
    Immediate,
    Backoff,
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
    pub step_traces: Vec<WorkflowRunStepTraceResponse>,
}

/// API representation of one workflow step execution trace.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workflow-run-step-trace-response.ts"
)]
pub struct WorkflowRunStepTraceResponse {
    pub step_path: String,
    pub step_type: String,
    pub status: String,
    #[ts(type = "Record<string, unknown>")]
    pub input_payload: Value,
    #[ts(type = "Record<string, unknown>")]
    pub output_payload: Value,
    pub error_message: Option<String>,
    pub duration_ms: Option<u64>,
}
