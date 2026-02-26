use super::*;

impl MetadataService {
    pub(super) fn normalize_record_payload_without_required(
        schema: &PublishedEntitySchema,
        data: Value,
    ) -> AppResult<serde_json::Map<String, Value>> {
        let mut object = match data {
            Value::Object(object) => object,
            _ => {
                return Err(AppError::Validation(
                    "runtime record payload must be a JSON object".to_owned(),
                ));
            }
        };

        let allowed_fields: BTreeSet<&str> = schema
            .fields()
            .iter()
            .map(|field| field.logical_name().as_str())
            .collect();
        let calculated_fields: BTreeSet<&str> = schema
            .fields()
            .iter()
            .filter_map(|field| {
                field
                    .calculation_expression()
                    .map(|_| field.logical_name().as_str())
            })
            .collect();

        for key in object.keys() {
            if !allowed_fields.contains(key.as_str()) {
                return Err(AppError::Validation(format!(
                    "unknown field '{}' for entity '{}'",
                    key,
                    schema.entity().logical_name().as_str()
                )));
            }

            if calculated_fields.contains(key.as_str()) {
                return Err(AppError::Validation(format!(
                    "calculated field '{}' cannot be set directly",
                    key
                )));
            }
        }

        for field in schema.fields() {
            let field_name = field.logical_name().as_str();
            if field.calculation_expression().is_some() {
                continue;
            }

            if let Some(value) = object.get(field_name) {
                field.validate_runtime_value(value)?;
                Self::validate_choice_value_against_option_set(schema, field, value)?;
                continue;
            }

            if let Some(default_value) = field.default_value() {
                Self::validate_choice_value_against_option_set(schema, field, default_value)?;
                object.insert(field_name.to_owned(), default_value.clone());
                continue;
            }
        }

        Ok(object)
    }

    pub(super) fn validate_record_values(
        schema: &PublishedEntitySchema,
        object: &serde_json::Map<String, Value>,
    ) -> AppResult<()> {
        for field in schema.fields() {
            let field_name = field.logical_name().as_str();
            let Some(value) = object.get(field_name) else {
                continue;
            };

            field.validate_runtime_value(value)?;
            Self::validate_choice_value_against_option_set(schema, field, value)?;
        }

        Ok(())
    }

    pub(super) fn enforce_required_fields_with_business_rules(
        schema: &PublishedEntitySchema,
        object: &serde_json::Map<String, Value>,
        effects: &EntityBusinessRuleEffects,
    ) -> AppResult<()> {
        for field in schema.fields() {
            if field.calculation_expression().is_some() {
                continue;
            }

            let field_name = field.logical_name().as_str();
            let is_required = effects
                .required_overrides
                .get(field_name)
                .copied()
                .unwrap_or_else(|| field.is_required());

            if !is_required || effects.is_field_hidden(field_name) {
                continue;
            }

            if !object.contains_key(field_name) {
                return Err(AppError::Validation(format!(
                    "missing required field '{}'",
                    field_name
                )));
            }
        }

        Ok(())
    }

    pub(super) fn apply_entity_business_rule_value_patches(
        schema: &PublishedEntitySchema,
        object: &mut serde_json::Map<String, Value>,
        effects: &EntityBusinessRuleEffects,
    ) -> AppResult<()> {
        for (field_logical_name, patched_value) in &effects.value_patches {
            let Some(field) = schema
                .fields()
                .iter()
                .find(|field| field.logical_name().as_str() == field_logical_name)
            else {
                continue;
            };

            if field.calculation_expression().is_some() {
                continue;
            }

            field.validate_runtime_value(patched_value)?;
            Self::validate_choice_value_against_option_set(schema, field, patched_value)?;
            object.insert(field_logical_name.clone(), patched_value.clone());
        }

        Ok(())
    }

    pub(super) fn preserve_hidden_or_locked_update_values(
        schema: &PublishedEntitySchema,
        existing_record_data: &Value,
        object: &mut serde_json::Map<String, Value>,
        effects: &EntityBusinessRuleEffects,
    ) -> AppResult<()> {
        let existing_object = existing_record_data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;
        let published_field_names: BTreeSet<&str> = schema
            .fields()
            .iter()
            .map(|field| field.logical_name().as_str())
            .collect();

        for (field_logical_name, is_visible) in &effects.visibility_overrides {
            if *is_visible || object.contains_key(field_logical_name) {
                continue;
            }

            if !published_field_names.contains(field_logical_name.as_str()) {
                continue;
            }

            if let Some(existing_value) = existing_object.get(field_logical_name) {
                object.insert(field_logical_name.clone(), existing_value.clone());
            }
        }

        for (field_logical_name, is_locked) in &effects.lock_overrides {
            if !*is_locked || object.contains_key(field_logical_name) {
                continue;
            }

            if !published_field_names.contains(field_logical_name.as_str()) {
                continue;
            }

            if let Some(existing_value) = existing_object.get(field_logical_name) {
                object.insert(field_logical_name.clone(), existing_value.clone());
            }
        }

        Ok(())
    }

    pub(super) fn enforce_locked_field_changes(
        schema: &PublishedEntitySchema,
        existing_record_data: &Value,
        object: &serde_json::Map<String, Value>,
        effects: &EntityBusinessRuleEffects,
    ) -> AppResult<()> {
        let existing_object = existing_record_data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;
        let published_field_names: BTreeSet<&str> = schema
            .fields()
            .iter()
            .map(|field| field.logical_name().as_str())
            .collect();

        for (field_logical_name, is_locked) in &effects.lock_overrides {
            if !*is_locked || effects.value_patches.contains_key(field_logical_name) {
                continue;
            }

            if !published_field_names.contains(field_logical_name.as_str()) {
                continue;
            }

            let existing_value = existing_object.get(field_logical_name);
            let next_value = object.get(field_logical_name);
            if existing_value != next_value {
                return Err(AppError::Validation(format!(
                    "business rule lock prevents updating field '{}'",
                    field_logical_name
                )));
            }
        }

        Ok(())
    }
}
