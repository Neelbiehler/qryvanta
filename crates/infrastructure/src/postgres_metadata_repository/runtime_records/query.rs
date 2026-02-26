use std::collections::BTreeMap;

use sqlx::{Postgres, QueryBuilder};

use super::*;

impl PostgresMetadataRepository {
    pub(in super::super) async fn query_runtime_records_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let limit = i64::try_from(query.limit).map_err(|error| {
            AppError::Validation(format!("invalid runtime record query limit: {error}"))
        })?;
        let offset = i64::try_from(query.offset).map_err(|error| {
            AppError::Validation(format!("invalid runtime record query offset: {error}"))
        })?;

        let root_table_alias = "runtime_root";
        let mut scope_table_aliases = BTreeMap::new();
        let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
            "SELECT runtime_root.id, runtime_root.entity_logical_name, runtime_root.data FROM runtime_records runtime_root",
        );

        for (index, link) in query.links.iter().enumerate() {
            let table_alias = format!("runtime_link_{index}");
            let parent_table_alias = link
                .parent_alias
                .as_deref()
                .map(|alias| resolve_scope_alias(&scope_table_aliases, alias))
                .transpose()?
                .unwrap_or(root_table_alias);

            match link.join_type {
                RuntimeRecordJoinType::Inner => builder.push(" JOIN runtime_records "),
                RuntimeRecordJoinType::Left => builder.push(" LEFT JOIN runtime_records "),
            };
            builder.push(table_alias.as_str());
            builder.push(" ON ");
            builder.push(table_alias.as_str());
            builder.push(".tenant_id = ");
            builder.push(root_table_alias);
            builder.push(".tenant_id AND ");
            builder.push(table_alias.as_str());
            builder.push(".entity_logical_name = ");
            builder.push_bind(link.target_entity_logical_name.clone());
            builder.push(" AND ");
            builder.push(table_alias.as_str());
            builder.push(".id::text = ");
            builder.push(parent_table_alias);
            builder.push(".data ->> ");
            builder.push_bind(link.relation_field_logical_name.clone());

            scope_table_aliases.insert(link.alias.clone(), table_alias);
        }

        builder.push(" WHERE ");
        builder.push(root_table_alias);
        builder.push(".tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND ");
        builder.push(root_table_alias);
        builder.push(".entity_logical_name = ");
        builder.push_bind(entity_logical_name);

        if let Some(owner_subject) = query.owner_subject {
            builder.push(" AND ");
            builder.push(root_table_alias);
            builder.push(".created_by_subject = ");
            builder.push_bind(owner_subject);
        }

        if let Some(where_clause) = &query.where_clause {
            builder.push(" AND ");
            push_runtime_group_condition(
                &mut builder,
                where_clause,
                &scope_table_aliases,
                root_table_alias,
            )?;
        }

        if !query.filters.is_empty() {
            builder.push(" AND (");
            for (index, filter) in query.filters.iter().enumerate() {
                if index > 0 {
                    match query.logical_mode {
                        RuntimeRecordLogicalMode::And => builder.push(" AND "),
                        RuntimeRecordLogicalMode::Or => builder.push(" OR "),
                    };
                }

                let scope_table_alias = filter
                    .scope_alias
                    .as_deref()
                    .map(|alias| resolve_scope_alias(&scope_table_aliases, alias))
                    .transpose()?
                    .unwrap_or(root_table_alias);

                push_runtime_filter_condition(&mut builder, filter, scope_table_alias);
            }
            builder.push(')');
        }

        if query.sort.is_empty() {
            builder.push(" ORDER BY ");
            builder.push(root_table_alias);
            builder.push(".created_at DESC");
        } else {
            builder.push(" ORDER BY ");
            for (index, sort) in query.sort.iter().enumerate() {
                if index > 0 {
                    builder.push(", ");
                }
                let scope_table_alias = sort
                    .scope_alias
                    .as_deref()
                    .map(|alias| resolve_scope_alias(&scope_table_aliases, alias))
                    .transpose()?
                    .unwrap_or(root_table_alias);
                push_runtime_sort_clause(&mut builder, sort, scope_table_alias);
            }
            builder.push(", ");
            builder.push(root_table_alias);
            builder.push(".created_at DESC");
        }

        builder.push(" LIMIT ");
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let rows = builder
            .build_query_as::<RuntimeRecordRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to query runtime records for entity '{}' in tenant '{}': {error}",
                    entity_logical_name, tenant_id
                ))
            })?;

        rows.into_iter().map(runtime_record_from_row).collect()
    }
}

fn resolve_scope_alias<'a>(
    scope_table_aliases: &'a BTreeMap<String, String>,
    alias: &str,
) -> AppResult<&'a str> {
    scope_table_aliases
        .get(alias)
        .map(String::as_str)
        .ok_or_else(|| {
            AppError::Validation(format!("unknown runtime query scope alias '{}'", alias))
        })
}

fn push_runtime_group_condition(
    builder: &mut QueryBuilder<'_, Postgres>,
    group: &RuntimeRecordConditionGroup,
    scope_table_aliases: &BTreeMap<String, String>,
    root_table_alias: &str,
) -> AppResult<()> {
    builder.push('(');

    for (index, node) in group.nodes.iter().enumerate() {
        if index > 0 {
            match group.logical_mode {
                RuntimeRecordLogicalMode::And => builder.push(" AND "),
                RuntimeRecordLogicalMode::Or => builder.push(" OR "),
            };
        }

        match node {
            RuntimeRecordConditionNode::Filter(filter) => {
                let scope_table_alias = filter
                    .scope_alias
                    .as_deref()
                    .map(|alias| resolve_scope_alias(scope_table_aliases, alias))
                    .transpose()?
                    .unwrap_or(root_table_alias);
                push_runtime_filter_condition(builder, filter, scope_table_alias);
            }
            RuntimeRecordConditionNode::Group(nested_group) => {
                push_runtime_group_condition(
                    builder,
                    nested_group,
                    scope_table_aliases,
                    root_table_alias,
                )?;
            }
        }
    }

    builder.push(')');
    Ok(())
}

fn push_runtime_filter_condition(
    builder: &mut QueryBuilder<'_, Postgres>,
    filter: &RuntimeRecordFilter,
    scope_table_alias: &str,
) {
    match filter.operator {
        RuntimeRecordOperator::Eq => {
            builder.push(scope_table_alias);
            builder.push(".data -> ");
            builder.push_bind(filter.field_logical_name.clone());
            builder.push(" = ");
            builder.push_bind(filter.field_value.clone());
        }
        RuntimeRecordOperator::Neq => {
            builder.push(scope_table_alias);
            builder.push(".data -> ");
            builder.push_bind(filter.field_logical_name.clone());
            builder.push(" <> ");
            builder.push_bind(filter.field_value.clone());
        }
        RuntimeRecordOperator::Gt
        | RuntimeRecordOperator::Gte
        | RuntimeRecordOperator::Lt
        | RuntimeRecordOperator::Lte => {
            let operator = match filter.operator {
                RuntimeRecordOperator::Gt => ">",
                RuntimeRecordOperator::Gte => ">=",
                RuntimeRecordOperator::Lt => "<",
                RuntimeRecordOperator::Lte => "<=",
                _ => unreachable!(),
            };

            match filter.field_type {
                FieldType::Number => {
                    builder.push("(");
                    builder.push(scope_table_alias);
                    builder.push(".data ->> ");
                    builder.push_bind(filter.field_logical_name.clone());
                    builder.push(")::NUMERIC ");
                    builder.push(operator);
                    builder.push(" (");
                    builder.push_bind(filter.field_value.to_string());
                    builder.push(")::NUMERIC");
                }
                _ => {
                    builder.push(scope_table_alias);
                    builder.push(".data ->> ");
                    builder.push_bind(filter.field_logical_name.clone());
                    builder.push(' ');
                    builder.push(operator);
                    builder.push(' ');
                    builder.push_bind(filter.field_value.as_str().unwrap_or_default().to_owned());
                }
            }
        }
        RuntimeRecordOperator::Contains => {
            builder.push(scope_table_alias);
            builder.push(".data ->> ");
            builder.push_bind(filter.field_logical_name.clone());
            builder.push(" ILIKE ");
            builder.push_bind(format!(
                "%{}%",
                filter.field_value.as_str().unwrap_or_default()
            ));
        }
        RuntimeRecordOperator::In => {
            let values = filter.field_value.as_array().cloned().unwrap_or_default();
            builder.push('(');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    builder.push(" OR ");
                }

                builder.push(scope_table_alias);
                builder.push(".data -> ");
                builder.push_bind(filter.field_logical_name.clone());
                builder.push(" = ");
                builder.push_bind(value.clone());
            }
            builder.push(')');
        }
    }
}

fn push_runtime_sort_clause(
    builder: &mut QueryBuilder<'_, Postgres>,
    sort: &RuntimeRecordSort,
    scope_table_alias: &str,
) {
    match sort.field_type {
        FieldType::Number => {
            builder.push("(");
            builder.push(scope_table_alias);
            builder.push(".data ->> ");
            builder.push_bind(sort.field_logical_name.clone());
            builder.push(")::NUMERIC");
        }
        _ => {
            builder.push(scope_table_alias);
            builder.push(".data ->> ");
            builder.push_bind(sort.field_logical_name.clone());
        }
    }

    builder.push(' ');
    match sort.direction {
        RuntimeRecordSortDirection::Asc => builder.push("ASC"),
        RuntimeRecordSortDirection::Desc => builder.push("DESC"),
    };
}
