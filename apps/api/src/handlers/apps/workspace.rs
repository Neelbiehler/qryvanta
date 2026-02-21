use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use qryvanta_core::UserIdentity;

use crate::dto::{
    AppEntityBindingResponse, AppEntityCapabilitiesResponse, AppResponse,
    CreateRuntimeRecordRequest, PublishedSchemaResponse, RuntimeRecordResponse,
    UpdateRuntimeRecordRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct RuntimeRecordListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
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
