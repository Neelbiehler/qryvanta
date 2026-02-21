use std::collections::HashSet;
use std::str::FromStr;

use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Metadata definition for a business entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityDefinition {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
}

impl EntityDefinition {
    /// Creates a new entity definition with validated fields.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
        })
    }

    /// Returns the logical (stable) name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns the display (human-friendly) name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }
}

/// Supported metadata field types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// UTF-8 string field.
    Text,
    /// Numeric field.
    Number,
    /// Boolean field.
    Boolean,
    /// Date-only string field.
    Date,
    /// Date-time string field.
    DateTime,
    /// Arbitrary JSON field.
    Json,
    /// Many-to-one relation field.
    Relation,
}

impl FieldType {
    /// Returns a stable storage value for the field type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::DateTime => "datetime",
            Self::Json => "json",
            Self::Relation => "relation",
        }
    }

    fn validate_value(self, value: &Value) -> AppResult<()> {
        let is_valid = match self {
            Self::Text | Self::Date | Self::DateTime => value.is_string(),
            Self::Number => value.is_number(),
            Self::Boolean => value.is_boolean(),
            Self::Json => true,
            Self::Relation => value
                .as_str()
                .map(|text| !text.trim().is_empty())
                .unwrap_or(false),
        };

        if !is_valid {
            return Err(AppError::Validation(format!(
                "value does not match field type '{}'",
                self.as_str()
            )));
        }

        Ok(())
    }
}

impl FromStr for FieldType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "text" => Ok(Self::Text),
            "number" => Ok(Self::Number),
            "boolean" => Ok(Self::Boolean),
            "date" => Ok(Self::Date),
            "datetime" => Ok(Self::DateTime),
            "json" => Ok(Self::Json),
            "relation" => Ok(Self::Relation),
            _ => Err(AppError::Validation(format!(
                "unknown field type '{value}'"
            ))),
        }
    }
}

/// Metadata definition for a single entity field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityFieldDefinition {
    entity_logical_name: NonEmptyString,
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    field_type: FieldType,
    is_required: bool,
    is_unique: bool,
    default_value: Option<Value>,
    relation_target_entity: Option<NonEmptyString>,
}

impl EntityFieldDefinition {
    /// Creates a validated metadata field definition.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_logical_name: impl Into<String>,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        field_type: FieldType,
        is_required: bool,
        is_unique: bool,
        default_value: Option<Value>,
        relation_target_entity: Option<String>,
    ) -> AppResult<Self> {
        if is_unique && matches!(field_type, FieldType::Json) {
            return Err(AppError::Validation(
                "unique constraints are not supported for json field type".to_owned(),
            ));
        }

        let relation_target_entity = relation_target_entity
            .map(NonEmptyString::new)
            .transpose()?;

        match (field_type, relation_target_entity.is_some()) {
            (FieldType::Relation, false) => {
                return Err(AppError::Validation(
                    "relation fields require relation_target_entity".to_owned(),
                ));
            }
            (FieldType::Relation, true) => {}
            (_, true) => {
                return Err(AppError::Validation(
                    "relation_target_entity is only allowed for relation fields".to_owned(),
                ));
            }
            (_, false) => {}
        }

        if let Some(default_value) = &default_value {
            field_type.validate_value(default_value)?;
        }

        Ok(Self {
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            field_type,
            is_required,
            is_unique,
            default_value,
            relation_target_entity,
        })
    }

    /// Returns the field's parent entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns the field logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns the field type.
    #[must_use]
    pub fn field_type(&self) -> FieldType {
        self.field_type
    }

    /// Returns whether the field is required.
    #[must_use]
    pub fn is_required(&self) -> bool {
        self.is_required
    }

    /// Returns whether the field is unique.
    #[must_use]
    pub fn is_unique(&self) -> bool {
        self.is_unique
    }

    /// Returns the default value.
    #[must_use]
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }

    /// Returns relation target entity when field type is relation.
    #[must_use]
    pub fn relation_target_entity(&self) -> Option<&NonEmptyString> {
        self.relation_target_entity.as_ref()
    }

    /// Validates a runtime value against this field definition.
    pub fn validate_runtime_value(&self, value: &Value) -> AppResult<()> {
        self.field_type.validate_value(value)
    }
}

/// Immutable published entity schema snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishedEntitySchema {
    entity: EntityDefinition,
    version: i32,
    fields: Vec<EntityFieldDefinition>,
}

impl PublishedEntitySchema {
    /// Creates a new published schema with invariant checks.
    pub fn new(
        entity: EntityDefinition,
        version: i32,
        fields: Vec<EntityFieldDefinition>,
    ) -> AppResult<Self> {
        if version <= 0 {
            return Err(AppError::Validation(
                "published schema version must be positive".to_owned(),
            ));
        }

        let mut seen = HashSet::new();
        for field in &fields {
            if !seen.insert(field.logical_name().as_str().to_owned()) {
                return Err(AppError::Validation(format!(
                    "duplicate field logical name '{}' in published schema",
                    field.logical_name().as_str()
                )));
            }
        }

        Ok(Self {
            entity,
            version,
            fields,
        })
    }

    /// Returns the entity metadata.
    #[must_use]
    pub fn entity(&self) -> &EntityDefinition {
        &self.entity
    }

    /// Returns the published schema version.
    #[must_use]
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Returns all published fields.
    #[must_use]
    pub fn fields(&self) -> &[EntityFieldDefinition] {
        &self.fields
    }
}

/// Runtime record payload persisted for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeRecord {
    record_id: NonEmptyString,
    entity_logical_name: NonEmptyString,
    data: Value,
}

impl RuntimeRecord {
    /// Creates a validated runtime record projection.
    pub fn new(
        record_id: impl Into<String>,
        entity_logical_name: impl Into<String>,
        data: Value,
    ) -> AppResult<Self> {
        if !data.is_object() {
            return Err(AppError::Validation(
                "runtime record data must be a JSON object".to_owned(),
            ));
        }

        Ok(Self {
            record_id: NonEmptyString::new(record_id)?,
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            data,
        })
    }

    /// Returns the stable runtime record identifier.
    #[must_use]
    pub fn record_id(&self) -> &NonEmptyString {
        &self.record_id
    }

    /// Returns the parent entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns the record JSON object.
    #[must_use]
    pub fn data(&self) -> &Value {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        EntityDefinition, EntityFieldDefinition, FieldType, PublishedEntitySchema, RuntimeRecord,
    };

    #[test]
    fn entity_requires_non_empty_fields() {
        let result = EntityDefinition::new("", "Contact");
        assert!(result.is_err());
    }

    #[test]
    fn relation_fields_require_target_entity() {
        let result = EntityFieldDefinition::new(
            "contact",
            "owner",
            "Owner",
            FieldType::Relation,
            false,
            false,
            None,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn published_schema_rejects_duplicate_fields() {
        let entity = EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
        let first = EntityFieldDefinition::new(
            "contact",
            "name",
            "Name",
            FieldType::Text,
            true,
            false,
            None,
            None,
        )
        .unwrap_or_else(|_| unreachable!());
        let second = EntityFieldDefinition::new(
            "contact",
            "name",
            "Name",
            FieldType::Text,
            true,
            false,
            None,
            None,
        )
        .unwrap_or_else(|_| unreachable!());

        let result = PublishedEntitySchema::new(entity, 1, vec![first, second]);
        assert!(result.is_err());
    }

    #[test]
    fn runtime_record_requires_object_payload() {
        let result = RuntimeRecord::new("1", "contact", json!("not-object"));
        assert!(result.is_err());
    }
}
