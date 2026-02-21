use std::str::FromStr;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::FieldType;

use crate::dto::{
    CreateEntityRequest, CreateFieldRequest, EntityResponse, FieldResponse, PublishedSchemaResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_entities_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<EntityResponse>>> {
    let entities = state
        .metadata_service
        .list_entities(&user)
        .await?
        .into_iter()
        .map(EntityResponse::from)
        .collect();

    Ok(Json(entities))
}

pub async fn create_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateEntityRequest>,
) -> ApiResult<(StatusCode, Json<EntityResponse>)> {
    let entity = state
        .metadata_service
        .register_entity(&user, payload.logical_name, payload.display_name)
        .await?;

    Ok((StatusCode::CREATED, Json(EntityResponse::from(entity))))
}

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
                relation_target_entity: payload.relation_target_entity,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(FieldResponse::from(field))))
}

pub async fn publish_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<PublishedSchemaResponse>> {
    let published_schema = state
        .metadata_service
        .publish_entity(&user, entity_logical_name.as_str())
        .await?;

    Ok(Json(PublishedSchemaResponse::from(published_schema)))
}

pub async fn latest_published_schema_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<PublishedSchemaResponse>> {
    let published_schema = state
        .metadata_service
        .latest_published_schema(&user, entity_logical_name.as_str())
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "entity '{}' does not have a published schema",
                entity_logical_name
            ))
        })?;

    Ok(Json(PublishedSchemaResponse::from(published_schema)))
}
