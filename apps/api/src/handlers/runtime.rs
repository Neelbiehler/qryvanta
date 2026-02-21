use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use qryvanta_core::UserIdentity;

use crate::dto::{CreateRuntimeRecordRequest, RuntimeRecordResponse, UpdateRuntimeRecordRequest};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct RuntimeRecordListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_runtime_records_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Query(query): Query<RuntimeRecordListQuery>,
) -> ApiResult<Json<Vec<RuntimeRecordResponse>>> {
    let records = state
        .metadata_service
        .list_runtime_records(
            &user,
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

pub async fn create_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateRuntimeRecordRequest>,
) -> ApiResult<(StatusCode, Json<RuntimeRecordResponse>)> {
    let record = state
        .metadata_service
        .create_runtime_record(&user, entity_logical_name.as_str(), payload.data)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(RuntimeRecordResponse::from(record)),
    ))
}

pub async fn update_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, record_id)): Path<(String, String)>,
    Json(payload): Json<UpdateRuntimeRecordRequest>,
) -> ApiResult<Json<RuntimeRecordResponse>> {
    let record = state
        .metadata_service
        .update_runtime_record(
            &user,
            entity_logical_name.as_str(),
            record_id.as_str(),
            payload.data,
        )
        .await?;

    Ok(Json(RuntimeRecordResponse::from(record)))
}

pub async fn get_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, record_id)): Path<(String, String)>,
) -> ApiResult<Json<RuntimeRecordResponse>> {
    let record = state
        .metadata_service
        .get_runtime_record(&user, entity_logical_name.as_str(), record_id.as_str())
        .await?;

    Ok(Json(RuntimeRecordResponse::from(record)))
}

pub async fn delete_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, record_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_runtime_record(&user, entity_logical_name.as_str(), record_id.as_str())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
