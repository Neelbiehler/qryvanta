use std::cmp::Ordering;

use super::*;

impl InMemoryMetadataRepository {
    pub(in super::super) async fn query_runtime_records_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let records = self.runtime_records.read().await;
        let record_owners = self.record_owners.read().await;
        let runtime_index = build_runtime_record_index(&records);
        let mut listed: Vec<RuntimeRecord> = collect_runtime_records_for_scope(
            &records,
            &record_owners,
            tenant_id,
            entity_logical_name,
            query.owner_subject.as_deref(),
        )
        .into_iter()
        .filter(|record| {
            let Some(scope_records) = resolve_runtime_query_scope_records(
                &query,
                tenant_id,
                entity_logical_name,
                record,
                &runtime_index,
            ) else {
                return false;
            };

            runtime_record_matches_filters(&scope_records, &query)
        })
        .collect();

        if query.sort.is_empty() {
            listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));
        } else {
            listed.sort_by(|left, right| {
                let left_scope_records = resolve_runtime_query_scope_records(
                    &query,
                    tenant_id,
                    entity_logical_name,
                    left,
                    &runtime_index,
                );
                let right_scope_records = resolve_runtime_query_scope_records(
                    &query,
                    tenant_id,
                    entity_logical_name,
                    right,
                    &runtime_index,
                );

                let Some(left_scope_records) = left_scope_records else {
                    return Ordering::Greater;
                };
                let Some(right_scope_records) = right_scope_records else {
                    return Ordering::Less;
                };

                for sort in &query.sort {
                    let ordering =
                        compare_values_for_sort(&left_scope_records, &right_scope_records, sort);
                    if ordering != Ordering::Equal {
                        return ordering;
                    }
                }

                left.record_id().as_str().cmp(right.record_id().as_str())
            });
        }

        Ok(listed
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect())
    }
}

fn build_runtime_record_index(
    records: &HashMap<(TenantId, String, String), RuntimeRecord>,
) -> HashMap<(TenantId, String, String), RuntimeRecord> {
    records.clone()
}

fn resolve_runtime_query_scope_records(
    query: &RuntimeRecordQuery,
    tenant_id: TenantId,
    root_entity_logical_name: &str,
    root_record: &RuntimeRecord,
    runtime_index: &HashMap<(TenantId, String, String), RuntimeRecord>,
) -> Option<HashMap<String, Option<RuntimeRecord>>> {
    let mut scope_records = HashMap::new();
    scope_records.insert(String::new(), Some(root_record.clone()));

    for link in &query.links {
        let parent_scope_key = link.parent_alias.clone().unwrap_or_default();
        let Some(parent_record) = scope_records
            .get(parent_scope_key.as_str())
            .and_then(|record| record.clone())
        else {
            if link.join_type == RuntimeRecordJoinType::Inner {
                return None;
            }

            scope_records.insert(link.alias.clone(), None);
            continue;
        };

        let relation_target_record_id = parent_record
            .data()
            .as_object()
            .and_then(|data| data.get(link.relation_field_logical_name.as_str()))
            .and_then(Value::as_str)
            .map(str::to_owned);

        let linked_record = relation_target_record_id.and_then(|record_id| {
            runtime_index
                .get(&(
                    tenant_id,
                    link.target_entity_logical_name.clone(),
                    record_id,
                ))
                .cloned()
        });

        if linked_record.is_none() && link.join_type == RuntimeRecordJoinType::Inner {
            return None;
        }

        scope_records.insert(link.alias.clone(), linked_record);
    }

    if root_entity_logical_name != root_record.entity_logical_name().as_str() {
        return None;
    }

    Some(scope_records)
}

fn runtime_record_matches_filters(
    scope_records: &HashMap<String, Option<RuntimeRecord>>,
    query: &RuntimeRecordQuery,
) -> bool {
    let matches_flat_filters = if query.filters.is_empty() {
        true
    } else {
        let evaluate = |filter: &RuntimeRecordFilter| {
            let value = resolve_scope_value(
                scope_records,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
            );

            runtime_record_filter_matches_value(value, filter)
        };

        match query.logical_mode {
            RuntimeRecordLogicalMode::And => query.filters.iter().all(evaluate),
            RuntimeRecordLogicalMode::Or => query.filters.iter().any(evaluate),
        }
    };

    if !matches_flat_filters {
        return false;
    }

    query
        .where_clause
        .as_ref()
        .map(|group| runtime_record_group_matches(group, scope_records))
        .unwrap_or(true)
}

fn runtime_record_group_matches(
    group: &RuntimeRecordConditionGroup,
    scope_records: &HashMap<String, Option<RuntimeRecord>>,
) -> bool {
    let evaluate = |node: &RuntimeRecordConditionNode| match node {
        RuntimeRecordConditionNode::Filter(filter) => {
            let value = resolve_scope_value(
                scope_records,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
            );
            runtime_record_filter_matches_value(value, filter)
        }
        RuntimeRecordConditionNode::Group(nested_group) => {
            runtime_record_group_matches(nested_group, scope_records)
        }
    };

    match group.logical_mode {
        RuntimeRecordLogicalMode::And => group.nodes.iter().all(evaluate),
        RuntimeRecordLogicalMode::Or => group.nodes.iter().any(evaluate),
    }
}

fn resolve_scope_value<'a>(
    scope_records: &'a HashMap<String, Option<RuntimeRecord>>,
    scope_alias: Option<&str>,
    field_logical_name: &str,
) -> Option<&'a Value> {
    let scope_key = scope_alias.unwrap_or_default();
    scope_records
        .get(scope_key)
        .and_then(Option::as_ref)
        .and_then(|record| record.data().as_object())
        .and_then(|data| data.get(field_logical_name))
}

fn runtime_record_filter_matches_value(
    value: Option<&Value>,
    filter: &RuntimeRecordFilter,
) -> bool {
    let Some(value) = value else {
        return false;
    };

    match filter.operator {
        RuntimeRecordOperator::Eq => value == &filter.field_value,
        RuntimeRecordOperator::Neq => value != &filter.field_value,
        RuntimeRecordOperator::Gt => {
            compare_filter_values(value, &filter.field_value, filter).is_gt()
        }
        RuntimeRecordOperator::Gte => {
            let comparison = compare_filter_values(value, &filter.field_value, filter);
            comparison.is_gt() || comparison.is_eq()
        }
        RuntimeRecordOperator::Lt => {
            compare_filter_values(value, &filter.field_value, filter).is_lt()
        }
        RuntimeRecordOperator::Lte => {
            let comparison = compare_filter_values(value, &filter.field_value, filter);
            comparison.is_lt() || comparison.is_eq()
        }
        RuntimeRecordOperator::Contains => value
            .as_str()
            .zip(filter.field_value.as_str())
            .map(|(stored, expected)| stored.contains(expected))
            .unwrap_or(false),
        RuntimeRecordOperator::In => filter
            .field_value
            .as_array()
            .map(|values| values.iter().any(|candidate| candidate == value))
            .unwrap_or(false),
    }
}

fn compare_filter_values(
    stored: &Value,
    expected: &Value,
    filter: &RuntimeRecordFilter,
) -> Ordering {
    match filter.field_type {
        FieldType::Number => stored
            .as_f64()
            .zip(expected.as_f64())
            .and_then(|(left, right)| left.partial_cmp(&right))
            .unwrap_or(Ordering::Equal),
        FieldType::Choice => stored
            .as_i64()
            .zip(expected.as_i64())
            .map(|(left, right)| left.cmp(&right))
            .unwrap_or(Ordering::Equal),
        FieldType::MultiChoice => Ordering::Equal,
        FieldType::Date | FieldType::DateTime | FieldType::Text | FieldType::Relation => stored
            .as_str()
            .zip(expected.as_str())
            .map(|(left, right)| left.cmp(right))
            .unwrap_or(Ordering::Equal),
        FieldType::Boolean => stored
            .as_bool()
            .zip(expected.as_bool())
            .map(|(left, right)| left.cmp(&right))
            .unwrap_or(Ordering::Equal),
        FieldType::Json => Ordering::Equal,
    }
}

fn compare_values_for_sort(
    left_scope_records: &HashMap<String, Option<RuntimeRecord>>,
    right_scope_records: &HashMap<String, Option<RuntimeRecord>>,
    sort: &RuntimeRecordSort,
) -> Ordering {
    let left_value = resolve_scope_value(
        left_scope_records,
        sort.scope_alias.as_deref(),
        sort.field_logical_name.as_str(),
    );
    let right_value = resolve_scope_value(
        right_scope_records,
        sort.scope_alias.as_deref(),
        sort.field_logical_name.as_str(),
    );

    let mut ordering = match (left_value, right_value) {
        (Some(left), Some(right)) => match sort.field_type {
            FieldType::Number => left
                .as_f64()
                .zip(right.as_f64())
                .and_then(|(left, right)| left.partial_cmp(&right))
                .unwrap_or(Ordering::Equal),
            FieldType::Choice => left
                .as_i64()
                .zip(right.as_i64())
                .map(|(left, right)| left.cmp(&right))
                .unwrap_or(Ordering::Equal),
            FieldType::MultiChoice => Ordering::Equal,
            FieldType::Boolean => left
                .as_bool()
                .zip(right.as_bool())
                .map(|(left, right)| left.cmp(&right))
                .unwrap_or(Ordering::Equal),
            FieldType::Date | FieldType::DateTime | FieldType::Text | FieldType::Relation => left
                .as_str()
                .zip(right.as_str())
                .map(|(left, right)| left.cmp(right))
                .unwrap_or(Ordering::Equal),
            FieldType::Json => Ordering::Equal,
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };

    if sort.direction == RuntimeRecordSortDirection::Desc {
        ordering = ordering.reverse();
    }

    ordering
}
