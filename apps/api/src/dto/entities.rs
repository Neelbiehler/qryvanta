use qryvanta_domain::{EntityDefinition, EntityFieldDefinition, PublishedEntitySchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// Incoming payload for entity creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/create-entity-request.ts"
)]
pub struct CreateEntityRequest {
    pub logical_name: String,
    pub display_name: String,
}

/// API representation of an entity.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/entity-response.ts"
)]
pub struct EntityResponse {
    pub logical_name: String,
    pub display_name: String,
}

/// Incoming payload for metadata field create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/create-field-request.ts"
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
}

/// API representation of a metadata field definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/field-response.ts"
)]
pub struct FieldResponse {
    pub entity_logical_name: String,
    pub logical_name: String,
    pub display_name: String,
    pub field_type: String,
    pub is_required: bool,
    pub is_unique: bool,
    #[ts(type = "unknown | null")]
    pub default_value: Option<Value>,
    pub relation_target_entity: Option<String>,
}

/// API representation of a published schema snapshot.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/published-schema-response.ts"
)]
pub struct PublishedSchemaResponse {
    pub entity_logical_name: String,
    pub entity_display_name: String,
    pub version: i32,
    pub fields: Vec<FieldResponse>,
}

impl From<EntityDefinition> for EntityResponse {
    fn from(entity: EntityDefinition) -> Self {
        Self {
            logical_name: entity.logical_name().as_str().to_owned(),
            display_name: entity.display_name().as_str().to_owned(),
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
            default_value: value.default_value().cloned(),
            relation_target_entity: value
                .relation_target_entity()
                .map(|target| target.as_str().to_owned()),
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
        }
    }
}
