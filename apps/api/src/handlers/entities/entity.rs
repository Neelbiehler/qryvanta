use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;

use qryvanta_core::UserIdentity;

use crate::dto::{CreateEntityRequest, EntityResponse, UpdateEntityRequest};
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
        .register_entity_with_details(
            &user,
            payload.logical_name,
            payload.display_name,
            payload.description,
            payload.plural_display_name,
            payload.icon,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(EntityResponse::from(entity))))
}

pub async fn update_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<UpdateEntityRequest>,
) -> ApiResult<Json<EntityResponse>> {
    let entity = state
        .metadata_service
        .update_entity(
            &user,
            qryvanta_application::UpdateEntityInput {
                logical_name: entity_logical_name,
                display_name: payload.display_name,
                description: payload.description,
                plural_display_name: payload.plural_display_name,
                icon: payload.icon,
            },
        )
        .await?;

    Ok(Json(EntityResponse::from(entity)))
}
