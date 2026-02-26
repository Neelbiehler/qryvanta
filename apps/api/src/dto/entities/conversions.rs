use qryvanta_domain::{
    BusinessRuleDefinition, EntityDefinition, EntityFieldDefinition, FormDefinition,
    OptionSetDefinition, OptionSetItem, PublishedEntitySchema, ViewDefinition,
};

use super::types::{
    BusinessRuleResponse, EntityResponse, FieldResponse, FormResponse, OptionSetItemDto,
    OptionSetResponse, PublishedSchemaResponse, ViewResponse,
};

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
            calculation_expression: value.calculation_expression().map(str::to_owned),
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

impl From<BusinessRuleDefinition> for BusinessRuleResponse {
    fn from(value: BusinessRuleDefinition) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            scope: value.scope().as_str().to_owned(),
            form_logical_name: value
                .form_logical_name()
                .map(|form| form.as_str().to_owned()),
            conditions: value
                .conditions()
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default(),
            actions: value
                .actions()
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default(),
            is_active: value.is_active(),
        }
    }
}
