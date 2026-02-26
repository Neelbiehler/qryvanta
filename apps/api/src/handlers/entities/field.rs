use std::str::FromStr;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;

use qryvanta_core::UserIdentity;
use qryvanta_domain::FieldType;

use crate::dto::{CreateFieldRequest, FieldResponse, UpdateFieldRequest};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_fields_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<FieldResponse>>> {
    let fields = state
        .metadata_service
        .list_fields(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .map(FieldResponse::from)
        .collect();

    Ok(Json(fields))
}

pub async fn save_field_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateFieldRequest>,
) -> ApiResult<(StatusCode, Json<FieldResponse>)> {
    let field_type = FieldType::from_str(payload.field_type.as_str())?;
    let field = state
        .metadata_service
        .save_field(
            &user,
            qryvanta_application::SaveFieldInput {
                entity_logical_name,
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                field_type,
                is_required: payload.is_required,
                is_unique: payload.is_unique,
                default_value: payload.default_value,
                calculation_expression: payload.calculation_expression,
                relation_target_entity: payload.relation_target_entity,
                option_set_logical_name: payload.option_set_logical_name,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(FieldResponse::from(field))))
}

pub async fn update_field_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, field_logical_name)): Path<(String, String)>,
    Json(payload): Json<UpdateFieldRequest>,
) -> ApiResult<Json<FieldResponse>> {
    let field = state
        .metadata_service
        .update_field(
            &user,
            qryvanta_application::UpdateFieldInput {
                entity_logical_name,
                logical_name: field_logical_name,
                display_name: payload.display_name,
                description: payload.description,
                default_value: payload.default_value,
                calculation_expression: payload.calculation_expression,
                max_length: payload.max_length,
                min_value: payload.min_value,
                max_value: payload.max_value,
            },
        )
        .await?;

    Ok(Json(FieldResponse::from(field)))
}

pub async fn delete_field_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, field_logical_name)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_field(
            &user,
            entity_logical_name.as_str(),
            field_logical_name.as_str(),
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
