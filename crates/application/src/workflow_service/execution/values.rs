use super::*;

impl WorkflowService {
    pub(super) fn interpolate_step(
        step: &WorkflowStep,
        context: WorkflowExecutionContext<'_>,
    ) -> AppResult<WorkflowStep> {
        match step {
            WorkflowStep::LogMessage { message } => Ok(WorkflowStep::LogMessage {
                message: Self::interpolate_string(message, context),
            }),
            WorkflowStep::CreateRuntimeRecord {
                entity_logical_name,
                data,
            } => Ok(WorkflowStep::CreateRuntimeRecord {
                entity_logical_name: Self::interpolate_string(entity_logical_name, context),
                data: Self::interpolate_json_value(data, context)?,
            }),
            WorkflowStep::UpdateRuntimeRecord {
                entity_logical_name,
                record_id,
                data,
            } => Ok(WorkflowStep::UpdateRuntimeRecord {
                entity_logical_name: Self::interpolate_string(entity_logical_name, context),
                record_id: Self::interpolate_string(record_id, context),
                data: Self::interpolate_json_value(data, context)?,
            }),
            WorkflowStep::DeleteRuntimeRecord {
                entity_logical_name,
                record_id,
            } => Ok(WorkflowStep::DeleteRuntimeRecord {
                entity_logical_name: Self::interpolate_string(entity_logical_name, context),
                record_id: Self::interpolate_string(record_id, context),
            }),
            WorkflowStep::SendEmail {
                to,
                subject,
                body,
                html_body,
            } => Ok(WorkflowStep::SendEmail {
                to: Self::interpolate_string(to, context),
                subject: Self::interpolate_string(subject, context),
                body: Self::interpolate_string(body, context),
                html_body: html_body
                    .as_ref()
                    .map(|value| Self::interpolate_string(value, context)),
            }),
            WorkflowStep::HttpRequest {
                method,
                url,
                headers,
                header_secret_refs,
                body,
            } => Ok(WorkflowStep::HttpRequest {
                method: Self::interpolate_string(method, context),
                url: Self::interpolate_string(url, context),
                headers: headers
                    .as_ref()
                    .map(|value| Self::interpolate_json_value(value, context))
                    .transpose()?,
                header_secret_refs: header_secret_refs.clone(),
                body: body
                    .as_ref()
                    .map(|value| Self::interpolate_json_value(value, context))
                    .transpose()?,
            }),
            WorkflowStep::Webhook {
                endpoint,
                event,
                headers,
                header_secret_refs,
                payload,
            } => Ok(WorkflowStep::Webhook {
                endpoint: Self::interpolate_string(endpoint, context),
                event: Self::interpolate_string(event, context),
                headers: headers
                    .as_ref()
                    .map(|value| Self::interpolate_json_value(value, context))
                    .transpose()?,
                header_secret_refs: header_secret_refs.clone(),
                payload: Self::interpolate_json_value(payload, context)?,
            }),
            WorkflowStep::AssignOwner {
                entity_logical_name,
                record_id,
                owner_id,
                reason,
            } => Ok(WorkflowStep::AssignOwner {
                entity_logical_name: Self::interpolate_string(entity_logical_name, context),
                record_id: Self::interpolate_string(record_id, context),
                owner_id: Self::interpolate_string(owner_id, context),
                reason: reason
                    .as_ref()
                    .map(|value| Self::interpolate_string(value, context)),
            }),
            WorkflowStep::ApprovalRequest {
                entity_logical_name,
                record_id,
                request_type,
                requested_by,
                approver_id,
                reason,
                payload,
            } => Ok(WorkflowStep::ApprovalRequest {
                entity_logical_name: Self::interpolate_string(entity_logical_name, context),
                record_id: Self::interpolate_string(record_id, context),
                request_type: Self::interpolate_string(request_type, context),
                requested_by: requested_by
                    .as_ref()
                    .map(|value| Self::interpolate_string(value, context)),
                approver_id: approver_id
                    .as_ref()
                    .map(|value| Self::interpolate_string(value, context)),
                reason: reason
                    .as_ref()
                    .map(|value| Self::interpolate_string(value, context)),
                payload: payload
                    .as_ref()
                    .map(|value| Self::interpolate_json_value(value, context))
                    .transpose()?,
            }),
            WorkflowStep::Delay {
                duration_ms,
                reason,
            } => Ok(WorkflowStep::Delay {
                duration_ms: *duration_ms,
                reason: reason
                    .as_ref()
                    .map(|value| Self::interpolate_string(value, context)),
            }),
            WorkflowStep::Condition { .. } => Err(AppError::Validation(
                "condition step cannot be interpolated as an executable action".to_owned(),
            )),
        }
    }

    pub(super) fn interpolate_json_value(
        value: &Value,
        context: WorkflowExecutionContext<'_>,
    ) -> AppResult<Value> {
        match value {
            Value::Null => Ok(Value::Null),
            Value::Bool(flag) => Ok(Value::Bool(*flag)),
            Value::Number(number) => Ok(Value::Number(number.clone())),
            Value::String(content) => {
                if let Some(token_name) = Self::single_token_name(content)
                    && let Some(token_value) = Self::token_value(token_name, context)
                {
                    return Ok(token_value);
                }

                Ok(Value::String(Self::interpolate_string(content, context)))
            }
            Value::Array(items) => items
                .iter()
                .map(|item| Self::interpolate_json_value(item, context))
                .collect::<AppResult<Vec<Value>>>()
                .map(Value::Array),
            Value::Object(map) => {
                let mut interpolated = serde_json::Map::with_capacity(map.len());
                for (key, value) in map {
                    interpolated.insert(key.clone(), Self::interpolate_json_value(value, context)?);
                }

                Ok(Value::Object(interpolated))
            }
        }
    }

    pub(super) fn interpolate_string(value: &str, context: WorkflowExecutionContext<'_>) -> String {
        let mut result = String::with_capacity(value.len());
        let mut rest = value;

        while let Some(start) = rest.find("{{") {
            let (head, after_head) = rest.split_at(start);
            result.push_str(head);

            let Some(end_relative) = after_head.find("}}") else {
                result.push_str(after_head);
                rest = "";
                break;
            };

            let token = &after_head[2..end_relative].trim();
            if let Some(token_value) = Self::token_value(token, context) {
                result.push_str(Self::value_to_string(&token_value).as_str());
            } else {
                result.push_str(&after_head[..end_relative + 2]);
            }

            rest = &after_head[end_relative + 2..];
        }

        result.push_str(rest);
        result
    }

    pub(super) fn single_token_name(value: &str) -> Option<&str> {
        let trimmed = value.trim();
        if !trimmed.starts_with("{{") || !trimmed.ends_with("}}") {
            return None;
        }

        let token = trimmed[2..trimmed.len().saturating_sub(2)].trim();
        if token.is_empty() {
            return None;
        }

        Some(token)
    }

    pub(super) fn token_value(token: &str, context: WorkflowExecutionContext<'_>) -> Option<Value> {
        match token {
            "trigger.type" => Some(Value::String(context.trigger_type.to_owned())),
            "trigger.entity" => Some(Value::String(
                context
                    .trigger_entity_logical_name
                    .unwrap_or_default()
                    .to_owned(),
            )),
            "run.id" => Some(Value::String(context.run_id.to_owned())),
            "run.attempt" => Some(Value::Number(context.attempt_number.into())),
            "now.iso" => Some(Value::String(Utc::now().to_rfc3339())),
            _ => {
                let path = token
                    .strip_prefix("trigger.payload.")
                    .or_else(|| token.strip_prefix("trigger."));
                path.and_then(|selected_path| {
                    Self::payload_value_by_path(context.trigger_payload, selected_path).cloned()
                })
            }
        }
    }

    pub(super) fn value_to_string(value: &Value) -> String {
        match value {
            Value::Null => "null".to_owned(),
            Value::Bool(flag) => flag.to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(content) => content.clone(),
            Value::Array(_) | Value::Object(_) => value.to_string(),
        }
    }

    pub(super) fn evaluate_condition(
        trigger_payload: &Value,
        field_path: &str,
        operator: WorkflowConditionOperator,
        value: Option<&Value>,
    ) -> AppResult<bool> {
        let selected_value = Self::payload_value_by_path(trigger_payload, field_path);
        match operator {
            WorkflowConditionOperator::Exists => Ok(selected_value.is_some()),
            WorkflowConditionOperator::Equals => {
                let expected_value = value.ok_or_else(|| {
                    AppError::Validation(
                        "workflow condition equals operator requires a comparison value".to_owned(),
                    )
                })?;

                Ok(selected_value == Some(expected_value))
            }
            WorkflowConditionOperator::NotEquals => {
                let expected_value = value.ok_or_else(|| {
                    AppError::Validation(
                        "workflow condition not_equals operator requires a comparison value"
                            .to_owned(),
                    )
                })?;

                Ok(selected_value != Some(expected_value))
            }
        }
    }

    pub(super) fn payload_value_by_path<'a>(
        payload: &'a Value,
        field_path: &str,
    ) -> Option<&'a Value> {
        let mut current_value = payload;
        for segment in field_path.split('.') {
            if segment.is_empty() {
                return None;
            }

            current_value = current_value.as_object()?.get(segment)?;
        }

        Some(current_value)
    }

    pub(super) fn step_by_path<'a>(
        steps: &'a [WorkflowStep],
        step_path: &str,
    ) -> AppResult<&'a WorkflowStep> {
        let mut branch_steps = steps;
        let mut selected_step: Option<&WorkflowStep> = None;

        for segment in step_path.split('.') {
            if segment == "then" {
                let Some(WorkflowStep::Condition { then_steps, .. }) = selected_step else {
                    return Err(AppError::Validation(format!(
                        "invalid workflow step path '{}': expected condition for then branch",
                        step_path
                    )));
                };

                branch_steps = then_steps.as_slice();
                selected_step = None;
                continue;
            }

            if segment == "else" {
                let Some(WorkflowStep::Condition { else_steps, .. }) = selected_step else {
                    return Err(AppError::Validation(format!(
                        "invalid workflow step path '{}': expected condition for else branch",
                        step_path
                    )));
                };

                branch_steps = else_steps.as_slice();
                selected_step = None;
                continue;
            }

            let index = segment.parse::<usize>().map_err(|error| {
                AppError::Validation(format!(
                    "invalid workflow step path '{}': segment '{}' is not an index ({error})",
                    step_path, segment
                ))
            })?;

            let step = branch_steps.get(index).ok_or_else(|| {
                AppError::Validation(format!(
                    "invalid workflow step path '{}': index {} is out of range",
                    step_path, index
                ))
            })?;

            selected_step = Some(step);
        }

        selected_step.ok_or_else(|| {
            AppError::Validation(format!(
                "invalid workflow step path '{}': no step resolved",
                step_path
            ))
        })
    }
}
