use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Workflow trigger source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowTrigger {
    /// Manually invoked trigger.
    Manual,
    /// Runtime record creation event trigger.
    RuntimeRecordCreated {
        /// Entity logical name that emits the trigger.
        entity_logical_name: String,
    },
}

impl WorkflowTrigger {
    /// Returns stable trigger type value.
    #[must_use]
    pub fn trigger_type(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::RuntimeRecordCreated { .. } => "runtime_record_created",
        }
    }

    /// Returns optional trigger entity scope.
    #[must_use]
    pub fn entity_logical_name(&self) -> Option<&str> {
        match self {
            Self::Manual => None,
            Self::RuntimeRecordCreated {
                entity_logical_name,
            } => Some(entity_logical_name.as_str()),
        }
    }
}

/// Workflow action behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowAction {
    /// Adds an informational message to execution output.
    LogMessage {
        /// Message captured as workflow action output.
        message: String,
    },
    /// Creates a runtime record in the target entity.
    CreateRuntimeRecord {
        /// Target runtime entity logical name.
        entity_logical_name: String,
        /// JSON object payload for record creation.
        data: Value,
    },
}

impl WorkflowAction {
    /// Returns stable action type value.
    #[must_use]
    pub fn action_type(&self) -> &'static str {
        match self {
            Self::LogMessage { .. } => "log_message",
            Self::CreateRuntimeRecord { .. } => "create_runtime_record",
        }
    }
}

/// Condition operator used by workflow branch steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowConditionOperator {
    /// True when selected payload value equals the configured value.
    Equals,
    /// True when selected payload value does not equal the configured value.
    NotEquals,
    /// True when selected payload path resolves to any value.
    Exists,
}

/// One workflow canvas step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowStep {
    /// Log message action step.
    LogMessage {
        /// Message to write to workflow output.
        message: String,
    },
    /// Runtime record creation step.
    CreateRuntimeRecord {
        /// Target runtime entity logical name.
        entity_logical_name: String,
        /// JSON object payload for record creation.
        data: Value,
    },
    /// Conditional branch that executes one branch of nested steps.
    Condition {
        /// Dot-separated payload path to evaluate.
        field_path: String,
        /// Condition operator.
        operator: WorkflowConditionOperator,
        /// Optional comparison value for equals/not_equals operators.
        value: Option<Value>,
        /// Optional display label for the success branch connector.
        then_label: Option<String>,
        /// Optional display label for the failure branch connector.
        else_label: Option<String>,
        /// Steps executed when the condition passes.
        then_steps: Vec<WorkflowStep>,
        /// Steps executed when the condition fails.
        else_steps: Vec<WorkflowStep>,
    },
}

impl WorkflowStep {
    /// Converts a legacy single action to one step.
    #[must_use]
    pub fn from_action(action: &WorkflowAction) -> Self {
        match action {
            WorkflowAction::LogMessage { message } => Self::LogMessage {
                message: message.clone(),
            },
            WorkflowAction::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => Self::CreateRuntimeRecord {
                entity_logical_name: entity_logical_name.clone(),
                data: data.clone(),
            },
        }
    }

    /// Converts a step to an action when the step is executable.
    #[must_use]
    pub fn as_action(&self) -> Option<WorkflowAction> {
        match self {
            Self::LogMessage { message } => Some(WorkflowAction::LogMessage {
                message: message.clone(),
            }),
            Self::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => Some(WorkflowAction::CreateRuntimeRecord {
                entity_logical_name: entity_logical_name.clone(),
                data: data.clone(),
            }),
            Self::Condition { .. } => None,
        }
    }
}

/// Tenant-scoped workflow definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    description: Option<String>,
    trigger: WorkflowTrigger,
    action: WorkflowAction,
    steps: Option<Vec<WorkflowStep>>,
    max_attempts: u16,
    is_enabled: bool,
}

/// Input payload used to construct a validated workflow definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDefinitionInput {
    /// Stable workflow logical name.
    pub logical_name: String,
    /// User-facing workflow display name.
    pub display_name: String,
    /// Optional workflow description.
    pub description: Option<String>,
    /// Trigger configuration.
    pub trigger: WorkflowTrigger,
    /// Legacy primary action configuration.
    pub action: WorkflowAction,
    /// Optional workflow canvas steps.
    pub steps: Option<Vec<WorkflowStep>>,
    /// Maximum execution attempts.
    pub max_attempts: u16,
    /// Enabled/disabled flag.
    pub is_enabled: bool,
}

impl WorkflowDefinition {
    /// Creates a validated workflow definition.
    pub fn new(input: WorkflowDefinitionInput) -> AppResult<Self> {
        let WorkflowDefinitionInput {
            logical_name,
            display_name,
            description,
            trigger,
            action,
            steps,
            max_attempts,
            is_enabled,
        } = input;

        if max_attempts == 0 {
            return Err(AppError::Validation(
                "max_attempts must be greater than zero".to_owned(),
            ));
        }

        if max_attempts > 10 {
            return Err(AppError::Validation(
                "max_attempts must be less than or equal to 10".to_owned(),
            ));
        }

        validate_trigger(&trigger)?;
        validate_action(&action)?;
        validate_steps(steps.as_deref())?;

        let description = description.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            description,
            trigger,
            action,
            steps,
            max_attempts,
            is_enabled,
        })
    }

    /// Returns workflow logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns workflow display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns optional workflow description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns workflow trigger configuration.
    #[must_use]
    pub fn trigger(&self) -> &WorkflowTrigger {
        &self.trigger
    }

    /// Returns legacy primary workflow action.
    #[must_use]
    pub fn action(&self) -> &WorkflowAction {
        &self.action
    }

    /// Returns optional workflow canvas step graph.
    #[must_use]
    pub fn steps(&self) -> Option<&[WorkflowStep]> {
        self.steps.as_deref()
    }

    /// Returns executable steps, falling back to the primary action.
    #[must_use]
    pub fn effective_steps(&self) -> Vec<WorkflowStep> {
        self.steps
            .clone()
            .unwrap_or_else(|| vec![WorkflowStep::from_action(self.action())])
    }

    /// Returns max retry attempts.
    #[must_use]
    pub fn max_attempts(&self) -> u16 {
        self.max_attempts
    }

    /// Returns whether workflow is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }
}

fn validate_trigger(trigger: &WorkflowTrigger) -> AppResult<()> {
    match trigger {
        WorkflowTrigger::Manual => Ok(()),
        WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name,
        } => {
            if entity_logical_name.trim().is_empty() {
                return Err(AppError::Validation(
                    "trigger entity_logical_name must not be empty".to_owned(),
                ));
            }

            Ok(())
        }
    }
}

fn validate_action(action: &WorkflowAction) -> AppResult<()> {
    match action {
        WorkflowAction::LogMessage { message } => {
            if message.trim().is_empty() {
                return Err(AppError::Validation(
                    "log_message action requires a non-empty message".to_owned(),
                ));
            }

            Ok(())
        }
        WorkflowAction::CreateRuntimeRecord {
            entity_logical_name,
            data,
        } => {
            if entity_logical_name.trim().is_empty() {
                return Err(AppError::Validation(
                    "create_runtime_record action requires entity_logical_name".to_owned(),
                ));
            }

            if !data.is_object() {
                return Err(AppError::Validation(
                    "create_runtime_record action data must be a JSON object".to_owned(),
                ));
            }

            Ok(())
        }
    }
}

fn validate_steps(steps: Option<&[WorkflowStep]>) -> AppResult<()> {
    let Some(steps) = steps else {
        return Ok(());
    };

    if steps.is_empty() {
        return Err(AppError::Validation(
            "workflow steps must include at least one step".to_owned(),
        ));
    }

    for step in steps {
        validate_step(step)?;
    }

    Ok(())
}

fn validate_step(step: &WorkflowStep) -> AppResult<()> {
    match step {
        WorkflowStep::LogMessage { message } => validate_action(&WorkflowAction::LogMessage {
            message: message.clone(),
        }),
        WorkflowStep::CreateRuntimeRecord {
            entity_logical_name,
            data,
        } => validate_action(&WorkflowAction::CreateRuntimeRecord {
            entity_logical_name: entity_logical_name.clone(),
            data: data.clone(),
        }),
        WorkflowStep::Condition {
            field_path,
            operator,
            value,
            then_label,
            else_label,
            then_steps,
            else_steps,
        } => {
            if field_path.trim().is_empty() {
                return Err(AppError::Validation(
                    "condition step field_path must not be empty".to_owned(),
                ));
            }

            match operator {
                WorkflowConditionOperator::Equals | WorkflowConditionOperator::NotEquals => {
                    if value.is_none() {
                        return Err(AppError::Validation(
                            "condition step equals/not_equals operator requires a value".to_owned(),
                        ));
                    }
                }
                WorkflowConditionOperator::Exists => {
                    if value.is_some() {
                        return Err(AppError::Validation(
                            "condition step exists operator does not accept a value".to_owned(),
                        ));
                    }
                }
            }

            if then_steps.is_empty() && else_steps.is_empty() {
                return Err(AppError::Validation(
                    "condition step must define at least one branch step".to_owned(),
                ));
            }

            if let Some(label) = then_label
                && label.trim().is_empty()
            {
                return Err(AppError::Validation(
                    "condition step then_label must not be empty when provided".to_owned(),
                ));
            }

            if let Some(label) = else_label
                && label.trim().is_empty()
            {
                return Err(AppError::Validation(
                    "condition step else_label must not be empty when provided".to_owned(),
                ));
            }

            for child_step in then_steps {
                validate_step(child_step)?;
            }

            for child_step in else_steps {
                validate_step(child_step)?;
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WorkflowAction, WorkflowConditionOperator, WorkflowDefinition, WorkflowDefinitionInput,
        WorkflowStep, WorkflowTrigger,
    };

    #[test]
    fn workflow_requires_positive_attempts() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "notify".to_owned(),
            display_name: "Notify".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            action: WorkflowAction::LogMessage {
                message: "hello".to_owned(),
            },
            steps: None,
            max_attempts: 0,
            is_enabled: true,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn create_runtime_record_action_requires_object_payload() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "create_contact".to_owned(),
            display_name: "Create Contact".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            action: WorkflowAction::CreateRuntimeRecord {
                entity_logical_name: "contact".to_owned(),
                data: serde_json::json!("invalid"),
            },
            steps: None,
            max_attempts: 3,
            is_enabled: true,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn condition_step_requires_at_least_one_branch_step() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "branching".to_owned(),
            display_name: "Branching".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            action: WorkflowAction::LogMessage {
                message: "legacy".to_owned(),
            },
            steps: Some(vec![WorkflowStep::Condition {
                field_path: "status".to_owned(),
                operator: WorkflowConditionOperator::Equals,
                value: Some(serde_json::json!("open")),
                then_label: None,
                else_label: None,
                then_steps: Vec::new(),
                else_steps: Vec::new(),
            }]),
            max_attempts: 3,
            is_enabled: true,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn workflow_accepts_canvas_steps() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "branching".to_owned(),
            display_name: "Branching".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            action: WorkflowAction::LogMessage {
                message: "legacy".to_owned(),
            },
            steps: Some(vec![WorkflowStep::Condition {
                field_path: "status".to_owned(),
                operator: WorkflowConditionOperator::Equals,
                value: Some(serde_json::json!("open")),
                then_label: Some("Matched".to_owned()),
                else_label: Some("Not Matched".to_owned()),
                then_steps: vec![WorkflowStep::LogMessage {
                    message: "open".to_owned(),
                }],
                else_steps: vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "task".to_owned(),
                    data: serde_json::json!({"title": "follow-up"}),
                }],
            }]),
            max_attempts: 3,
            is_enabled: true,
        });

        assert!(workflow.is_ok());
    }
}
