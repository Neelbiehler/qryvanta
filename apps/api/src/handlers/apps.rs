use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;

use qryvanta_core::UserIdentity;

use crate::dto::{
    AppEntityBindingResponse, AppEntityCapabilitiesResponse, AppResponse,
    AppRoleEntityPermissionResponse, BindAppEntityRequest, CreateAppRequest,
    CreateRuntimeRecordRequest, PublishedSchemaResponse, RuntimeRecordResponse,
    SaveAppRoleEntityPermissionRequest, UpdateRuntimeRecordRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct RuntimeRecordListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

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

pub async fn list_workspace_apps_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<AppResponse>>> {
    let apps = state
        .app_service
        .list_accessible_apps(&user)
        .await?
        .into_iter()
        .map(AppResponse::from)
        .collect();

    Ok(Json(apps))
}

pub async fn app_navigation_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
) -> ApiResult<Json<Vec<AppEntityBindingResponse>>> {
    let entities = state
        .app_service
        .app_navigation_for_subject(&user, app_logical_name.as_str())
        .await?
        .into_iter()
        .map(AppEntityBindingResponse::from)
        .collect();

    Ok(Json(entities))
}

pub async fn workspace_entity_schema_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<PublishedSchemaResponse>> {
    let schema = state
        .app_service
        .schema_for_subject(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
        )
        .await?;

    Ok(Json(PublishedSchemaResponse::from(schema)))
}

pub async fn workspace_entity_capabilities_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<AppEntityCapabilitiesResponse>> {
    let capabilities = state
        .app_service
        .entity_capabilities_for_subject(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
        )
        .await?;

    Ok(Json(AppEntityCapabilitiesResponse::from(capabilities)))
}

pub async fn workspace_list_records_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
    Query(query): Query<RuntimeRecordListQuery>,
) -> ApiResult<Json<Vec<RuntimeRecordResponse>>> {
    let records = state
        .app_service
        .list_records(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            qryvanta_application::RecordListQuery {
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
            },
        )
        .await?
        .into_iter()
        .map(RuntimeRecordResponse::from)
        .collect();

    Ok(Json(records))
}

pub async fn workspace_create_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
    Json(payload): Json<CreateRuntimeRecordRequest>,
) -> ApiResult<(StatusCode, Json<RuntimeRecordResponse>)> {
    let record = state
        .app_service
        .create_record(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            payload.data,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(RuntimeRecordResponse::from(record)),
    ))
}

pub async fn workspace_get_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name, record_id)): Path<(String, String, String)>,
) -> ApiResult<Json<RuntimeRecordResponse>> {
    let record = state
        .app_service
        .get_record(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            record_id.as_str(),
        )
        .await?;

    Ok(Json(RuntimeRecordResponse::from(record)))
}

pub async fn workspace_update_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name, record_id)): Path<(String, String, String)>,
    Json(payload): Json<UpdateRuntimeRecordRequest>,
) -> ApiResult<Json<RuntimeRecordResponse>> {
    let record = state
        .app_service
        .update_record(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            record_id.as_str(),
            payload.data,
        )
        .await?;

    Ok(Json(RuntimeRecordResponse::from(record)))
}

pub async fn workspace_delete_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name, record_id)): Path<(String, String, String)>,
) -> ApiResult<StatusCode> {
    state
        .app_service
        .delete_record(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            record_id.as_str(),
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
