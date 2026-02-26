use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;

use qryvanta_core::{AppError, UserIdentity};

use crate::dto::{CreateOptionSetRequest, OptionSetResponse};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_option_sets_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<OptionSetResponse>>> {
    let option_sets = state
        .metadata_service
        .list_option_sets(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(OptionSetResponse::from)
        .collect();

    Ok(Json(option_sets))
}

pub async fn save_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateOptionSetRequest>,
) -> ApiResult<(StatusCode, Json<OptionSetResponse>)> {
    let options = payload
        .options
        .into_iter()
        .map(qryvanta_domain::OptionSetItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let option_set = state
        .metadata_service
        .save_option_set(
            &user,
            qryvanta_application::SaveOptionSetInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                options,
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(OptionSetResponse::from(option_set)),
    ))
}

pub async fn update_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, option_set_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateOptionSetRequest>,
) -> ApiResult<Json<OptionSetResponse>> {
    if payload.logical_name != option_set_logical_name {
        return Err(AppError::Validation(format!(
            "option set logical name in path '{}' must match payload '{}'",
            option_set_logical_name, payload.logical_name
        ))
        .into());
    }

    let options = payload
        .options
        .into_iter()
        .map(qryvanta_domain::OptionSetItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let option_set = state
        .metadata_service
        .save_option_set(
            &user,
            qryvanta_application::SaveOptionSetInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                options,
            },
        )
        .await?;

    Ok(Json(OptionSetResponse::from(option_set)))
}

pub async fn get_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, option_set_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<OptionSetResponse>> {
    let option_set = state
        .metadata_service
        .find_option_set(
            &user,
            entity_logical_name.as_str(),
            option_set_logical_name.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "option set '{}.{}' does not exist",
                entity_logical_name, option_set_logical_name
            ))
        })?;
    Ok(Json(OptionSetResponse::from(option_set)))
}

pub async fn delete_option_set_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, option_set_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_option_set(
            &user,
            entity_logical_name.as_str(),
            option_set_logical_name.as_str(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
