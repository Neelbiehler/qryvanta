use qryvanta_core::{AppError, AppResult, NonEmptyString, validate_secret_reference};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stable workflow release lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowLifecycleState {
    /// Draft exists but no published version is active.
    Draft,
    /// Published version is active and eligible for execution.
    Published,
    /// Published version exists but is disabled for execution.
    Disabled,
}

impl WorkflowLifecycleState {
    /// Returns stable storage value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Disabled => "disabled",
        }
    }

    /// Parses a stable storage value.
    pub fn parse(value: &str) -> AppResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "disabled" => Ok(Self::Disabled),
            _ => Err(AppError::Validation(format!(
                "unknown workflow lifecycle state '{value}'"
            ))),
        }
    }
}

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
    /// Runtime record update event trigger.
    RuntimeRecordUpdated {
        /// Entity logical name that emits the trigger.
        entity_logical_name: String,
    },
    /// Runtime record delete event trigger.
    RuntimeRecordDeleted {
        /// Entity logical name that emits the trigger.
        entity_logical_name: String,
    },
    /// Scheduler tick trigger.
    ScheduleTick {
        /// Schedule key for the tick source (for example: hourly, daily_utc_0900).
        schedule_key: String,
    },
    /// Inbound webhook trigger.
    WebhookReceived {
        /// Stable webhook key routed from the ingress endpoint.
        webhook_key: String,
    },
    /// Inbound form submission trigger.
    FormSubmitted {
        /// Stable form key routed from the ingress endpoint.
        form_key: String,
    },
    /// Inbound email trigger.
    InboundEmailReceived {
        /// Stable mailbox key routed from the ingress endpoint.
        mailbox_key: String,
    },
    /// Approval event trigger.
    ApprovalEventReceived {
        /// Stable approval key routed from the ingress endpoint.
        approval_key: String,
    },
}

impl WorkflowTrigger {
    /// Returns stable trigger type value.
    #[must_use]
    pub fn trigger_type(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::RuntimeRecordCreated { .. } => "runtime_record_created",
            Self::RuntimeRecordUpdated { .. } => "runtime_record_updated",
            Self::RuntimeRecordDeleted { .. } => "runtime_record_deleted",
            Self::ScheduleTick { .. } => "schedule_tick",
            Self::WebhookReceived { .. } => "webhook_received",
            Self::FormSubmitted { .. } => "form_submitted",
            Self::InboundEmailReceived { .. } => "inbound_email_received",
            Self::ApprovalEventReceived { .. } => "approval_event_received",
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
            Self::RuntimeRecordUpdated {
                entity_logical_name,
            } => Some(entity_logical_name.as_str()),
            Self::RuntimeRecordDeleted {
                entity_logical_name,
            } => Some(entity_logical_name.as_str()),
            Self::ScheduleTick { schedule_key } => Some(schedule_key.as_str()),
            Self::WebhookReceived { webhook_key } => Some(webhook_key.as_str()),
            Self::FormSubmitted { form_key } => Some(form_key.as_str()),
            Self::InboundEmailReceived { mailbox_key } => Some(mailbox_key.as_str()),
            Self::ApprovalEventReceived { approval_key } => Some(approval_key.as_str()),
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
    /// Runtime record update step.
    UpdateRuntimeRecord {
        /// Target runtime entity logical name.
        entity_logical_name: String,
        /// Target record identifier.
        record_id: String,
        /// JSON object payload for record update.
        data: Value,
    },
    /// Runtime record delete step.
    DeleteRuntimeRecord {
        /// Target runtime entity logical name.
        entity_logical_name: String,
        /// Target record identifier.
        record_id: String,
    },
    /// Outbound email delivery step.
    SendEmail {
        /// Recipient email address.
        to: String,
        /// Email subject line.
        subject: String,
        /// Plain-text message body.
        body: String,
        /// Optional HTML body for rich email rendering.
        html_body: Option<String>,
    },
    /// Outbound HTTP request action step.
    HttpRequest {
        /// HTTP method to use for the request.
        method: String,
        /// Destination URL.
        url: String,
        /// Optional HTTP header map.
        headers: Option<Value>,
        /// Optional HTTP header -> secret reference map.
        header_secret_refs: Option<Value>,
        /// Optional request body payload.
        body: Option<Value>,
    },
    /// Outbound webhook dispatch step.
    Webhook {
        /// Destination endpoint URL.
        endpoint: String,
        /// Event name attached to the webhook.
        event: String,
        /// Optional webhook headers.
        headers: Option<Value>,
        /// Optional webhook header -> secret reference map.
        header_secret_refs: Option<Value>,
        /// JSON object payload sent to the endpoint.
        payload: Value,
    },
    /// Assigns ownership of a target record.
    AssignOwner {
        /// Target runtime entity logical name.
        entity_logical_name: String,
        /// Target record identifier.
        record_id: String,
        /// Owner or queue identifier receiving the record.
        owner_id: String,
        /// Optional assignment reason.
        reason: Option<String>,
    },
    /// Creates an approval request for a target record.
    ApprovalRequest {
        /// Target runtime entity logical name.
        entity_logical_name: String,
        /// Target record identifier.
        record_id: String,
        /// Stable request type.
        request_type: String,
        /// Optional requested-by subject identifier.
        requested_by: Option<String>,
        /// Optional approver subject identifier.
        approver_id: Option<String>,
        /// Optional free-form reason.
        reason: Option<String>,
        /// Optional structured request payload.
        payload: Option<Value>,
    },
    /// In-worker delay step.
    Delay {
        /// Delay duration in milliseconds.
        duration_ms: u64,
        /// Optional operator-facing reason for the delay.
        reason: Option<String>,
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
    /// Returns stable step type value.
    #[must_use]
    pub fn step_type(&self) -> &'static str {
        match self {
            Self::LogMessage { .. } => "log_message",
            Self::CreateRuntimeRecord { .. } => "create_runtime_record",
            Self::UpdateRuntimeRecord { .. } => "update_runtime_record",
            Self::DeleteRuntimeRecord { .. } => "delete_runtime_record",
            Self::SendEmail { .. } => "send_email",
            Self::HttpRequest { .. } => "http_request",
            Self::Webhook { .. } => "webhook",
            Self::AssignOwner { .. } => "assign_owner",
            Self::ApprovalRequest { .. } => "approval_request",
            Self::Delay { .. } => "delay",
            Self::Condition { .. } => "condition",
        }
    }

    /// Returns whether this step or any nested branch contains executable work.
    #[must_use]
    pub fn contains_executable_step(&self) -> bool {
        match self {
            Self::LogMessage { .. }
            | Self::CreateRuntimeRecord { .. }
            | Self::UpdateRuntimeRecord { .. }
            | Self::DeleteRuntimeRecord { .. }
            | Self::SendEmail { .. }
            | Self::HttpRequest { .. }
            | Self::Webhook { .. }
            | Self::AssignOwner { .. }
            | Self::ApprovalRequest { .. }
            | Self::Delay { .. } => true,
            Self::Condition {
                then_steps,
                else_steps,
                ..
            } => {
                then_steps.iter().any(Self::contains_executable_step)
                    || else_steps.iter().any(Self::contains_executable_step)
            }
        }
    }

    /// Returns whether this step or any nested branch dispatches work to an external integration.
    #[must_use]
    pub fn contains_outbound_integration_step(&self) -> bool {
        match self {
            Self::SendEmail { .. } | Self::HttpRequest { .. } | Self::Webhook { .. } => true,
            Self::Condition {
                then_steps,
                else_steps,
                ..
            } => {
                then_steps
                    .iter()
                    .any(Self::contains_outbound_integration_step)
                    || else_steps
                        .iter()
                        .any(Self::contains_outbound_integration_step)
            }
            Self::LogMessage { .. }
            | Self::CreateRuntimeRecord { .. }
            | Self::UpdateRuntimeRecord { .. }
            | Self::DeleteRuntimeRecord { .. }
            | Self::AssignOwner { .. }
            | Self::ApprovalRequest { .. }
            | Self::Delay { .. } => false,
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
    steps: Vec<WorkflowStep>,
    max_attempts: u16,
    lifecycle_state: WorkflowLifecycleState,
    published_version: Option<i32>,
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
    /// Canonical workflow step graph.
    pub steps: Vec<WorkflowStep>,
    /// Maximum execution attempts.
    pub max_attempts: u16,
}

impl WorkflowDefinition {
    /// Creates a validated workflow definition.
    pub fn new(input: WorkflowDefinitionInput) -> AppResult<Self> {
        let WorkflowDefinitionInput {
            logical_name,
            display_name,
            description,
            trigger,
            steps,
            max_attempts,
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
        validate_steps(steps.as_slice())?;

        let description = description.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            description,
            trigger,
            steps,
            max_attempts,
            lifecycle_state: WorkflowLifecycleState::Draft,
            published_version: None,
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

    /// Returns workflow canvas step graph.
    #[must_use]
    pub fn steps(&self) -> &[WorkflowStep] {
        self.steps.as_slice()
    }

    /// Returns max retry attempts.
    #[must_use]
    pub fn max_attempts(&self) -> u16 {
        self.max_attempts
    }

    /// Returns workflow release lifecycle state.
    #[must_use]
    pub fn lifecycle_state(&self) -> WorkflowLifecycleState {
        self.lifecycle_state
    }

    /// Returns latest published version when one exists.
    #[must_use]
    pub fn published_version(&self) -> Option<i32> {
        self.published_version
    }

    /// Returns whether workflow is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(self.lifecycle_state, WorkflowLifecycleState::Published)
    }

    /// Returns whether any step dispatches to an outbound integration surface.
    #[must_use]
    pub fn contains_outbound_integration_steps(&self) -> bool {
        self.steps
            .iter()
            .any(WorkflowStep::contains_outbound_integration_step)
    }

    /// Rehydrates persisted publish metadata onto a validated workflow draft or snapshot.
    pub fn with_publish_state(
        mut self,
        lifecycle_state: WorkflowLifecycleState,
        published_version: Option<i32>,
    ) -> AppResult<Self> {
        if matches!(
            lifecycle_state,
            WorkflowLifecycleState::Published | WorkflowLifecycleState::Disabled
        ) && published_version.is_none()
        {
            return Err(AppError::Validation(
                "published workflow lifecycle requires a published version".to_owned(),
            ));
        }

        if matches!(lifecycle_state, WorkflowLifecycleState::Draft) && published_version.is_some() {
            return Err(AppError::Validation(
                "draft workflow lifecycle cannot carry a published version".to_owned(),
            ));
        }

        if let Some(version) = published_version
            && version <= 0
        {
            return Err(AppError::Validation(
                "workflow published version must be positive".to_owned(),
            ));
        }

        self.lifecycle_state = lifecycle_state;
        self.published_version = published_version;

        Ok(self)
    }
}

fn validate_trigger(trigger: &WorkflowTrigger) -> AppResult<()> {
    match trigger {
        WorkflowTrigger::Manual => Ok(()),
        WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name,
        }
        | WorkflowTrigger::RuntimeRecordUpdated {
            entity_logical_name,
        }
        | WorkflowTrigger::RuntimeRecordDeleted {
            entity_logical_name,
        } => {
            if entity_logical_name.trim().is_empty() {
                return Err(AppError::Validation(
                    "trigger entity_logical_name must not be empty".to_owned(),
                ));
            }

            Ok(())
        }
        WorkflowTrigger::ScheduleTick { schedule_key } => {
            if schedule_key.trim().is_empty() {
                return Err(AppError::Validation(
                    "schedule_tick trigger requires a non-empty schedule_key".to_owned(),
                ));
            }

            Ok(())
        }
        WorkflowTrigger::WebhookReceived { webhook_key } => {
            if webhook_key.trim().is_empty() {
                return Err(AppError::Validation(
                    "webhook_received trigger requires a non-empty webhook_key".to_owned(),
                ));
            }

            Ok(())
        }
        WorkflowTrigger::FormSubmitted { form_key } => {
            if form_key.trim().is_empty() {
                return Err(AppError::Validation(
                    "form_submitted trigger requires a non-empty form_key".to_owned(),
                ));
            }

            Ok(())
        }
        WorkflowTrigger::InboundEmailReceived { mailbox_key } => {
            if mailbox_key.trim().is_empty() {
                return Err(AppError::Validation(
                    "inbound_email_received trigger requires a non-empty mailbox_key".to_owned(),
                ));
            }

            Ok(())
        }
        WorkflowTrigger::ApprovalEventReceived { approval_key } => {
            if approval_key.trim().is_empty() {
                return Err(AppError::Validation(
                    "approval_event_received trigger requires a non-empty approval_key".to_owned(),
                ));
            }

            Ok(())
        }
    }
}

fn validate_log_message_step(message: &str) -> AppResult<()> {
    if message.trim().is_empty() {
        return Err(AppError::Validation(
            "log_message step requires a non-empty message".to_owned(),
        ));
    }

    Ok(())
}

/// Returns whether a header name is treated as credential-bearing for workflow governance.
#[must_use]
pub fn is_sensitive_workflow_header_name(header_name: &str) -> bool {
    matches!(
        header_name.trim().to_ascii_lowercase().as_str(),
        "authorization"
            | "proxy-authorization"
            | "cookie"
            | "set-cookie"
            | "x-api-key"
            | "api-key"
            | "x-auth-token"
            | "x-access-token"
            | "x-amz-security-token"
    )
}

/// Redacts credential-bearing headers before workflow trace persistence.
#[must_use]
pub fn redact_sensitive_workflow_headers(headers: Option<&Value>) -> Option<Value> {
    let Some(headers) = headers.and_then(Value::as_object) else {
        return headers.cloned();
    };

    let mut redacted = serde_json::Map::with_capacity(headers.len());
    for (key, value) in headers {
        if is_sensitive_workflow_header_name(key.as_str()) {
            redacted.insert(key.clone(), Value::String("[REDACTED]".to_owned()));
        } else {
            redacted.insert(key.clone(), value.clone());
        }
    }

    Some(Value::Object(redacted))
}

/// Redacts secret-reference-backed headers before workflow trace persistence.
#[must_use]
pub fn redact_workflow_header_secret_refs(header_secret_refs: Option<&Value>) -> Option<Value> {
    let Some(header_secret_refs) = header_secret_refs.and_then(Value::as_object) else {
        return header_secret_refs.cloned();
    };

    let mut redacted = serde_json::Map::with_capacity(header_secret_refs.len());
    for key in header_secret_refs.keys() {
        redacted.insert(key.clone(), Value::String("[SECRET_REF]".to_owned()));
    }

    Some(Value::Object(redacted))
}

fn validate_create_runtime_record_step(entity_logical_name: &str, data: &Value) -> AppResult<()> {
    if entity_logical_name.trim().is_empty() {
        return Err(AppError::Validation(
            "create_runtime_record step requires entity_logical_name".to_owned(),
        ));
    }

    if !data.is_object() {
        return Err(AppError::Validation(
            "create_runtime_record step data must be a JSON object".to_owned(),
        ));
    }

    Ok(())
}

fn validate_update_runtime_record_step(
    entity_logical_name: &str,
    record_id: &str,
    data: &Value,
) -> AppResult<()> {
    validate_record_target(entity_logical_name, record_id, "update_runtime_record")?;

    if !data.is_object() {
        return Err(AppError::Validation(
            "update_runtime_record step data must be a JSON object".to_owned(),
        ));
    }

    Ok(())
}

fn validate_delete_runtime_record_step(
    entity_logical_name: &str,
    record_id: &str,
) -> AppResult<()> {
    validate_record_target(entity_logical_name, record_id, "delete_runtime_record")
}

fn validate_record_target(
    entity_logical_name: &str,
    record_id: &str,
    step_type: &str,
) -> AppResult<()> {
    if entity_logical_name.trim().is_empty() {
        return Err(AppError::Validation(format!(
            "{step_type} step requires entity_logical_name"
        )));
    }

    if record_id.trim().is_empty() {
        return Err(AppError::Validation(format!(
            "{step_type} step requires record_id"
        )));
    }

    Ok(())
}

fn validate_send_email_step(
    to: &str,
    subject: &str,
    body: &str,
    html_body: Option<&str>,
) -> AppResult<()> {
    if to.trim().is_empty() {
        return Err(AppError::Validation(
            "send_email step requires a recipient address".to_owned(),
        ));
    }

    if subject.trim().is_empty() {
        return Err(AppError::Validation(
            "send_email step requires a non-empty subject".to_owned(),
        ));
    }

    if body.trim().is_empty() {
        return Err(AppError::Validation(
            "send_email step requires a non-empty body".to_owned(),
        ));
    }

    if let Some(value) = html_body
        && value.trim().is_empty()
    {
        return Err(AppError::Validation(
            "send_email step html_body must not be empty when provided".to_owned(),
        ));
    }

    Ok(())
}

fn validate_string_map<'a>(
    map_value: Option<&'a Value>,
    field_label: &str,
    step_type: &str,
) -> AppResult<Option<&'a serde_json::Map<String, Value>>> {
    let Some(map_value) = map_value else {
        return Ok(None);
    };

    let Some(map) = map_value.as_object() else {
        return Err(AppError::Validation(format!(
            "{step_type} step {field_label} must be a JSON object when provided"
        )));
    };

    for (key, value) in map {
        if key.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "{step_type} step {field_label} cannot contain an empty key"
            )));
        }

        if !value.is_string() {
            return Err(AppError::Validation(format!(
                "{step_type} step {field_label} entry '{key}' must be a string"
            )));
        }
    }

    Ok(Some(map))
}

fn validate_headers<'a>(
    headers: Option<&'a Value>,
    step_type: &str,
) -> AppResult<Option<&'a serde_json::Map<String, Value>>> {
    validate_string_map(headers, "headers", step_type)
}

fn validate_header_secret_refs<'a>(
    header_secret_refs: Option<&'a Value>,
    step_type: &str,
) -> AppResult<Option<&'a serde_json::Map<String, Value>>> {
    let Some(header_secret_refs) =
        validate_string_map(header_secret_refs, "header_secret_refs", step_type)?
    else {
        return Ok(None);
    };

    for (key, value) in header_secret_refs {
        let Some(reference) = value.as_str() else {
            return Err(AppError::Validation(format!(
                "{step_type} step header_secret_refs entry '{key}' must be a string"
            )));
        };

        validate_workflow_header_secret_reference(reference).map_err(|error| match error {
            AppError::Validation(message) => AppError::Validation(format!(
                "{step_type} step header_secret_refs entry '{key}' is invalid: {message}"
            )),
            other => other,
        })?;
    }

    Ok(Some(header_secret_refs))
}

fn validate_workflow_header_secret_reference(reference: &str) -> AppResult<()> {
    if let Some(inner_reference) = reference.strip_prefix("bearer+") {
        return validate_secret_reference(inner_reference);
    }

    if let Some(inner_reference) = reference.strip_prefix("basic+") {
        return validate_secret_reference(inner_reference);
    }

    validate_secret_reference(reference)
}

fn validate_duplicate_header_sources(
    headers: Option<&serde_json::Map<String, Value>>,
    header_secret_refs: Option<&serde_json::Map<String, Value>>,
    step_type: &str,
) -> AppResult<()> {
    let Some(headers) = headers else {
        return Ok(());
    };
    let Some(header_secret_refs) = header_secret_refs else {
        return Ok(());
    };

    for header_name in headers.keys() {
        if header_secret_refs
            .keys()
            .any(|secret_header_name| secret_header_name.eq_ignore_ascii_case(header_name))
        {
            return Err(AppError::Validation(format!(
                "{step_type} step cannot define header '{header_name}' in both headers and header_secret_refs"
            )));
        }
    }

    Ok(())
}

fn validate_http_request_step(
    method: &str,
    url: &str,
    headers: Option<&Value>,
    header_secret_refs: Option<&Value>,
) -> AppResult<()> {
    if method.trim().is_empty() {
        return Err(AppError::Validation(
            "http_request step requires a non-empty method".to_owned(),
        ));
    }

    if url.trim().is_empty() {
        return Err(AppError::Validation(
            "http_request step requires a non-empty url".to_owned(),
        ));
    }

    let headers = validate_headers(headers, "http_request")?;
    let header_secret_refs = validate_header_secret_refs(header_secret_refs, "http_request")?;
    validate_duplicate_header_sources(headers, header_secret_refs, "http_request")
}

fn validate_webhook_step(
    endpoint: &str,
    event: &str,
    headers: Option<&Value>,
    header_secret_refs: Option<&Value>,
    payload: &Value,
) -> AppResult<()> {
    if endpoint.trim().is_empty() {
        return Err(AppError::Validation(
            "webhook step requires a non-empty endpoint".to_owned(),
        ));
    }

    if event.trim().is_empty() {
        return Err(AppError::Validation(
            "webhook step requires a non-empty event".to_owned(),
        ));
    }

    if !payload.is_object() {
        return Err(AppError::Validation(
            "webhook step payload must be a JSON object".to_owned(),
        ));
    }

    let headers = validate_headers(headers, "webhook")?;
    let header_secret_refs = validate_header_secret_refs(header_secret_refs, "webhook")?;
    validate_duplicate_header_sources(headers, header_secret_refs, "webhook")
}

fn validate_assign_owner_step(
    entity_logical_name: &str,
    record_id: &str,
    owner_id: &str,
    reason: Option<&str>,
) -> AppResult<()> {
    validate_record_target(entity_logical_name, record_id, "assign_owner")?;

    if owner_id.trim().is_empty() {
        return Err(AppError::Validation(
            "assign_owner step requires owner_id".to_owned(),
        ));
    }

    if let Some(value) = reason
        && value.trim().is_empty()
    {
        return Err(AppError::Validation(
            "assign_owner step reason must not be empty when provided".to_owned(),
        ));
    }

    Ok(())
}

fn validate_approval_request_step(
    entity_logical_name: &str,
    record_id: &str,
    request_type: &str,
    requested_by: Option<&str>,
    approver_id: Option<&str>,
    reason: Option<&str>,
    payload: Option<&Value>,
) -> AppResult<()> {
    validate_record_target(entity_logical_name, record_id, "approval_request")?;

    if request_type.trim().is_empty() {
        return Err(AppError::Validation(
            "approval_request step requires request_type".to_owned(),
        ));
    }

    if let Some(value) = requested_by
        && value.trim().is_empty()
    {
        return Err(AppError::Validation(
            "approval_request step requested_by must not be empty when provided".to_owned(),
        ));
    }

    if let Some(value) = approver_id
        && value.trim().is_empty()
    {
        return Err(AppError::Validation(
            "approval_request step approver_id must not be empty when provided".to_owned(),
        ));
    }

    if let Some(value) = reason
        && value.trim().is_empty()
    {
        return Err(AppError::Validation(
            "approval_request step reason must not be empty when provided".to_owned(),
        ));
    }

    if let Some(payload) = payload
        && !payload.is_object()
    {
        return Err(AppError::Validation(
            "approval_request step payload must be a JSON object when provided".to_owned(),
        ));
    }

    Ok(())
}

fn validate_delay_step(duration_ms: u64, reason: Option<&str>) -> AppResult<()> {
    if duration_ms == 0 {
        return Err(AppError::Validation(
            "delay step requires duration_ms greater than zero".to_owned(),
        ));
    }

    if duration_ms > 86_400_000 {
        return Err(AppError::Validation(
            "delay step duration_ms must be less than or equal to 86400000".to_owned(),
        ));
    }

    if let Some(value) = reason
        && value.trim().is_empty()
    {
        return Err(AppError::Validation(
            "delay step reason must not be empty when provided".to_owned(),
        ));
    }

    Ok(())
}

fn validate_steps(steps: &[WorkflowStep]) -> AppResult<()> {
    if steps.is_empty() {
        return Err(AppError::Validation(
            "workflow steps must include at least one step".to_owned(),
        ));
    }

    if !steps.iter().any(WorkflowStep::contains_executable_step) {
        return Err(AppError::Validation(
            "workflow steps must include at least one executable step".to_owned(),
        ));
    }

    for step in steps {
        validate_step(step)?;
    }

    Ok(())
}

fn validate_step(step: &WorkflowStep) -> AppResult<()> {
    match step {
        WorkflowStep::LogMessage { message } => validate_log_message_step(message),
        WorkflowStep::CreateRuntimeRecord {
            entity_logical_name,
            data,
        } => validate_create_runtime_record_step(entity_logical_name, data),
        WorkflowStep::UpdateRuntimeRecord {
            entity_logical_name,
            record_id,
            data,
        } => validate_update_runtime_record_step(entity_logical_name, record_id, data),
        WorkflowStep::DeleteRuntimeRecord {
            entity_logical_name,
            record_id,
        } => validate_delete_runtime_record_step(entity_logical_name, record_id),
        WorkflowStep::SendEmail {
            to,
            subject,
            body,
            html_body,
        } => validate_send_email_step(to, subject, body, html_body.as_deref()),
        WorkflowStep::HttpRequest {
            method,
            url,
            headers,
            header_secret_refs,
            body: _,
        } => validate_http_request_step(method, url, headers.as_ref(), header_secret_refs.as_ref()),
        WorkflowStep::Webhook {
            endpoint,
            event,
            headers,
            header_secret_refs,
            payload,
        } => validate_webhook_step(
            endpoint,
            event,
            headers.as_ref(),
            header_secret_refs.as_ref(),
            payload,
        ),
        WorkflowStep::AssignOwner {
            entity_logical_name,
            record_id,
            owner_id,
            reason,
        } => {
            validate_assign_owner_step(entity_logical_name, record_id, owner_id, reason.as_deref())
        }
        WorkflowStep::ApprovalRequest {
            entity_logical_name,
            record_id,
            request_type,
            requested_by,
            approver_id,
            reason,
            payload,
        } => validate_approval_request_step(
            entity_logical_name,
            record_id,
            request_type,
            requested_by.as_deref(),
            approver_id.as_deref(),
            reason.as_deref(),
            payload.as_ref(),
        ),
        WorkflowStep::Delay {
            duration_ms,
            reason,
        } => validate_delay_step(*duration_ms, reason.as_deref()),
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
        WorkflowConditionOperator, WorkflowDefinition, WorkflowDefinitionInput, WorkflowStep,
        WorkflowTrigger, is_sensitive_workflow_header_name, redact_sensitive_workflow_headers,
        redact_workflow_header_secret_refs,
    };

    #[test]
    fn workflow_requires_positive_attempts() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "notify".to_owned(),
            display_name: "Notify".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::LogMessage {
                message: "hello".to_owned(),
            }],
            max_attempts: 0,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn create_runtime_record_step_requires_object_payload() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "create_contact".to_owned(),
            display_name: "Create Contact".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::CreateRuntimeRecord {
                entity_logical_name: "contact".to_owned(),
                data: serde_json::json!("invalid"),
            }],
            max_attempts: 3,
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
            steps: vec![WorkflowStep::Condition {
                field_path: "status".to_owned(),
                operator: WorkflowConditionOperator::Equals,
                value: Some(serde_json::json!("open")),
                then_label: None,
                else_label: None,
                then_steps: Vec::new(),
                else_steps: Vec::new(),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn send_email_step_requires_recipient() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "notify".to_owned(),
            display_name: "Notify".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::SendEmail {
                to: "   ".to_owned(),
                subject: "hello".to_owned(),
                body: "world".to_owned(),
                html_body: None,
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn http_request_step_requires_header_values_to_be_strings() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "dispatch_http".to_owned(),
            display_name: "Dispatch HTTP".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::HttpRequest {
                method: "POST".to_owned(),
                url: "https://example.org/hook".to_owned(),
                headers: Some(serde_json::json!({"x-attempt": 1})),
                header_secret_refs: None,
                body: None,
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn webhook_step_requires_object_payload() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "dispatch_webhook".to_owned(),
            display_name: "Dispatch Webhook".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::Webhook {
                endpoint: "https://example.org/webhook".to_owned(),
                event: "record.updated".to_owned(),
                headers: None,
                header_secret_refs: None,
                payload: serde_json::json!("invalid"),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn webhook_received_trigger_requires_key() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "incoming_webhook".to_owned(),
            display_name: "Incoming Webhook".to_owned(),
            description: None,
            trigger: WorkflowTrigger::WebhookReceived {
                webhook_key: "   ".to_owned(),
            },
            steps: vec![WorkflowStep::LogMessage {
                message: "received".to_owned(),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn form_submitted_trigger_requires_key() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "lead_form".to_owned(),
            display_name: "Lead Form".to_owned(),
            description: None,
            trigger: WorkflowTrigger::FormSubmitted {
                form_key: "   ".to_owned(),
            },
            steps: vec![WorkflowStep::LogMessage {
                message: "submitted".to_owned(),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn inbound_email_received_trigger_requires_key() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "support_mailbox".to_owned(),
            display_name: "Support Mailbox".to_owned(),
            description: None,
            trigger: WorkflowTrigger::InboundEmailReceived {
                mailbox_key: "   ".to_owned(),
            },
            steps: vec![WorkflowStep::LogMessage {
                message: "email".to_owned(),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn approval_event_received_trigger_requires_key() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "approval_events".to_owned(),
            display_name: "Approval Events".to_owned(),
            description: None,
            trigger: WorkflowTrigger::ApprovalEventReceived {
                approval_key: "   ".to_owned(),
            },
            steps: vec![WorkflowStep::LogMessage {
                message: "approval".to_owned(),
            }],
            max_attempts: 3,
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
            steps: vec![WorkflowStep::Condition {
                field_path: "status".to_owned(),
                operator: WorkflowConditionOperator::Equals,
                value: Some(serde_json::json!("open")),
                then_label: Some("Matched".to_owned()),
                else_label: Some("Not Matched".to_owned()),
                then_steps: vec![WorkflowStep::LogMessage {
                    message: "open".to_owned(),
                }],
                else_steps: vec![WorkflowStep::SendEmail {
                    to: "ops@example.com".to_owned(),
                    subject: "follow-up".to_owned(),
                    body: "check workflow output".to_owned(),
                    html_body: None,
                }],
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_ok());
    }

    #[test]
    fn update_runtime_record_step_requires_record_id() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "update_contact".to_owned(),
            display_name: "Update Contact".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::UpdateRuntimeRecord {
                entity_logical_name: "contact".to_owned(),
                record_id: " ".to_owned(),
                data: serde_json::json!({"name": "Alice"}),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn approval_request_payload_must_be_object_when_present() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "approval".to_owned(),
            display_name: "Approval".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::ApprovalRequest {
                entity_logical_name: "contact".to_owned(),
                record_id: "rec-1".to_owned(),
                request_type: "record_change".to_owned(),
                requested_by: None,
                approver_id: None,
                reason: None,
                payload: Some(serde_json::json!("bad")),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn delay_step_rejects_zero_duration() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "delay".to_owned(),
            display_name: "Delay".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::Delay {
                duration_ms: 0,
                reason: None,
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn workflow_detects_outbound_integration_steps_inside_conditions() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "notify_ops".to_owned(),
            display_name: "Notify Ops".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::Condition {
                field_path: "status".to_owned(),
                operator: WorkflowConditionOperator::Exists,
                value: None,
                then_label: None,
                else_label: None,
                then_steps: vec![WorkflowStep::HttpRequest {
                    method: "POST".to_owned(),
                    url: "https://example.com/hook".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    body: None,
                }],
                else_steps: vec![WorkflowStep::LogMessage {
                    message: "noop".to_owned(),
                }],
            }],
            max_attempts: 2,
        })
        .unwrap_or_else(|_| unreachable!());

        assert!(workflow.contains_outbound_integration_steps());
    }

    #[test]
    fn workflow_marks_sensitive_header_names() {
        assert!(is_sensitive_workflow_header_name("Authorization"));
        assert!(is_sensitive_workflow_header_name("x-api-key"));
        assert!(!is_sensitive_workflow_header_name("content-type"));
    }

    #[test]
    fn workflow_redacts_sensitive_headers() {
        let redacted = redact_sensitive_workflow_headers(Some(&serde_json::json!({
            "authorization": "Bearer top-secret",
            "content-type": "application/json",
        })))
        .unwrap_or_else(|| unreachable!());

        assert_eq!(redacted["authorization"], serde_json::json!("[REDACTED]"));
        assert_eq!(
            redacted["content-type"],
            serde_json::json!("application/json")
        );
    }

    #[test]
    fn workflow_redacts_header_secret_references() {
        let redacted = redact_workflow_header_secret_refs(Some(&serde_json::json!({
            "authorization": "op://vault/item/password",
        })))
        .unwrap_or_else(|| unreachable!());

        assert_eq!(redacted["authorization"], serde_json::json!("[SECRET_REF]"));
    }

    #[test]
    fn http_request_step_accepts_formatted_secret_header_refs() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "dispatch_http".to_owned(),
            display_name: "Dispatch HTTP".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::HttpRequest {
                method: "POST".to_owned(),
                url: "https://example.org/hook".to_owned(),
                headers: None,
                header_secret_refs: Some(serde_json::json!({
                    "authorization": "bearer+op://vault/item/password",
                    "x-basic-auth": "basic+aws-sm://prod/basic-creds"
                })),
                body: None,
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_ok());
    }

    #[test]
    fn http_request_step_rejects_duplicate_header_and_secret_ref_keys() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "dispatch_http".to_owned(),
            display_name: "Dispatch HTTP".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::HttpRequest {
                method: "POST".to_owned(),
                url: "https://example.org/hook".to_owned(),
                headers: Some(serde_json::json!({"authorization": "Bearer token"})),
                header_secret_refs: Some(serde_json::json!({
                    "Authorization": "op://vault/item/password"
                })),
                body: None,
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_err());
    }

    #[test]
    fn webhook_step_accepts_secret_header_refs() {
        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: "dispatch_webhook".to_owned(),
            display_name: "Dispatch Webhook".to_owned(),
            description: None,
            trigger: WorkflowTrigger::Manual,
            steps: vec![WorkflowStep::Webhook {
                endpoint: "https://example.org/webhook".to_owned(),
                event: "record.updated".to_owned(),
                headers: Some(serde_json::json!({"content-type": "application/json"})),
                header_secret_refs: Some(serde_json::json!({
                    "authorization": "op://vault/item/password"
                })),
                payload: serde_json::json!({"ok": true}),
            }],
            max_attempts: 3,
        });

        assert!(workflow.is_ok());
    }
}
