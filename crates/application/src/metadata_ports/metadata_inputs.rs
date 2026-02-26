use qryvanta_domain::{
    BusinessRuleAction, BusinessRuleCondition, BusinessRuleScope, FieldType, FormTab, FormType,
    OptionSetItem, ViewColumn, ViewFilterGroup, ViewSort, ViewType,
};
use serde_json::Value;

/// Input payload for metadata field create/update operations.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveFieldInput {
    /// Parent entity logical name.
    pub entity_logical_name: String,
    /// Field logical name.
    pub logical_name: String,
    /// Field display name.
    pub display_name: String,
    /// Field type.
    pub field_type: FieldType,
    /// Required field marker.
    pub is_required: bool,
    /// Unique field marker.
    pub is_unique: bool,
    /// Optional default value.
    pub default_value: Option<Value>,
    /// Optional relation target entity logical name.
    pub relation_target_entity: Option<String>,
    /// Optional option set logical name for choice-like fields.
    pub option_set_logical_name: Option<String>,
    /// Optional calculation expression for computed fields.
    pub calculation_expression: Option<String>,
}

/// Input payload for option set create/update operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOptionSetInput {
    /// Parent entity logical name.
    pub entity_logical_name: String,
    /// Option set logical name.
    pub logical_name: String,
    /// Display name.
    pub display_name: String,
    /// Ordered option values.
    pub options: Vec<OptionSetItem>,
}

/// Input payload for form create/update operations.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveFormInput {
    /// Parent entity logical name.
    pub entity_logical_name: String,
    /// Form logical name.
    pub logical_name: String,
    /// Form display name.
    pub display_name: String,
    /// Form type.
    pub form_type: FormType,
    /// Form tabs.
    pub tabs: Vec<FormTab>,
    /// Header field logical names.
    pub header_fields: Vec<String>,
}

/// Input payload for view create/update operations.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveViewInput {
    /// Parent entity logical name.
    pub entity_logical_name: String,
    /// View logical name.
    pub logical_name: String,
    /// View display name.
    pub display_name: String,
    /// View type.
    pub view_type: ViewType,
    /// View columns.
    pub columns: Vec<ViewColumn>,
    /// Optional default sort.
    pub default_sort: Option<ViewSort>,
    /// Optional filter criteria.
    pub filter_criteria: Option<ViewFilterGroup>,
    /// Default view marker.
    pub is_default: bool,
}

/// Input payload for business-rule create/update operations.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveBusinessRuleInput {
    /// Parent entity logical name.
    pub entity_logical_name: String,
    /// Rule logical name.
    pub logical_name: String,
    /// Rule display name.
    pub display_name: String,
    /// Rule scope.
    pub scope: BusinessRuleScope,
    /// Optional form logical name for form-scoped rules.
    pub form_logical_name: Option<String>,
    /// Rule conditions.
    pub conditions: Vec<BusinessRuleCondition>,
    /// Rule actions.
    pub actions: Vec<BusinessRuleAction>,
    /// Active state.
    pub is_active: bool,
}

/// Input payload for entity update operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEntityInput {
    /// Entity logical name (immutable identifier).
    pub logical_name: String,
    /// New display name.
    pub display_name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional plural display name.
    pub plural_display_name: Option<String>,
    /// Optional icon key.
    pub icon: Option<String>,
}

/// Input payload for metadata field update operations.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateFieldInput {
    /// Parent entity logical name.
    pub entity_logical_name: String,
    /// Field logical name.
    pub logical_name: String,
    /// Field display name.
    pub display_name: String,
    /// Optional freeform description.
    pub description: Option<String>,
    /// Optional default value.
    pub default_value: Option<Value>,
    /// Optional calculation expression for computed fields.
    pub calculation_expression: Option<String>,
    /// Optional text max length constraint.
    pub max_length: Option<i32>,
    /// Optional number minimum value constraint.
    pub min_value: Option<f64>,
    /// Optional number maximum value constraint.
    pub max_value: Option<f64>,
}
