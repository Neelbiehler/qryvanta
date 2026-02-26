use super::*;

impl MetadataService {
    pub(super) fn apply_calculated_field_values(
        schema: &PublishedEntitySchema,
        object: &mut serde_json::Map<String, Value>,
    ) -> AppResult<()> {
        for field in schema.fields() {
            let Some(expression) = field.calculation_expression() else {
                continue;
            };

            let calculated_value = Self::evaluate_calculation_expression(expression, object)?;
            field.validate_runtime_value(&calculated_value)?;
            object.insert(field.logical_name().as_str().to_owned(), calculated_value);
        }

        Ok(())
    }

    fn evaluate_calculation_expression(
        expression: &str,
        object: &serde_json::Map<String, Value>,
    ) -> AppResult<Value> {
        if let Some(args) = Self::parse_calculation_call(expression, "add")? {
            let mut sum = 0.0_f64;
            for token in args {
                let value = Self::resolve_calculation_token(token.as_str(), object)?;
                let numeric = if value.is_null() {
                    0.0
                } else if let Some(number) = value.as_f64() {
                    number
                } else if let Some(raw) = value.as_str() {
                    raw.parse::<f64>().map_err(|_| {
                        AppError::Validation(format!(
                            "calculation expression '{}' expects numeric values",
                            expression
                        ))
                    })?
                } else {
                    return Err(AppError::Validation(format!(
                        "calculation expression '{}' expects numeric values",
                        expression
                    )));
                };

                sum += numeric;
            }

            let Some(number) = serde_json::Number::from_f64(sum) else {
                return Err(AppError::Validation(format!(
                    "calculation expression '{}' produced invalid numeric output",
                    expression
                )));
            };

            return Ok(Value::Number(number));
        }

        if let Some(args) = Self::parse_calculation_call(expression, "concat")? {
            let mut output = String::new();
            for token in args {
                let value = Self::resolve_calculation_token(token.as_str(), object)?;
                if value.is_null() {
                    continue;
                }

                if let Some(text) = value.as_str() {
                    output.push_str(text);
                } else if let Some(number) = value.as_f64() {
                    output.push_str(number.to_string().as_str());
                } else if let Some(boolean) = value.as_bool() {
                    output.push_str(if boolean { "true" } else { "false" });
                } else {
                    output.push_str(value.to_string().as_str());
                }
            }

            return Ok(Value::String(output));
        }

        Err(AppError::Validation(format!(
            "unsupported calculation expression '{}'",
            expression
        )))
    }

    fn parse_calculation_call(
        expression: &str,
        function_name: &str,
    ) -> AppResult<Option<Vec<String>>> {
        let trimmed = expression.trim();
        let prefix = format!("{function_name}(");
        if !trimmed.starts_with(prefix.as_str()) {
            return Ok(None);
        }

        if !trimmed.ends_with(')') {
            return Err(AppError::Validation(format!(
                "calculation expression '{}' has invalid syntax",
                expression
            )));
        }

        let inner = &trimmed[prefix.len()..trimmed.len() - 1];
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut escaped = false;

        for character in inner.chars() {
            if escaped {
                current.push(character);
                escaped = false;
                continue;
            }

            if character == '\\' && in_string {
                current.push(character);
                escaped = true;
                continue;
            }

            if character == '"' {
                in_string = !in_string;
                current.push(character);
                continue;
            }

            if character == ',' && !in_string {
                let token = current.trim();
                if token.is_empty() {
                    return Err(AppError::Validation(format!(
                        "calculation expression '{}' contains empty argument",
                        expression
                    )));
                }
                args.push(token.to_owned());
                current.clear();
                continue;
            }

            current.push(character);
        }

        if in_string {
            return Err(AppError::Validation(format!(
                "calculation expression '{}' has unclosed string literal",
                expression
            )));
        }

        let token = current.trim();
        if token.is_empty() {
            return Err(AppError::Validation(format!(
                "calculation expression '{}' requires at least one argument",
                expression
            )));
        }
        args.push(token.to_owned());

        Ok(Some(args))
    }

    fn resolve_calculation_token(
        token: &str,
        object: &serde_json::Map<String, Value>,
    ) -> AppResult<Value> {
        let trimmed = token.trim();
        if trimmed.starts_with('"') {
            let parsed = serde_json::from_str::<String>(trimmed).map_err(|_| {
                AppError::Validation(format!(
                    "invalid string literal '{}' in calculation expression",
                    token
                ))
            })?;
            return Ok(Value::String(parsed));
        }

        if let Ok(number) = trimmed.parse::<f64>() {
            let Some(number) = serde_json::Number::from_f64(number) else {
                return Err(AppError::Validation(format!(
                    "invalid numeric literal '{}' in calculation expression",
                    token
                )));
            };
            return Ok(Value::Number(number));
        }

        Ok(object.get(trimmed).cloned().unwrap_or(Value::Null))
    }
}
