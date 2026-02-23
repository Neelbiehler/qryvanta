use qryvanta_domain::{
    EntityDefinition, EntityFieldDefinition, FormDefinition, OptionSetDefinition, OptionSetItem,
    PublishedEntitySchema, ViewDefinition,
};
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

impl From<EntityDefinition> for EntityResponse {
    fn from(entity: EntityDefinition) -> Self {
        Self {
            logical_name: entity.logical_name().as_str().to_owned(),
            display_name: entity.display_name().as_str().to_owned(),
            description: entity.description().map(str::to_owned),
            plural_display_name: entity
                .plural_display_name()
                .map(|value| value.as_str().to_owned()),
            icon: entity.icon().map(str::to_owned),
        }
    }
}

impl From<EntityFieldDefinition> for FieldResponse {
    fn from(value: EntityFieldDefinition) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            field_type: value.field_type().as_str().to_owned(),
            is_required: value.is_required(),
            is_unique: value.is_unique(),
            description: value.description().map(str::to_owned),
            default_value: value.default_value().cloned(),
            relation_target_entity: value
                .relation_target_entity()
                .map(|target| target.as_str().to_owned()),
            option_set_logical_name: value
                .option_set_logical_name()
                .map(|target| target.as_str().to_owned()),
            max_length: value.max_length(),
            min_value: value.min_value(),
            max_value: value.max_value(),
        }
    }
}

impl From<OptionSetItem> for OptionSetItemDto {
    fn from(value: OptionSetItem) -> Self {
        Self {
            value: value.value(),
            label: value.label().as_str().to_owned(),
            color: value.color().map(str::to_owned),
            position: value.position(),
        }
    }
}

impl TryFrom<OptionSetItemDto> for OptionSetItem {
    type Error = qryvanta_core::AppError;

    fn try_from(value: OptionSetItemDto) -> Result<Self, Self::Error> {
        OptionSetItem::new(value.value, value.label, value.color, value.position)
    }
}

impl From<OptionSetDefinition> for OptionSetResponse {
    fn from(value: OptionSetDefinition) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            options: value
                .options()
                .iter()
                .cloned()
                .map(OptionSetItemDto::from)
                .collect(),
        }
    }
}

impl From<PublishedEntitySchema> for PublishedSchemaResponse {
    fn from(value: PublishedEntitySchema) -> Self {
        Self {
            entity_logical_name: value.entity().logical_name().as_str().to_owned(),
            entity_display_name: value.entity().display_name().as_str().to_owned(),
            version: value.version(),
            fields: value
                .fields()
                .iter()
                .cloned()
                .map(FieldResponse::from)
                .collect(),
            option_sets: value
                .option_sets()
                .iter()
                .cloned()
                .map(OptionSetResponse::from)
                .collect(),
        }
    }
}

impl From<FormDefinition> for FormResponse {
    fn from(value: FormDefinition) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            form_type: value.form_type().as_str().to_owned(),
            tabs: value
                .tabs()
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default(),
            header_fields: value.header_fields().to_vec(),
        }
    }
}

impl From<ViewDefinition> for ViewResponse {
    fn from(value: ViewDefinition) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            view_type: value.view_type().as_str().to_owned(),
            columns: value
                .columns()
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default(),
            default_sort: value
                .default_sort()
                .and_then(|sort| serde_json::to_value(sort).ok()),
            filter_criteria: value
                .filter_criteria()
                .and_then(|group| serde_json::to_value(group).ok()),
            is_default: value.is_default(),
        }
    }
}
