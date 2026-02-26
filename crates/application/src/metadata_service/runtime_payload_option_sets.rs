use super::*;

impl MetadataService {
    pub(super) fn validate_choice_value_against_option_set(
        schema: &PublishedEntitySchema,
        field: &EntityFieldDefinition,
        value: &Value,
    ) -> AppResult<()> {
        let Some(option_set_logical_name) = field.option_set_logical_name() else {
            return Ok(());
        };
        let Some(option_set) = schema
            .option_sets()
            .iter()
            .find(|set| set.logical_name().as_str() == option_set_logical_name.as_str())
        else {
            return Err(AppError::Validation(format!(
                "field '{}.{}' references unknown option set '{}'",
                field.entity_logical_name().as_str(),
                field.logical_name().as_str(),
                option_set_logical_name.as_str()
            )));
        };

        match field.field_type() {
            FieldType::Choice => {
                let selected = value.as_i64().ok_or_else(|| {
                    AppError::Validation(format!(
                        "choice field '{}' requires integer value",
                        field.logical_name().as_str()
                    ))
                })?;
                let selected = i32::try_from(selected).map_err(|_| {
                    AppError::Validation(format!(
                        "choice field '{}' value is out of supported range",
                        field.logical_name().as_str()
                    ))
                })?;
                if !option_set.contains_value(selected) {
                    return Err(AppError::Validation(format!(
                        "choice field '{}' includes unknown option value '{}'",
                        field.logical_name().as_str(),
                        selected
                    )));
                }
            }
            FieldType::MultiChoice => {
                let selected_values = value.as_array().ok_or_else(|| {
                    AppError::Validation(format!(
                        "multichoice field '{}' requires array value",
                        field.logical_name().as_str()
                    ))
                })?;
                for selected in selected_values {
                    let selected = selected.as_i64().ok_or_else(|| {
                        AppError::Validation(format!(
                            "multichoice field '{}' values must be integers",
                            field.logical_name().as_str()
                        ))
                    })?;
                    let selected = i32::try_from(selected).map_err(|_| {
                        AppError::Validation(format!(
                            "multichoice field '{}' value is out of supported range",
                            field.logical_name().as_str()
                        ))
                    })?;
                    if !option_set.contains_value(selected) {
                        return Err(AppError::Validation(format!(
                            "multichoice field '{}' includes unknown option value '{}'",
                            field.logical_name().as_str(),
                            selected
                        )));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}
