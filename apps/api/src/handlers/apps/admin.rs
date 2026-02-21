use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use qryvanta_core::UserIdentity;

use crate::dto::{
    AppEntityBindingResponse, AppResponse, AppRoleEntityPermissionResponse, BindAppEntityRequest,
    CreateAppRequest, SaveAppRoleEntityPermissionRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_apps_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<AppResponse>>> {
    let apps = state
        .app_service
        .list_apps(&user)
        .await?
        .into_iter()
        .map(AppResponse::from)
        .collect();

    Ok(Json(apps))
}

pub async fn create_app_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateAppRequest>,
) -> ApiResult<(StatusCode, Json<AppResponse>)> {
    let app = state
        .app_service
        .create_app(
            &user,
            qryvanta_application::CreateAppInput {
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                description: payload.description,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(AppResponse::from(app))))
}

pub async fn list_app_entities_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
) -> ApiResult<Json<Vec<AppEntityBindingResponse>>> {
    let entities = state
        .app_service
        .list_app_entities(&user, app_logical_name.as_str())
        .await?
        .into_iter()
        .map(AppEntityBindingResponse::from)
        .collect();

    Ok(Json(entities))
}

pub async fn bind_app_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
    Json(payload): Json<BindAppEntityRequest>,
) -> ApiResult<(StatusCode, Json<AppEntityBindingResponse>)> {
    let binding = state
        .app_service
        .bind_entity(
            &user,
            qryvanta_application::BindAppEntityInput {
                app_logical_name,
                entity_logical_name: payload.entity_logical_name,
                navigation_label: payload.navigation_label,
                navigation_order: payload.navigation_order,
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(AppEntityBindingResponse::from(binding)),
    ))
}

pub async fn list_app_role_permissions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
) -> ApiResult<Json<Vec<AppRoleEntityPermissionResponse>>> {
    let permissions = state
        .app_service
        .list_role_entity_permissions(&user, app_logical_name.as_str())
        .await?
        .into_iter()
        .map(AppRoleEntityPermissionResponse::from)
        .collect();

    Ok(Json(permissions))
}

pub async fn save_app_role_permission_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
    Json(payload): Json<SaveAppRoleEntityPermissionRequest>,
) -> ApiResult<Json<AppRoleEntityPermissionResponse>> {
    let permission = state
        .app_service
        .save_role_entity_permission(
            &user,
            qryvanta_application::SaveAppRoleEntityPermissionInput {
                app_logical_name,
                role_name: payload.role_name,
                entity_logical_name: payload.entity_logical_name,
                can_read: payload.can_read,
                can_create: payload.can_create,
                can_update: payload.can_update,
                can_delete: payload.can_delete,
            },
        )
        .await?;

    Ok(Json(AppRoleEntityPermissionResponse::from(permission)))
}
