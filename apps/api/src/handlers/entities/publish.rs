use axum::Json;
use axum::extract::{Extension, Path, State};

use qryvanta_core::{AppError, UserIdentity};

use crate::dto::{PublishChecksResponse, PublishedSchemaResponse};
use crate::error::ApiResult;
use crate::state::AppState;

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

pub async fn publish_checks_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<PublishChecksResponse>> {
    let errors = state
        .metadata_service
        .publish_checks(&user, entity_logical_name.as_str())
        .await?;

    Ok(Json(PublishChecksResponse {
        is_publishable: errors.is_empty(),
        errors,
    }))
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
