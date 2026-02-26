use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Supported business-rule evaluation scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessRuleScope {
    /// Rule runs for all entity operations.
    Entity,
    /// Rule runs in one form context.
    Form,
}

impl BusinessRuleScope {
    /// Returns stable storage value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Form => "form",
        }
    }
}

/// Supported condition operators for business rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessRuleOperator {
    /// Equality comparison.
    Eq,
    /// Inequality comparison.
    Neq,
    /// Greater-than comparison.
    Gt,
    /// Greater-than-or-equal comparison.
    Gte,
    /// Less-than comparison.
    Lt,
    /// Less-than-or-equal comparison.
    Lte,
    /// Case-insensitive contains comparison.
    Contains,
}

/// Supported action kinds for business rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessRuleActionType {
    /// Show one field.
    ShowField,
    /// Hide one field.
    HideField,
    /// Mark one field required.
    SetRequired,
    /// Mark one field optional.
    SetOptional,
    /// Set one field default value.
    SetDefaultValue,
    /// Set one field current value.
    SetFieldValue,
    /// Lock one field from edits.
    LockField,
    /// Unlock one field for edits.
    UnlockField,
    /// Emit a validation error.
    ShowError,
}

/// One business-rule condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRuleCondition {
    field_logical_name: NonEmptyString,
    operator: BusinessRuleOperator,
    value: Value,
}

impl BusinessRuleCondition {
    /// Creates a validated business-rule condition.
    pub fn new(
        field_logical_name: impl Into<String>,
        operator: BusinessRuleOperator,
        value: Value,
    ) -> AppResult<Self> {
        Ok(Self {
            field_logical_name: NonEmptyString::new(field_logical_name)?,
            operator,
            value,
        })
    }

    /// Returns condition field logical name.
    #[must_use]
    pub fn field_logical_name(&self) -> &NonEmptyString {
        &self.field_logical_name
    }

    /// Returns condition operator.
    #[must_use]
    pub fn operator(&self) -> BusinessRuleOperator {
        self.operator
    }

    /// Returns condition value.
    #[must_use]
    pub fn value(&self) -> &Value {
        &self.value
    }
}

/// One business-rule action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRuleAction {
    action_type: BusinessRuleActionType,
    target_field_logical_name: Option<NonEmptyString>,
    value: Option<Value>,
    error_message: Option<NonEmptyString>,
}

impl BusinessRuleAction {
    /// Creates a validated business-rule action.
    pub fn new(
        action_type: BusinessRuleActionType,
        target_field_logical_name: Option<String>,
        value: Option<Value>,
        error_message: Option<String>,
    ) -> AppResult<Self> {
        let target_field = target_field_logical_name
            .map(NonEmptyString::new)
            .transpose()?;
        let error_message = error_message.map(NonEmptyString::new).transpose()?;

        let requires_target_field = matches!(
            action_type,
            BusinessRuleActionType::ShowField
                | BusinessRuleActionType::HideField
                | BusinessRuleActionType::SetRequired
                | BusinessRuleActionType::SetOptional
                | BusinessRuleActionType::SetDefaultValue
                | BusinessRuleActionType::SetFieldValue
                | BusinessRuleActionType::LockField
                | BusinessRuleActionType::UnlockField
        );

        if requires_target_field && target_field.is_none() {
            return Err(AppError::Validation(
                "action requires target_field_logical_name".to_owned(),
            ));
        }

        if matches!(
            action_type,
            BusinessRuleActionType::SetDefaultValue | BusinessRuleActionType::SetFieldValue
        ) && value.is_none()
        {
            return Err(AppError::Validation(
                "set_default_value and set_field_value actions require value".to_owned(),
            ));
        }

        if action_type == BusinessRuleActionType::ShowError && error_message.is_none() {
            return Err(AppError::Validation(
                "show_error action requires error_message".to_owned(),
            ));
        }

        Ok(Self {
            action_type,
            target_field_logical_name: target_field,
            value,
            error_message,
        })
    }

    /// Returns action kind.
    #[must_use]
    pub fn action_type(&self) -> BusinessRuleActionType {
        self.action_type
    }

    /// Returns optional target field logical name.
    #[must_use]
    pub fn target_field_logical_name(&self) -> Option<&NonEmptyString> {
        self.target_field_logical_name.as_ref()
    }

    /// Returns optional action value payload.
    #[must_use]
    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    /// Returns optional error message.
    #[must_use]
    pub fn error_message(&self) -> Option<&NonEmptyString> {
        self.error_message.as_ref()
    }
}

/// Standalone business rule definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRuleDefinition {
    entity_logical_name: NonEmptyString,
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    scope: BusinessRuleScope,
    form_logical_name: Option<NonEmptyString>,
    conditions: Vec<BusinessRuleCondition>,
    actions: Vec<BusinessRuleAction>,
    is_active: bool,
}

/// Input payload for constructing one business rule definition.
#[derive(Debug, Clone, PartialEq)]
pub struct BusinessRuleDefinitionInput {
    /// Rule evaluation scope.
    pub scope: BusinessRuleScope,
    /// Optional form logical name for form-scoped rules.
    pub form_logical_name: Option<String>,
    /// Condition list.
    pub conditions: Vec<BusinessRuleCondition>,
    /// Action list.
    pub actions: Vec<BusinessRuleAction>,
    /// Active state.
    pub is_active: bool,
}

impl BusinessRuleDefinition {
    /// Creates a validated business rule definition.
    pub fn new(
        entity_logical_name: impl Into<String>,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        input: BusinessRuleDefinitionInput,
    ) -> AppResult<Self> {
        let BusinessRuleDefinitionInput {
            scope,
            form_logical_name,
            conditions,
            actions,
            is_active,
        } = input;

        if conditions.is_empty() {
            return Err(AppError::Validation(
                "business rules require at least one condition".to_owned(),
            ));
        }

        if actions.is_empty() {
            return Err(AppError::Validation(
                "business rules require at least one action".to_owned(),
            ));
        }

        let form_logical_name = form_logical_name.map(NonEmptyString::new).transpose()?;
        match (scope, form_logical_name.is_some()) {
            (BusinessRuleScope::Form, false) => {
                return Err(AppError::Validation(
                    "form-scoped business rules require form_logical_name".to_owned(),
                ));
            }
            (BusinessRuleScope::Entity, true) => {
                return Err(AppError::Validation(
                    "entity-scoped business rules cannot set form_logical_name".to_owned(),
                ));
            }
            _ => {}
        }

        Ok(Self {
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            scope,
            form_logical_name,
            conditions,
            actions,
            is_active,
        })
    }

    /// Returns parent entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns business-rule logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns rule scope.
    #[must_use]
    pub fn scope(&self) -> BusinessRuleScope {
        self.scope
    }

    /// Returns optional form logical name.
    #[must_use]
    pub fn form_logical_name(&self) -> Option<&NonEmptyString> {
        self.form_logical_name.as_ref()
    }

    /// Returns condition list.
    #[must_use]
    pub fn conditions(&self) -> &[BusinessRuleCondition] {
        &self.conditions
    }

    /// Returns action list.
    #[must_use]
    pub fn actions(&self) -> &[BusinessRuleAction] {
        &self.actions
    }

    /// Returns active flag.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
