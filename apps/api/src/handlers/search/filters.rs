use std::collections::BTreeMap;

use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{EntityDefinition, EntityFieldDefinition, FieldType, OptionSetDefinition};
use serde::Serialize;

use crate::state::AppState;

pub(super) async fn plan_filters_for_query(
    state: &AppState,
    user: &UserIdentity,
    query: &str,
) -> Result<SearchPlan, AppError> {
    let normalized_query = normalize_for_match(query);
    if normalized_query.is_empty() {
        return Ok(SearchPlan {
            filters: None,
            normalized_query,
            selected_entity: None,
            planned_filter_count: 0,
            negated_filter_count: 0,
        });
    }

    let entities = state.metadata_service.list_entities(user).await?;
    let selected = select_entity_for_query(&entities, normalized_query.as_str());
    let Some(entity) = selected else {
        return Ok(SearchPlan {
            filters: None,
            normalized_query,
            selected_entity: None,
            planned_filter_count: 0,
            negated_filter_count: 0,
        });
    };

    let entity_logical_name = entity.logical_name().as_str().to_owned();
    let fields = state
        .metadata_service
        .list_fields(user, entity_logical_name.as_str())
        .await?;
    let option_sets = state
        .metadata_service
        .list_option_sets(user, entity_logical_name.as_str())
        .await?;

    let filters = derive_filters_from_metadata(
        entity_logical_name.as_str(),
        normalized_query.as_str(),
        &fields,
        &option_sets,
    );
    let planned_filter_count =
        filters.eq.len() + filters.in_values.values().map(Vec::len).sum::<usize>();
    let negated_filter_count = filters.not_in_values.values().map(Vec::len).sum::<usize>();

    if filters.eq.is_empty() && filters.in_values.is_empty() && filters.not_in_values.is_empty() {
        return Ok(SearchPlan {
            filters: None,
            normalized_query,
            selected_entity: Some(entity_logical_name),
            planned_filter_count,
            negated_filter_count,
        });
    }

    Ok(SearchPlan {
        filters: Some(filters),
        normalized_query,
        selected_entity: Some(entity_logical_name),
        planned_filter_count,
        negated_filter_count,
    })
}

fn select_entity_for_query<'a>(
    entities: &'a [EntityDefinition],
    normalized_query: &str,
) -> Option<&'a EntityDefinition> {
    entities
        .iter()
        .map(|entity| {
            let mut score = 0_i32;
            let aliases = entity_aliases(entity);
            for alias in aliases {
                if alias.is_empty() {
                    continue;
                }
                if normalized_query.contains(alias.as_str()) {
                    score += 10;
                }
                let alias_tokens = tokenize_query(alias.as_str());
                let overlap = alias_tokens
                    .iter()
                    .filter(|token| normalized_query.contains(token.as_str()))
                    .count() as i32;
                score += overlap;
            }

            (entity, score)
        })
        .filter(|(_, score)| *score > 0)
        .max_by_key(|(_, score)| *score)
        .map(|(entity, _)| entity)
}

fn entity_aliases(entity: &EntityDefinition) -> Vec<String> {
    let mut aliases = vec![
        normalize_for_match(entity.logical_name().as_str()),
        normalize_for_match(entity.display_name().as_str()),
    ];
    if let Some(plural) = entity.plural_display_name() {
        aliases.push(normalize_for_match(plural.as_str()));
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

fn derive_filters_from_metadata(
    entity_logical_name: &str,
    normalized_query: &str,
    fields: &[EntityFieldDefinition],
    option_sets: &[OptionSetDefinition],
) -> QrywellSearchFilters {
    let mut eq = BTreeMap::new();
    eq.insert(
        "entity".to_owned(),
        entity_logical_name.trim().to_lowercase(),
    );

    let option_set_map = option_sets
        .iter()
        .map(|option_set| (option_set.logical_name().as_str(), option_set))
        .collect::<BTreeMap<_, _>>();

    let mut in_values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut not_in_values: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for field in fields {
        if !matches!(
            field.field_type(),
            FieldType::Choice | FieldType::MultiChoice
        ) {
            continue;
        }
        let Some(option_set_name) = field.option_set_logical_name().map(|value| value.as_str())
        else {
            continue;
        };
        let Some(option_set) = option_set_map.get(option_set_name) else {
            continue;
        };

        let facet_key = field.logical_name().as_str().to_owned();
        for option in option_set.options() {
            let normalized_label = normalize_for_match(option.label().as_str());
            if normalized_label.is_empty() || !normalized_query.contains(normalized_label.as_str())
            {
                continue;
            }

            let target = if is_negated_phrase(normalized_query, normalized_label.as_str()) {
                &mut not_in_values
            } else {
                &mut in_values
            };

            target
                .entry(facet_key.clone())
                .or_default()
                .push(option.value().to_string());
        }
    }

    dedup_filter_values(&mut in_values);
    dedup_filter_values(&mut not_in_values);

    QrywellSearchFilters {
        eq,
        in_values,
        not_in_values,
    }
}

fn dedup_filter_values(map: &mut BTreeMap<String, Vec<String>>) {
    for values in map.values_mut() {
        values.sort();
        values.dedup();
    }
}

fn normalize_for_match(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize_query(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(ToOwned::to_owned)
        .filter(|token| token.len() > 1)
        .collect()
}

fn is_negated_phrase(query: &str, phrase: &str) -> bool {
    ["not", "without", "excluding", "except", "no"]
        .iter()
        .any(|negation| query.contains(format!("{negation} {phrase}").as_str()))
}

#[derive(Debug)]
pub(super) struct SearchPlan {
    pub(super) filters: Option<QrywellSearchFilters>,
    pub(super) normalized_query: String,
    pub(super) selected_entity: Option<String>,
    pub(super) planned_filter_count: usize,
    pub(super) negated_filter_count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct QrywellSearchFilters {
    eq: BTreeMap<String, String>,
    in_values: BTreeMap<String, Vec<String>>,
    not_in_values: BTreeMap<String, Vec<String>>,
}
