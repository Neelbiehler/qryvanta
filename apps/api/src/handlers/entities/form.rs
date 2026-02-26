use std::str::FromStr;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;

use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{FormTab, FormType};

use crate::dto::{CreateFormRequest, FormResponse};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_forms_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<FormResponse>>> {
    let forms = state
        .metadata_service
        .list_forms(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(FormResponse::from)
        .collect();
    Ok(Json(forms))
}

pub async fn save_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateFormRequest>,
) -> ApiResult<(StatusCode, Json<FormResponse>)> {
    let form_type = FormType::from_str(payload.form_type.as_str())?;
    let tabs = payload
        .tabs
        .into_iter()
        .map(serde_json::from_value::<FormTab>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid form tab payload: {error}")))?;
    let form = state
        .metadata_service
        .save_form(
            &user,
            qryvanta_application::SaveFormInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                form_type,
                tabs,
                header_fields: payload.header_fields,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(FormResponse::from(form))))
}

pub async fn get_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, form_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<FormResponse>> {
    let form = state
        .metadata_service
        .find_form(
            &user,
            entity_logical_name.as_str(),
            form_logical_name.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "form '{}.{}' does not exist",
                entity_logical_name, form_logical_name
            ))
        })?;
    Ok(Json(FormResponse::from(form)))
}

pub async fn update_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, form_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateFormRequest>,
) -> ApiResult<Json<FormResponse>> {
    if payload.logical_name != form_logical_name {
        return Err(AppError::Validation(format!(
            "form logical name in path '{}' must match payload '{}'",
            form_logical_name, payload.logical_name
        ))
        .into());
    }

    let form_type = FormType::from_str(payload.form_type.as_str())?;
    let tabs = payload
        .tabs
        .into_iter()
        .map(serde_json::from_value::<FormTab>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::Validation(format!("invalid form tab payload: {error}")))?;
    let form = state
        .metadata_service
        .save_form(
            &user,
            qryvanta_application::SaveFormInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                form_type,
                tabs,
                header_fields: payload.header_fields,
            },
        )
        .await?;
    Ok(Json(FormResponse::from(form)))
}

pub async fn delete_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, form_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_form(
            &user,
            entity_logical_name.as_str(),
            form_logical_name.as_str(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
