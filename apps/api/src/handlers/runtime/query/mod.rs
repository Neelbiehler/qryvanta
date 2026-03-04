use super::*;

use conditions::{runtime_record_filter_from_request, runtime_record_group_from_request};
use links::runtime_record_links_from_request;
use scope::{ScopeFieldTypes, normalize_scope_alias};

mod conditions;
mod links;
mod scope;

pub(crate) async fn runtime_record_query_from_request(
    metadata_service: &qryvanta_application::MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    payload: QueryRuntimeRecordsRequest,
    max_limit: usize,
) -> Result<qryvanta_application::RuntimeRecordQuery, AppError> {
    let QueryRuntimeRecordsRequest {
        limit,
        offset,
        logical_mode,
        where_clause,
        conditions,
        link_entities,
        sort,
        filters: legacy_filters,
    } = payload;

    let root_scope_key = String::new();
    let schema = metadata_service
        .latest_published_schema_unchecked(actor, entity_logical_name)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "entity '{}' must be published before runtime records can be queried",
                entity_logical_name
            ))
        })?;

    let mut scope_field_types: ScopeFieldTypes = std::collections::BTreeMap::new();
    scope_field_types.insert(
        root_scope_key.clone(),
        schema
            .fields()
            .iter()
            .map(|field| (field.logical_name().as_str().to_owned(), field.field_type()))
            .collect::<std::collections::BTreeMap<_, _>>(),
    );

    let links = runtime_record_links_from_request(
        metadata_service,
        actor,
        entity_logical_name,
        &schema,
        link_entities,
        &mut scope_field_types,
    )
    .await?;

    let mut filters = conditions
        .unwrap_or_default()
        .into_iter()
        .map(|condition| {
            runtime_record_filter_from_request(
                condition,
                entity_logical_name,
                &scope_field_types,
                true,
            )
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    filters.extend(legacy_filters.unwrap_or_default().into_iter().map(
        |(field_logical_name, field_value)| {
            qryvanta_application::RuntimeRecordFilter {
                scope_alias: None,
                field_type: scope_field_types
                    .get(&root_scope_key)
                    .and_then(|field_types| field_types.get(field_logical_name.as_str()))
                    .copied()
                    .unwrap_or(qryvanta_domain::FieldType::Json),
                field_logical_name,
                operator: qryvanta_application::RuntimeRecordOperator::Eq,
                field_value,
            }
        },
    ));

    let sort = sort
        .unwrap_or_default()
        .into_iter()
        .map(|entry| {
            let scope_alias = normalize_scope_alias(entry.scope_alias, "sort")?;
            let scope_key = scope_alias.clone().unwrap_or_default();
            let direction = entry
                .direction
                .as_deref()
                .map(qryvanta_application::RuntimeRecordSortDirection::parse_transport)
                .transpose()?
                .unwrap_or(qryvanta_application::RuntimeRecordSortDirection::Asc);

            let field_type = scope_field_types
                .get(&scope_key)
                .and_then(|field_types| field_types.get(entry.field_logical_name.as_str()))
                .copied()
                .ok_or_else(|| match scope_alias.as_deref() {
                    Some(alias) => AppError::Validation(format!(
                        "unknown sort field '{}' for alias '{}'",
                        entry.field_logical_name, alias
                    )),
                    None => AppError::Validation(format!(
                        "unknown sort field '{}' for entity '{}'",
                        entry.field_logical_name, entity_logical_name
                    )),
                })?;

            Ok(qryvanta_application::RuntimeRecordSort {
                scope_alias,
                field_logical_name: entry.field_logical_name,
                field_type,
                direction,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let where_clause = where_clause
        .map(|group| {
            runtime_record_group_from_request(group, entity_logical_name, &scope_field_types)
        })
        .transpose()?;

    let logical_mode = logical_mode
        .as_deref()
        .map(qryvanta_application::RuntimeRecordLogicalMode::parse_transport)
        .transpose()?
        .unwrap_or(qryvanta_application::RuntimeRecordLogicalMode::And);

    let requested_limit = limit.unwrap_or(50);
    if requested_limit == 0 {
        return Err(AppError::Validation(
            "runtime record query limit must be greater than zero".to_owned(),
        ));
    }
    let effective_limit = requested_limit.min(max_limit);

    Ok(qryvanta_application::RuntimeRecordQuery {
        limit: effective_limit,
        offset: offset.unwrap_or(0),
        logical_mode,
        where_clause,
        filters,
        links,
        sort,
        owner_subject: None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use proptest::prelude::*;
    use serde_json::json;

    use super::conditions::{
        runtime_record_filter_from_request, runtime_record_group_from_request,
    };
    use super::scope::{ScopeFieldTypes, normalize_scope_alias};
    use crate::dto::{RuntimeRecordQueryFilterRequest, RuntimeRecordQueryGroupRequest};

    fn scope_field_types() -> ScopeFieldTypes {
        BTreeMap::from([
            (
                String::new(),
                BTreeMap::from([
                    ("name".to_owned(), qryvanta_domain::FieldType::Text),
                    ("status".to_owned(), qryvanta_domain::FieldType::Choice),
                ]),
            ),
            (
                "owner".to_owned(),
                BTreeMap::from([("name".to_owned(), qryvanta_domain::FieldType::Text)]),
            ),
        ])
    }

    proptest! {
        #[test]
        fn normalize_scope_alias_trims_non_empty_aliases(
            left in proptest::string::string_regex("[ \\t\\n]{0,4}").unwrap_or_else(|_| unreachable!()),
            alias in proptest::string::string_regex("[A-Za-z0-9_]{1,20}").unwrap_or_else(|_| unreachable!()),
            right in proptest::string::string_regex("[ \\t\\n]{0,4}").unwrap_or_else(|_| unreachable!()),
        ) {
            let normalized = normalize_scope_alias(Some(format!("{left}{alias}{right}")), "condition")
                .unwrap_or_else(|_| unreachable!());

            prop_assert_eq!(normalized.as_deref(), Some(alias.as_str()));
        }

        #[test]
        fn normalize_scope_alias_rejects_whitespace_only_aliases(
            whitespace in proptest::string::string_regex("[ \\t\\n]{1,20}").unwrap_or_else(|_| unreachable!())
        ) {
            let result = normalize_scope_alias(Some(whitespace), "sort");
            prop_assert!(result.is_err());
        }

        #[test]
        fn runtime_record_filter_uses_json_fallback_when_field_validation_disabled(
            unknown_field in proptest::string::string_regex("[a-z_]{1,24}").unwrap_or_else(|_| unreachable!())
                .prop_filter("field must not exist in root scope", |value| value != "name" && value != "status")
        ) {
            let scope_map = scope_field_types();
            let filter = runtime_record_filter_from_request(
                RuntimeRecordQueryFilterRequest {
                    scope_alias: None,
                    field_logical_name: unknown_field.clone(),
                    operator: "eq".to_owned(),
                    field_value: json!(1),
                },
                "contact",
                &scope_map,
                false,
            )
            .unwrap_or_else(|_| unreachable!());

            prop_assert_eq!(filter.field_type, qryvanta_domain::FieldType::Json);
            prop_assert_eq!(filter.field_logical_name, unknown_field);
            prop_assert_eq!(filter.scope_alias, None);
        }

        #[test]
        fn runtime_record_group_defaults_to_and_and_preserves_filter_count(
            filters in prop::collection::vec(
                (
                    prop::sample::select(vec!["name".to_owned(), "status".to_owned()]),
                    prop::sample::select(vec![
                        "eq".to_owned(),
                        "neq".to_owned(),
                        "contains".to_owned(),
                        "in".to_owned(),
                    ]),
                    -1000i64..1000i64,
                ),
                1..8
            )
        ) {
            let scope_map = scope_field_types();
            let expected_count = filters.len();
            let request = RuntimeRecordQueryGroupRequest {
                logical_mode: None,
                conditions: Some(
                    filters
                        .into_iter()
                        .map(|(field_logical_name, operator, value)| RuntimeRecordQueryFilterRequest {
                            scope_alias: None,
                            field_logical_name,
                            operator,
                            field_value: json!(value),
                        })
                        .collect(),
                ),
                groups: None,
            };

            let group = runtime_record_group_from_request(request, "contact", &scope_map)
                .unwrap_or_else(|_| unreachable!());

            prop_assert_eq!(
                group.logical_mode,
                qryvanta_application::RuntimeRecordLogicalMode::And
            );
            prop_assert_eq!(group.nodes.len(), expected_count);
            prop_assert!(group.nodes.iter().all(|node| matches!(
                node,
                qryvanta_application::RuntimeRecordConditionNode::Filter(_)
            )));
        }
    }
}
