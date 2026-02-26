use std::str::FromStr;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;

use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{ViewColumn, ViewFilterGroup, ViewSort, ViewType};

use crate::dto::{CreateViewRequest, ViewResponse};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_views_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<ViewResponse>>> {
    let views = state
        .metadata_service
        .list_views(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(ViewResponse::from)
        .collect();
    Ok(Json(views))
}

pub async fn save_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateViewRequest>,
) -> ApiResult<(StatusCode, Json<ViewResponse>)> {
    let view_type = ViewType::from_str(payload.view_type.as_str())?;
    let columns = payload
        .columns
        .into_iter()
        .map(serde_json::from_value::<ViewColumn>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid view column payload: {error}")))?;
    let default_sort = payload
        .default_sort
        .map(serde_json::from_value::<ViewSort>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view default_sort payload: {error}"))
        })?;
    let filter_criteria = payload
        .filter_criteria
        .map(serde_json::from_value::<ViewFilterGroup>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view filter_criteria payload: {error}"))
        })?;
    let view = state
        .metadata_service
        .save_view(
            &user,
            qryvanta_application::SaveViewInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                view_type,
                columns,
                default_sort,
                filter_criteria,
                is_default: payload.is_default,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(ViewResponse::from(view))))
}

pub async fn get_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, view_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<ViewResponse>> {
    let view = state
        .metadata_service
        .find_view(
            &user,
            entity_logical_name.as_str(),
            view_logical_name.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "view '{}.{}' does not exist",
                entity_logical_name, view_logical_name
            ))
        })?;
    Ok(Json(ViewResponse::from(view)))
}

pub async fn update_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, view_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateViewRequest>,
) -> ApiResult<Json<ViewResponse>> {
    if payload.logical_name != view_logical_name {
        return Err(AppError::Validation(format!(
            "view logical name in path '{}' must match payload '{}'",
            view_logical_name, payload.logical_name
        ))
        .into());
    }

    let view_type = ViewType::from_str(payload.view_type.as_str())?;
    let columns = payload
        .columns
        .into_iter()
        .map(serde_json::from_value::<ViewColumn>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid view column payload: {error}")))?;
    let default_sort = payload
        .default_sort
        .map(serde_json::from_value::<ViewSort>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view default_sort payload: {error}"))
        })?;
    let filter_criteria = payload
        .filter_criteria
        .map(serde_json::from_value::<ViewFilterGroup>)
        .transpose()
        .map_err(|error| {
            AppError::Validation(format!("invalid view filter_criteria payload: {error}"))
        })?;
    let view = state
        .metadata_service
        .save_view(
            &user,
            qryvanta_application::SaveViewInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                view_type,
                columns,
                default_sort,
                filter_criteria,
                is_default: payload.is_default,
            },
        )
        .await?;
    Ok(Json(ViewResponse::from(view)))
}

pub async fn delete_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, view_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_view(
            &user,
            entity_logical_name.as_str(),
            view_logical_name.as_str(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
