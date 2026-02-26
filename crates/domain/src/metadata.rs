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
    description: Option<String>,
    plural_display_name: Option<NonEmptyString>,
    icon: Option<String>,
}

impl EntityDefinition {
    /// Creates a new entity definition with validated fields.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> AppResult<Self> {
        Self::new_with_details(logical_name, display_name, None, None, None)
    }

    /// Creates a new entity definition with optional enriched metadata fields.
    pub fn new_with_details(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        description: Option<String>,
        plural_display_name: Option<String>,
        icon: Option<String>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            description: normalize_optional_text(description),
            plural_display_name: plural_display_name
                .and_then(|value| normalize_optional_text(Some(value)))
                .map(NonEmptyString::new)
                .transpose()?,
            icon: normalize_optional_text(icon),
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

    /// Returns optional longer-form description text.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns optional plural display label.
    #[must_use]
    pub fn plural_display_name(&self) -> Option<&NonEmptyString> {
        self.plural_display_name.as_ref()
    }

    /// Returns optional icon key.
    #[must_use]
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Returns a copy with updated mutable metadata fields.
    pub fn with_updates(
        &self,
        display_name: impl Into<String>,
        description: Option<String>,
        plural_display_name: Option<String>,
        icon: Option<String>,
    ) -> AppResult<Self> {
        Self::new_with_details(
            self.logical_name.as_str(),
            display_name,
            description,
            plural_display_name,
            icon,
        )
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
    /// Single-select option set field.
    Choice,
    /// Multi-select option set field.
    MultiChoice,
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
            Self::Choice => "choice",
            Self::MultiChoice => "multichoice",
            Self::Relation => "relation",
        }
    }

    fn validate_value(self, value: &Value) -> AppResult<()> {
        let is_valid = match self {
            Self::Text | Self::Date | Self::DateTime => value.is_string(),
            Self::Number => value.is_number(),
            Self::Boolean => value.is_boolean(),
            Self::Json => true,
            Self::Choice => value.is_i64() || value.is_u64(),
            Self::MultiChoice => value.is_array(),
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
            "choice" => Ok(Self::Choice),
            "multichoice" => Ok(Self::MultiChoice),
            "relation" => Ok(Self::Relation),
            _ => Err(AppError::Validation(format!(
                "unknown field type '{value}'"
            ))),
        }
    }
}

/// Option set item used by choice-style field types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionSetItem {
    value: i32,
    label: NonEmptyString,
    color: Option<String>,
    position: i32,
}

impl OptionSetItem {
    /// Creates a validated option set item.
    pub fn new(
        value: i32,
        label: impl Into<String>,
        color: Option<String>,
        position: i32,
    ) -> AppResult<Self> {
        Ok(Self {
            value,
            label: NonEmptyString::new(label)?,
            color: normalize_optional_text(color),
            position,
        })
    }

    /// Returns stable item value.
    #[must_use]
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Returns item label.
    #[must_use]
    pub fn label(&self) -> &NonEmptyString {
        &self.label
    }

    /// Returns optional color token.
    #[must_use]
    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    /// Returns item sort position.
    #[must_use]
    pub fn position(&self) -> i32 {
        self.position
    }
}

/// Entity-scoped option set definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionSetDefinition {
    entity_logical_name: NonEmptyString,
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    options: Vec<OptionSetItem>,
}

impl OptionSetDefinition {
    /// Creates a validated option set definition.
    pub fn new(
        entity_logical_name: impl Into<String>,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        options: Vec<OptionSetItem>,
    ) -> AppResult<Self> {
        if options.is_empty() {
            return Err(AppError::Validation(
                "option sets must include at least one item".to_owned(),
            ));
        }

        let mut seen_values = HashSet::new();
        for option in &options {
            if !seen_values.insert(option.value()) {
                return Err(AppError::Validation(format!(
                    "duplicate option set value '{}' in option set",
                    option.value()
                )));
            }
        }

        Ok(Self {
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            options,
        })
    }

    /// Returns parent entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns option set logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns option set display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns configured options.
    #[must_use]
    pub fn options(&self) -> &[OptionSetItem] {
        &self.options
    }

    /// Returns whether a numeric option value exists.
    #[must_use]
    pub fn contains_value(&self, value: i32) -> bool {
        self.options.iter().any(|item| item.value() == value)
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
    option_set_logical_name: Option<NonEmptyString>,
    description: Option<String>,
    calculation_expression: Option<String>,
    max_length: Option<i32>,
    min_value: Option<f64>,
    max_value: Option<f64>,
}

/// Input payload for updating mutable metadata field attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityFieldMutableUpdateInput {
    /// Updated display name.
    pub display_name: String,
    /// Optional freeform field description.
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
        Self::new_with_details(
            entity_logical_name,
            logical_name,
            display_name,
            field_type,
            is_required,
            is_unique,
            default_value,
            relation_target_entity,
            None,
            None,
            None,
            None,
            None,
        )
    }

    /// Creates a validated metadata field definition with optional enriched metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_details(
        entity_logical_name: impl Into<String>,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        field_type: FieldType,
        is_required: bool,
        is_unique: bool,
        default_value: Option<Value>,
        relation_target_entity: Option<String>,
        option_set_logical_name: Option<String>,
        description: Option<String>,
        max_length: Option<i32>,
        min_value: Option<f64>,
        max_value: Option<f64>,
    ) -> AppResult<Self> {
        Self::new_with_details_and_calculation(
            entity_logical_name,
            logical_name,
            display_name,
            field_type,
            is_required,
            is_unique,
            default_value,
            relation_target_entity,
            option_set_logical_name,
            description,
            None,
            max_length,
            min_value,
            max_value,
        )
    }

    /// Creates a validated metadata field definition with optional calculation expression.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_details_and_calculation(
        entity_logical_name: impl Into<String>,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        field_type: FieldType,
        is_required: bool,
        is_unique: bool,
        default_value: Option<Value>,
        relation_target_entity: Option<String>,
        option_set_logical_name: Option<String>,
        description: Option<String>,
        calculation_expression: Option<String>,
        max_length: Option<i32>,
        min_value: Option<f64>,
        max_value: Option<f64>,
    ) -> AppResult<Self> {
        if is_unique && matches!(field_type, FieldType::Json) {
            return Err(AppError::Validation(
                "unique constraints are not supported for json field type".to_owned(),
            ));
        }

        let relation_target_entity = relation_target_entity
            .map(NonEmptyString::new)
            .transpose()?;
        let option_set_logical_name = option_set_logical_name
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

        match (field_type, option_set_logical_name.is_some()) {
            (FieldType::Choice | FieldType::MultiChoice, false) => {
                return Err(AppError::Validation(
                    "choice and multichoice fields require option_set_logical_name".to_owned(),
                ));
            }
            (FieldType::Choice | FieldType::MultiChoice, true) => {}
            (_, true) => {
                return Err(AppError::Validation(
                    "option_set_logical_name is only allowed for choice and multichoice fields"
                        .to_owned(),
                ));
            }
            (_, false) => {}
        }

        if let Some(default_value) = &default_value {
            field_type.validate_value(default_value)?;
        }

        let calculation_expression = normalize_optional_text(calculation_expression);
        if calculation_expression.is_some() {
            if !matches!(field_type, FieldType::Text | FieldType::Number) {
                return Err(AppError::Validation(
                    "calculation_expression is only allowed for text and number fields".to_owned(),
                ));
            }

            if default_value.is_some() {
                return Err(AppError::Validation(
                    "calculated fields cannot define default_value".to_owned(),
                ));
            }
        }

        match field_type {
            FieldType::Text => {
                if let Some(value) = max_length
                    && value <= 0
                {
                    return Err(AppError::Validation(
                        "max_length must be greater than zero for text fields".to_owned(),
                    ));
                }

                if min_value.is_some() || max_value.is_some() {
                    return Err(AppError::Validation(
                        "min_value/max_value are only allowed for number fields".to_owned(),
                    ));
                }
            }
            FieldType::Number => {
                if max_length.is_some() {
                    return Err(AppError::Validation(
                        "max_length is only allowed for text fields".to_owned(),
                    ));
                }

                if let (Some(minimum), Some(maximum)) = (min_value, max_value)
                    && minimum > maximum
                {
                    return Err(AppError::Validation(
                        "min_value must be less than or equal to max_value".to_owned(),
                    ));
                }
            }
            _ => {
                if max_length.is_some() {
                    return Err(AppError::Validation(
                        "max_length is only allowed for text fields".to_owned(),
                    ));
                }

                if min_value.is_some() || max_value.is_some() {
                    return Err(AppError::Validation(
                        "min_value/max_value are only allowed for number fields".to_owned(),
                    ));
                }
            }
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
            option_set_logical_name,
            description: normalize_optional_text(description),
            calculation_expression,
            max_length,
            min_value,
            max_value,
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

    /// Returns option set logical name for choice-like fields.
    #[must_use]
    pub fn option_set_logical_name(&self) -> Option<&NonEmptyString> {
        self.option_set_logical_name.as_ref()
    }

    /// Returns optional field description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns optional calculation expression used for runtime computed values.
    #[must_use]
    pub fn calculation_expression(&self) -> Option<&str> {
        self.calculation_expression.as_deref()
    }

    /// Returns optional text max length constraint.
    #[must_use]
    pub fn max_length(&self) -> Option<i32> {
        self.max_length
    }

    /// Returns optional number minimum value constraint.
    #[must_use]
    pub fn min_value(&self) -> Option<f64> {
        self.min_value
    }

    /// Returns optional number maximum value constraint.
    #[must_use]
    pub fn max_value(&self) -> Option<f64> {
        self.max_value
    }

    /// Returns a copy with updated mutable metadata fields.
    pub fn with_mutable_updates(
        &self,
        display_name: String,
        description: Option<String>,
        default_value: Option<Value>,
        max_length: Option<i32>,
        min_value: Option<f64>,
        max_value: Option<f64>,
    ) -> AppResult<Self> {
        Self::new_with_details(
            self.entity_logical_name().as_str(),
            self.logical_name().as_str(),
            display_name,
            self.field_type,
            self.is_required,
            self.is_unique,
            default_value,
            self.relation_target_entity()
                .map(|value| value.as_str().to_owned()),
            self.option_set_logical_name()
                .map(|value| value.as_str().to_owned()),
            description,
            max_length,
            min_value,
            max_value,
        )
    }

    /// Returns a copy with updated mutable metadata fields and calculation expression.
    pub fn with_mutable_updates_and_calculation(
        &self,
        input: EntityFieldMutableUpdateInput,
    ) -> AppResult<Self> {
        let EntityFieldMutableUpdateInput {
            display_name,
            description,
            default_value,
            calculation_expression,
            max_length,
            min_value,
            max_value,
        } = input;

        Self::new_with_details_and_calculation(
            self.entity_logical_name().as_str(),
            self.logical_name().as_str(),
            display_name,
            self.field_type,
            self.is_required,
            self.is_unique,
            default_value,
            self.relation_target_entity()
                .map(|value| value.as_str().to_owned()),
            self.option_set_logical_name()
                .map(|value| value.as_str().to_owned()),
            description,
            calculation_expression,
            max_length,
            min_value,
            max_value,
        )
    }

    /// Validates a runtime value against this field definition.
    pub fn validate_runtime_value(&self, value: &Value) -> AppResult<()> {
        self.field_type.validate_value(value)?;

        match self.field_type {
            FieldType::Text => {
                if let Some(max_length) = self.max_length
                    && let Some(text) = value.as_str()
                    && text.chars().count() > max_length as usize
                {
                    return Err(AppError::Validation(format!(
                        "field '{}' exceeds max_length {}",
                        self.logical_name.as_str(),
                        max_length
                    )));
                }
            }
            FieldType::Number => {
                let Some(number) = value.as_f64() else {
                    return Ok(());
                };

                if let Some(minimum) = self.min_value
                    && number < minimum
                {
                    return Err(AppError::Validation(format!(
                        "field '{}' must be greater than or equal to {}",
                        self.logical_name.as_str(),
                        minimum
                    )));
                }

                if let Some(maximum) = self.max_value
                    && number > maximum
                {
                    return Err(AppError::Validation(format!(
                        "field '{}' must be less than or equal to {}",
                        self.logical_name.as_str(),
                        maximum
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|candidate| {
        let trimmed = candidate.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

/// Immutable published entity schema snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishedEntitySchema {
    entity: EntityDefinition,
    version: i32,
    fields: Vec<EntityFieldDefinition>,
    #[serde(default)]
    option_sets: Vec<OptionSetDefinition>,
}

impl PublishedEntitySchema {
    /// Creates a new published schema with invariant checks.
    pub fn new(
        entity: EntityDefinition,
        version: i32,
        fields: Vec<EntityFieldDefinition>,
        option_sets: Vec<OptionSetDefinition>,
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

        for field in &fields {
            let Some(option_set_logical_name) = field.option_set_logical_name() else {
                continue;
            };

            let exists = option_sets
                .iter()
                .any(|set| set.logical_name().as_str() == option_set_logical_name.as_str());
            if !exists {
                return Err(AppError::Validation(format!(
                    "field '{}.{}' references missing option set '{}'",
                    field.entity_logical_name().as_str(),
                    field.logical_name().as_str(),
                    option_set_logical_name.as_str()
                )));
            }
        }

        Ok(Self {
            entity,
            version,
            fields,
            option_sets,
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

    /// Returns all option set definitions resolved into this schema snapshot.
    #[must_use]
    pub fn option_sets(&self) -> &[OptionSetDefinition] {
        &self.option_sets
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
        EntityDefinition, EntityFieldDefinition, FieldType, OptionSetDefinition, OptionSetItem,
        PublishedEntitySchema, RuntimeRecord,
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

        let result = PublishedEntitySchema::new(entity, 1, vec![first, second], Vec::new());
        assert!(result.is_err());
    }

    #[test]
    fn runtime_record_requires_object_payload() {
        let result = RuntimeRecord::new("1", "contact", json!("not-object"));
        assert!(result.is_err());
    }

    #[test]
    fn choice_field_requires_option_set_reference() {
        let result = EntityFieldDefinition::new(
            "contact",
            "status",
            "Status",
            FieldType::Choice,
            true,
            false,
            None,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn option_set_definition_rejects_duplicate_values() {
        let option_set = OptionSetDefinition::new(
            "contact",
            "status",
            "Status",
            vec![
                OptionSetItem::new(1, "Open", None, 0).unwrap_or_else(|_| unreachable!()),
                OptionSetItem::new(1, "Closed", None, 1).unwrap_or_else(|_| unreachable!()),
            ],
        );
        assert!(option_set.is_err());
    }

    #[test]
    fn text_field_enforces_max_length_at_runtime() {
        let field = EntityFieldDefinition::new_with_details(
            "contact",
            "name",
            "Name",
            FieldType::Text,
            true,
            false,
            None,
            None,
            None,
            None,
            Some(3),
            None,
            None,
        )
        .unwrap_or_else(|_| unreachable!());

        let valid = field.validate_runtime_value(&json!("abc"));
        assert!(valid.is_ok());

        let invalid = field.validate_runtime_value(&json!("abcd"));
        assert!(invalid.is_err());
    }

    #[test]
    fn number_field_enforces_min_and_max_at_runtime() {
        let field = EntityFieldDefinition::new_with_details(
            "invoice",
            "amount",
            "Amount",
            FieldType::Number,
            true,
            false,
            None,
            None,
            None,
            None,
            None,
            Some(10.0),
            Some(20.0),
        )
        .unwrap_or_else(|_| unreachable!());

        let too_low = field.validate_runtime_value(&json!(9));
        assert!(too_low.is_err());

        let in_range = field.validate_runtime_value(&json!(15));
        assert!(in_range.is_ok());

        let too_high = field.validate_runtime_value(&json!(21));
        assert!(too_high.is_err());
    }

    #[test]
    fn published_schema_deserializes_missing_option_sets_for_backwards_compatibility() {
        let entity = EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
        let field = EntityFieldDefinition::new(
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
        let schema = PublishedEntitySchema::new(entity, 1, vec![field], Vec::new())
            .unwrap_or_else(|_| unreachable!());

        let mut schema_json = serde_json::to_value(schema).unwrap_or_else(|_| unreachable!());
        if let Some(object) = schema_json.as_object_mut() {
            object.remove("option_sets");
        }

        let parsed: PublishedEntitySchema =
            serde_json::from_value(schema_json).unwrap_or_else(|_| unreachable!());
        assert!(parsed.option_sets().is_empty());
    }
}
