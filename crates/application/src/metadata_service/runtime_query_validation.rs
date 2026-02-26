use super::*;

impl MetadataService {
    pub(super) fn enforce_query_readable_fields(
        query: &RuntimeRecordQuery,
        scope_field_access: &BTreeMap<String, crate::RuntimeFieldAccess>,
    ) -> AppResult<()> {
        for filter in &query.filters {
            Self::enforce_scope_readable_field(
                scope_field_access,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
                "query filters",
            )?;
        }

        if let Some(where_clause) = &query.where_clause {
            Self::enforce_group_readable_fields(where_clause, scope_field_access)?;
        }

        for sort in &query.sort {
            Self::enforce_scope_readable_field(
                scope_field_access,
                sort.scope_alias.as_deref(),
                sort.field_logical_name.as_str(),
                "query sorting",
            )?;
        }

        Ok(())
    }

    fn enforce_group_readable_fields(
        group: &RuntimeRecordConditionGroup,
        scope_field_access: &BTreeMap<String, crate::RuntimeFieldAccess>,
    ) -> AppResult<()> {
        for node in &group.nodes {
            match node {
                RuntimeRecordConditionNode::Filter(filter) => Self::enforce_scope_readable_field(
                    scope_field_access,
                    filter.scope_alias.as_deref(),
                    filter.field_logical_name.as_str(),
                    "query filters",
                )?,
                RuntimeRecordConditionNode::Group(nested_group) => {
                    Self::enforce_group_readable_fields(nested_group, scope_field_access)?;
                }
            }
        }

        Ok(())
    }

    fn enforce_scope_readable_field(
        scope_field_access: &BTreeMap<String, crate::RuntimeFieldAccess>,
        scope_alias: Option<&str>,
        field_logical_name: &str,
        context: &str,
    ) -> AppResult<()> {
        let scope_key = scope_alias.unwrap_or_default();
        let Some(field_access) = scope_field_access.get(scope_key) else {
            return Ok(());
        };

        if field_access.readable_fields.contains(field_logical_name) {
            return Ok(());
        }

        if scope_key.is_empty() {
            return Err(AppError::Forbidden(format!(
                "field '{}' is not readable for {}",
                field_logical_name, context
            )));
        }

        Err(AppError::Forbidden(format!(
            "field '{}' is not readable for {} in alias '{}'",
            field_logical_name, context, scope_key
        )))
    }

    pub(super) fn validate_runtime_query_group(
        root_entity_logical_name: &str,
        alias_entities: &BTreeMap<String, String>,
        schema_cache: &BTreeMap<String, PublishedEntitySchema>,
        group: &RuntimeRecordConditionGroup,
    ) -> AppResult<()> {
        if group.nodes.is_empty() {
            return Err(AppError::Validation(
                "runtime query where clause must include at least one condition or nested group"
                    .to_owned(),
            ));
        }

        for node in &group.nodes {
            match node {
                RuntimeRecordConditionNode::Filter(filter) => {
                    let field = Self::resolve_query_field_definition(
                        root_entity_logical_name,
                        alias_entities,
                        schema_cache,
                        filter.scope_alias.as_deref(),
                        filter.field_logical_name.as_str(),
                        "filter",
                    )?;
                    Self::validate_runtime_query_filter(field, filter)?;
                }
                RuntimeRecordConditionNode::Group(nested_group) => {
                    Self::validate_runtime_query_group(
                        root_entity_logical_name,
                        alias_entities,
                        schema_cache,
                        nested_group,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn validate_runtime_query_filter(
        field: &EntityFieldDefinition,
        filter: &RuntimeRecordFilter,
    ) -> AppResult<()> {
        if field.field_type() != filter.field_type {
            return Err(AppError::Validation(format!(
                "query filter field type mismatch for '{}': expected '{}', got '{}'",
                filter.field_logical_name,
                field.field_type().as_str(),
                filter.field_type.as_str()
            )));
        }

        match filter.operator {
            RuntimeRecordOperator::Eq | RuntimeRecordOperator::Neq => {
                field.validate_runtime_value(&filter.field_value)?;
            }
            RuntimeRecordOperator::Gt
            | RuntimeRecordOperator::Gte
            | RuntimeRecordOperator::Lt
            | RuntimeRecordOperator::Lte => {
                if !matches!(
                    field.field_type(),
                    FieldType::Number | FieldType::Date | FieldType::DateTime
                ) {
                    return Err(AppError::Validation(format!(
                        "operator '{}' is not supported for field '{}' with type '{}'",
                        filter.operator.as_str(),
                        filter.field_logical_name,
                        field.field_type().as_str()
                    )));
                }

                field.validate_runtime_value(&filter.field_value)?;
            }
            RuntimeRecordOperator::Contains => {
                if field.field_type() != FieldType::Text {
                    return Err(AppError::Validation(format!(
                        "operator 'contains' requires text field type for '{}'",
                        filter.field_logical_name
                    )));
                }

                if !filter.field_value.is_string() {
                    return Err(AppError::Validation(format!(
                        "operator 'contains' requires string value for '{}'",
                        filter.field_logical_name
                    )));
                }
            }
            RuntimeRecordOperator::In => {
                let values = filter.field_value.as_array().ok_or_else(|| {
                    AppError::Validation(format!(
                        "operator 'in' requires array value for '{}'",
                        filter.field_logical_name
                    ))
                })?;

                if values.is_empty() {
                    return Err(AppError::Validation(format!(
                        "operator 'in' requires at least one value for '{}'",
                        filter.field_logical_name
                    )));
                }

                for value in values {
                    field.validate_runtime_value(value)?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn validate_runtime_query_sort(
        field: &EntityFieldDefinition,
        sort: &RuntimeRecordSort,
    ) -> AppResult<()> {
        if field.field_type() != sort.field_type {
            return Err(AppError::Validation(format!(
                "query sort field type mismatch for '{}': expected '{}', got '{}'",
                sort.field_logical_name,
                field.field_type().as_str(),
                sort.field_type.as_str()
            )));
        }

        if field.field_type() == FieldType::Json {
            return Err(AppError::Validation(format!(
                "sorting is not supported for json field '{}'",
                sort.field_logical_name
            )));
        }

        Ok(())
    }
}
