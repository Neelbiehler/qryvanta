use super::*;

use super::scope::{ScopeFieldTypes, normalize_scope_alias};

pub(super) fn runtime_record_filter_from_request(
    condition: crate::dto::RuntimeRecordQueryFilterRequest,
    root_entity_logical_name: &str,
    scope_field_types: &ScopeFieldTypes,
    validate_field_exists: bool,
) -> Result<qryvanta_application::RuntimeRecordFilter, AppError> {
    let scope_alias = normalize_scope_alias(condition.scope_alias, "condition")?;
    let scope_key = scope_alias.clone().unwrap_or_default();
    let operator =
        qryvanta_application::RuntimeRecordOperator::parse_transport(condition.operator.as_str())?;

    let field_type = scope_field_types
        .get(&scope_key)
        .and_then(|field_types| field_types.get(condition.field_logical_name.as_str()))
        .copied();

    let field_type = match (validate_field_exists, field_type, scope_alias.as_deref()) {
        (_, Some(field_type), _) => field_type,
        (true, None, Some(alias)) => {
            return Err(AppError::Validation(format!(
                "unknown filter field '{}' for alias '{}'",
                condition.field_logical_name, alias
            )));
        }
        (true, None, None) => {
            return Err(AppError::Validation(format!(
                "unknown filter field '{}' for entity '{}'",
                condition.field_logical_name, root_entity_logical_name
            )));
        }
        (false, None, _) => qryvanta_domain::FieldType::Json,
    };

    Ok(qryvanta_application::RuntimeRecordFilter {
        scope_alias,
        field_logical_name: condition.field_logical_name,
        operator,
        field_type,
        field_value: condition.field_value,
    })
}

pub(super) fn runtime_record_group_from_request(
    group: crate::dto::RuntimeRecordQueryGroupRequest,
    root_entity_logical_name: &str,
    scope_field_types: &ScopeFieldTypes,
) -> Result<qryvanta_application::RuntimeRecordConditionGroup, AppError> {
    let logical_mode = group
        .logical_mode
        .as_deref()
        .map(qryvanta_application::RuntimeRecordLogicalMode::parse_transport)
        .transpose()?
        .unwrap_or(qryvanta_application::RuntimeRecordLogicalMode::And);

    let mut nodes = Vec::new();
    for condition in group.conditions.unwrap_or_default() {
        nodes.push(qryvanta_application::RuntimeRecordConditionNode::Filter(
            runtime_record_filter_from_request(
                condition,
                root_entity_logical_name,
                scope_field_types,
                true,
            )?,
        ));
    }

    for nested_group in group.groups.unwrap_or_default() {
        nodes.push(qryvanta_application::RuntimeRecordConditionNode::Group(
            runtime_record_group_from_request(
                nested_group,
                root_entity_logical_name,
                scope_field_types,
            )?,
        ));
    }

    if nodes.is_empty() {
        return Err(AppError::Validation(
            "runtime query where clause must include at least one condition or nested group"
                .to_owned(),
        ));
    }

    Ok(qryvanta_application::RuntimeRecordConditionGroup {
        logical_mode,
        nodes,
    })
}
