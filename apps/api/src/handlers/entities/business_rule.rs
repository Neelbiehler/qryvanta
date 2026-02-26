use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;

use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{BusinessRuleAction, BusinessRuleCondition, BusinessRuleScope};

use crate::dto::{BusinessRuleResponse, CreateBusinessRuleRequest};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_business_rules_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<BusinessRuleResponse>>> {
    let rules = state
        .metadata_service
        .list_business_rules(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(BusinessRuleResponse::from)
        .collect();
    Ok(Json(rules))
}

pub async fn save_business_rule_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateBusinessRuleRequest>,
) -> ApiResult<(StatusCode, Json<BusinessRuleResponse>)> {
    let scope = parse_business_rule_scope(payload.scope.as_str())?;
    let conditions = payload
        .conditions
        .into_iter()
        .map(serde_json::from_value::<BusinessRuleCondition>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            AppError::Validation(format!("invalid business rule condition payload: {error}"))
        })?;
    let actions = payload
        .actions
        .into_iter()
        .map(serde_json::from_value::<BusinessRuleAction>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            AppError::Validation(format!("invalid business rule action payload: {error}"))
        })?;

    let rule = state
        .metadata_service
        .save_business_rule(
            &user,
            qryvanta_application::SaveBusinessRuleInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                scope,
                form_logical_name: payload.form_logical_name,
                conditions,
                actions,
                is_active: payload.is_active,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(BusinessRuleResponse::from(rule))))
}

pub async fn get_business_rule_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, business_rule_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<BusinessRuleResponse>> {
    let rule = state
        .metadata_service
        .find_business_rule(
            &user,
            entity_logical_name.as_str(),
            business_rule_logical_name.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "business rule '{}.{}' does not exist",
                entity_logical_name, business_rule_logical_name
            ))
        })?;
    Ok(Json(BusinessRuleResponse::from(rule)))
}

pub async fn update_business_rule_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, business_rule_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateBusinessRuleRequest>,
) -> ApiResult<Json<BusinessRuleResponse>> {
    if payload.logical_name != business_rule_logical_name {
        return Err(AppError::Validation(format!(
            "business rule logical name in path '{}' must match payload '{}'",
            business_rule_logical_name, payload.logical_name
        ))
        .into());
    }

    let scope = parse_business_rule_scope(payload.scope.as_str())?;
    let conditions = payload
        .conditions
        .into_iter()
        .map(serde_json::from_value::<BusinessRuleCondition>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            AppError::Validation(format!("invalid business rule condition payload: {error}"))
        })?;
    let actions = payload
        .actions
        .into_iter()
        .map(serde_json::from_value::<BusinessRuleAction>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            AppError::Validation(format!("invalid business rule action payload: {error}"))
        })?;

    let rule = state
        .metadata_service
        .save_business_rule(
            &user,
            qryvanta_application::SaveBusinessRuleInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                scope,
                form_logical_name: payload.form_logical_name,
                conditions,
                actions,
                is_active: payload.is_active,
            },
        )
        .await?;

    Ok(Json(BusinessRuleResponse::from(rule)))
}

pub async fn delete_business_rule_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, business_rule_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_business_rule(
            &user,
            entity_logical_name.as_str(),
            business_rule_logical_name.as_str(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_business_rule_scope(value: &str) -> Result<BusinessRuleScope, AppError> {
    match value {
        "entity" => Ok(BusinessRuleScope::Entity),
        "form" => Ok(BusinessRuleScope::Form),
        _ => Err(AppError::Validation(format!(
            "unknown business rule scope '{value}'"
        ))),
    }
}
