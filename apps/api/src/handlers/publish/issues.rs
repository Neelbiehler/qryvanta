use std::collections::HashSet;

use qryvanta_core::UserIdentity;

use crate::dto::{
    PublishCheckCategoryDto, PublishCheckIssueResponse, PublishCheckScopeDto,
    PublishCheckSeverityDto,
};
use crate::error::ApiResult;

use super::PublishState;

pub(super) async fn collect_workspace_issues(
    state: &PublishState,
    user: &UserIdentity,
    entity_logical_names: &[String],
    app_logical_names: &[String],
) -> ApiResult<Vec<PublishCheckIssueResponse>> {
    let mut issues = Vec::new();
    let mut publishable_entity_logical_names = Vec::new();

    for entity_logical_name in entity_logical_names {
        let errors = state
            .metadata_service
            .publish_checks_with_allowed_unpublished_entities(
                user,
                entity_logical_name.as_str(),
                entity_logical_names,
            )
            .await?;
        if errors.is_empty() {
            publishable_entity_logical_names.push(entity_logical_name.clone());
        }
        for message in errors {
            let category = classify_issue(PublishCheckScopeDto::Entity, message.as_str());
            let dependency_path = extract_dependency_path(message.as_str());
            issues.push(PublishCheckIssueResponse {
                scope: PublishCheckScopeDto::Entity,
                scope_logical_name: entity_logical_name.clone(),
                category,
                severity: PublishCheckSeverityDto::Error,
                fix_path: Some(entity_fix_path(entity_logical_name, category)),
                message,
                dependency_path,
            });
        }
    }

    for app_logical_name in app_logical_names {
        let errors = state
            .app_service
            .publish_checks_with_allowed_unpublished_entities(
                user,
                app_logical_name.as_str(),
                &publishable_entity_logical_names,
            )
            .await?;
        for message in errors {
            let category = classify_issue(PublishCheckScopeDto::App, message.as_str());
            issues.push(PublishCheckIssueResponse {
                scope: PublishCheckScopeDto::App,
                scope_logical_name: app_logical_name.clone(),
                category,
                severity: PublishCheckSeverityDto::Error,
                fix_path: Some(app_fix_path(app_logical_name, category)),
                dependency_path: extract_dependency_path(message.as_str()),
                message,
            });
        }
    }

    Ok(issues)
}

pub(super) fn resolve_requested_names(
    requested: Vec<String>,
    fallback: Vec<String>,
) -> Vec<String> {
    if requested.is_empty() {
        return fallback;
    }

    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for logical_name in requested {
        if seen.insert(logical_name.clone()) {
            unique.push(logical_name);
        }
    }

    unique
}

pub(super) fn partition_known_names(
    requested: &[String],
    available: &[String],
) -> (Vec<String>, Vec<String>) {
    let available_set = available.iter().map(String::as_str).collect::<HashSet<_>>();
    let mut known = Vec::new();
    let mut unknown = Vec::new();

    for logical_name in requested {
        if available_set.contains(logical_name.as_str()) {
            known.push(logical_name.clone());
        } else {
            unknown.push(logical_name.clone());
        }
    }

    (known, unknown)
}

pub(super) fn build_unknown_selection_issues(
    scope: PublishCheckScopeDto,
    logical_names: &[String],
) -> Vec<PublishCheckIssueResponse> {
    logical_names
        .iter()
        .map(|logical_name| PublishCheckIssueResponse {
            scope,
            scope_logical_name: logical_name.clone(),
            category: PublishCheckCategoryDto::Unknown,
            severity: PublishCheckSeverityDto::Error,
            message: match scope {
                PublishCheckScopeDto::Entity => {
                    format!("selected entity '{}' does not exist", logical_name)
                }
                PublishCheckScopeDto::App => {
                    format!("selected app '{}' does not exist", logical_name)
                }
            },
            fix_path: Some(match scope {
                PublishCheckScopeDto::Entity => "/maker/entities".to_owned(),
                PublishCheckScopeDto::App => "/maker/apps".to_owned(),
            }),
            dependency_path: None,
        })
        .collect()
}

pub(super) fn extract_dependency_path(message: &str) -> Option<String> {
    let Some(entity_fragment) = message.strip_prefix("dependency check failed: entity '") else {
        let edge_fragment = message.strip_prefix("dependency check failed: app '")?;
        let (app_logical_name, rest) = edge_fragment.split_once("' -> entity '")?;
        let (entity_logical_name, _) = rest.split_once('"').or_else(|| rest.split_once('\''))?;

        return Some(format!("{app_logical_name} -> {entity_logical_name}"));
    };

    let (entity_logical_name, rest) = entity_fragment.split_once("' relation field '")?;
    let (field_logical_name, rest) = rest.split_once("' -> entity '")?;
    let (target_entity_logical_name, _) = rest
        .split_once('"')
        .or_else(|| rest.split_once('\''))
        .or_else(|| rest.split_once(' '))?;

    Some(format!(
        "{entity_logical_name}.{field_logical_name} -> {target_entity_logical_name}"
    ))
}

fn classify_issue(scope: PublishCheckScopeDto, message: &str) -> PublishCheckCategoryDto {
    let normalized = message.to_ascii_lowercase();

    if normalized.contains("relation") || normalized.contains("target entity") {
        return PublishCheckCategoryDto::Relationship;
    }
    if normalized.contains("form ") || normalized.contains("header field") {
        return PublishCheckCategoryDto::Form;
    }
    if normalized.contains("view ")
        || normalized.contains("default sort")
        || normalized.contains("filter field")
    {
        return PublishCheckCategoryDto::View;
    }
    if normalized.contains("sitemap") {
        return PublishCheckCategoryDto::Sitemap;
    }
    if normalized.contains("binding") || normalized.contains("bound") {
        return PublishCheckCategoryDto::Binding;
    }
    if normalized.contains("schema") || normalized.contains("field") {
        return PublishCheckCategoryDto::Schema;
    }

    match scope {
        PublishCheckScopeDto::Entity => PublishCheckCategoryDto::Schema,
        PublishCheckScopeDto::App => PublishCheckCategoryDto::Unknown,
    }
}

fn entity_fix_path(entity_logical_name: &str, category: PublishCheckCategoryDto) -> String {
    match category {
        PublishCheckCategoryDto::Form => format!("/maker/entities/{entity_logical_name}/forms"),
        PublishCheckCategoryDto::View => format!("/maker/entities/{entity_logical_name}/views"),
        _ => format!("/maker/entities/{entity_logical_name}"),
    }
}

fn app_fix_path(app_logical_name: &str, category: PublishCheckCategoryDto) -> String {
    match category {
        PublishCheckCategoryDto::Sitemap => format!("/maker/apps/{app_logical_name}/sitemap"),
        _ => "/maker/apps".to_owned(),
    }
}
