use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use qryvanta_core::UserIdentity;

use crate::dto::{CreateEntityRequest, EntityResponse};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_entities_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<EntityResponse>>> {
    let entities = state
        .metadata_service
        .list_entities(user.tenant_id())
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
        .register_entity(user.tenant_id(), payload.logical_name, payload.display_name)
        .await?;

    Ok((StatusCode::CREATED, Json(EntityResponse::from(entity))))
}
