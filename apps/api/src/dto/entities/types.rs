use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// Incoming payload for entity creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-entity-request.ts"
)]
pub struct CreateEntityRequest {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub plural_display_name: Option<String>,
    pub icon: Option<String>,
}

/// API representation of an entity.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/entity-response.ts"
)]
pub struct EntityResponse {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub plural_display_name: Option<String>,
    pub icon: Option<String>,
}

/// Incoming payload for entity update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/update-entity-request.ts"
)]
pub struct UpdateEntityRequest {
    pub display_name: String,
    pub description: Option<String>,
    pub plural_display_name: Option<String>,
    pub icon: Option<String>,
}

/// Incoming payload for metadata field create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-field-request.ts"
)]
pub struct CreateFieldRequest {
    pub logical_name: String,
    pub display_name: String,
    pub field_type: String,
    pub is_required: bool,
    pub is_unique: bool,
    #[ts(type = "unknown | null")]
    pub default_value: Option<Value>,
    pub calculation_expression: Option<String>,
    pub relation_target_entity: Option<String>,
    pub option_set_logical_name: Option<String>,
}

/// Incoming payload for metadata field updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/update-field-request.ts"
)]
pub struct UpdateFieldRequest {
    pub display_name: String,
    pub description: Option<String>,
    #[ts(type = "unknown | null")]
    pub default_value: Option<Value>,
    pub calculation_expression: Option<String>,
    pub max_length: Option<i32>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

/// API representation of a metadata field definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/field-response.ts"
)]
pub struct FieldResponse {
    pub entity_logical_name: String,
    pub logical_name: String,
    pub display_name: String,
    pub field_type: String,
    pub is_required: bool,
    pub is_unique: bool,
    pub description: Option<String>,
    #[ts(type = "unknown | null")]
    pub default_value: Option<Value>,
    pub calculation_expression: Option<String>,
    pub relation_target_entity: Option<String>,
    pub option_set_logical_name: Option<String>,
    pub max_length: Option<i32>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

/// Incoming payload for option set create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-option-set-request.ts"
)]
pub struct CreateOptionSetRequest {
    pub logical_name: String,
    pub display_name: String,
    pub options: Vec<OptionSetItemDto>,
}

/// API transport representation of one option set item.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/option-set-item-dto.ts"
)]
pub struct OptionSetItemDto {
    pub value: i32,
    pub label: String,
    pub color: Option<String>,
    pub position: i32,
}

/// API response for one option set definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/option-set-response.ts"
)]
pub struct OptionSetResponse {
    pub entity_logical_name: String,
    pub logical_name: String,
    pub display_name: String,
    pub options: Vec<OptionSetItemDto>,
}

/// Incoming payload for standalone form create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-form-request.ts"
)]
pub struct CreateFormRequest {
    pub logical_name: String,
    pub display_name: String,
    pub form_type: String,
    #[ts(type = "unknown[]")]
    pub tabs: Vec<Value>,
    pub header_fields: Vec<String>,
}

/// API response for standalone forms.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/form-response.ts"
)]
pub struct FormResponse {
    pub entity_logical_name: String,
    pub logical_name: String,
    pub display_name: String,
    pub form_type: String,
    #[ts(type = "unknown[]")]
    pub tabs: Vec<Value>,
    pub header_fields: Vec<String>,
}

/// Incoming payload for standalone view create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-view-request.ts"
)]
pub struct CreateViewRequest {
    pub logical_name: String,
    pub display_name: String,
    pub view_type: String,
    #[ts(type = "unknown[]")]
    pub columns: Vec<Value>,
    #[ts(type = "unknown | null")]
    pub default_sort: Option<Value>,
    #[ts(type = "unknown | null")]
    pub filter_criteria: Option<Value>,
    pub is_default: bool,
}

/// API response for standalone views.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/view-response.ts"
)]
pub struct ViewResponse {
    pub entity_logical_name: String,
    pub logical_name: String,
    pub display_name: String,
    pub view_type: String,
    #[ts(type = "unknown[]")]
    pub columns: Vec<Value>,
    #[ts(type = "unknown | null")]
    pub default_sort: Option<Value>,
    #[ts(type = "unknown | null")]
    pub filter_criteria: Option<Value>,
    pub is_default: bool,
}

/// Incoming payload for business-rule create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-business-rule-request.ts"
)]
pub struct CreateBusinessRuleRequest {
    pub logical_name: String,
    pub display_name: String,
    pub scope: String,
    pub form_logical_name: Option<String>,
    #[ts(type = "unknown[]")]
    pub conditions: Vec<Value>,
    #[ts(type = "unknown[]")]
    pub actions: Vec<Value>,
    pub is_active: bool,
}

/// API response for standalone business rules.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/business-rule-response.ts"
)]
pub struct BusinessRuleResponse {
    pub entity_logical_name: String,
    pub logical_name: String,
    pub display_name: String,
    pub scope: String,
    pub form_logical_name: Option<String>,
    #[ts(type = "unknown[]")]
    pub conditions: Vec<Value>,
    #[ts(type = "unknown[]")]
    pub actions: Vec<Value>,
    pub is_active: bool,
}

/// API representation of a published schema snapshot.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/published-schema-response.ts"
)]
pub struct PublishedSchemaResponse {
    pub entity_logical_name: String,
    pub entity_display_name: String,
    pub version: i32,
    pub fields: Vec<FieldResponse>,
    pub option_sets: Vec<OptionSetResponse>,
}

/// Publish validation report for one entity.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-checks-response.ts"
)]
pub struct PublishChecksResponse {
    pub is_publishable: bool,
    pub errors: Vec<String>,
}
