use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use qryvanta_core::UserIdentity;
use tracing::warn;

use crate::dto::{
    CreateRuntimeRecordRequest, QueryRuntimeRecordsRequest, RuntimeRecordResponse,
    UpdateRuntimeRecordRequest,
};
use crate::error::ApiResult;
use crate::handlers::runtime::runtime_record_query_from_request;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct RuntimeRecordListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
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
                owner_subject: None,
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

    if let Err(error) = state
        .workflow_service
        .drain_runtime_record_workflow_events_inline(
            &user,
            state.workflow_worker_max_claim_limit,
            state.workflow_worker_default_lease_seconds,
        )
        .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            app_logical_name = %app_logical_name,
            entity_logical_name = %entity_logical_name,
            record_id = %record.record_id().as_str(),
            "runtime workflow event drain failed after workspace record creation"
        );
    }

    Ok((StatusCode::CREATED, {
        let response = RuntimeRecordResponse::from(record);
        if let Err(error) = crate::qrywell_sync::enqueue_runtime_record_upsert(
            &state.postgres_pool,
            user.tenant_id(),
            entity_logical_name.as_str(),
            &response,
            state.qrywell_sync_max_attempts,
        )
        .await
        {
            warn!(
                error = %error,
                tenant_id = %user.tenant_id(),
                app_logical_name = %app_logical_name,
                entity_logical_name = %entity_logical_name,
                record_id = %response.record_id,
                "qrywell sync enqueue failed after workspace record creation"
            );
        }

        Json(response)
    }))
}

pub async fn workspace_query_records_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
    Json(payload): Json<QueryRuntimeRecordsRequest>,
) -> ApiResult<Json<Vec<RuntimeRecordResponse>>> {
    let _query_permit = state.try_acquire_runtime_query_permit()?;
    let query = runtime_record_query_from_request(
        &state.metadata_service,
        &user,
        entity_logical_name.as_str(),
        payload,
        state.runtime_query_max_limit,
    )
    .await?;

    let records = state
        .app_service
        .query_records(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            query,
        )
        .await?
        .into_iter()
        .map(RuntimeRecordResponse::from)
        .collect();

    Ok(Json(records))
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

    let response = RuntimeRecordResponse::from(record);
    if let Err(error) = crate::qrywell_sync::enqueue_runtime_record_upsert(
        &state.postgres_pool,
        user.tenant_id(),
        entity_logical_name.as_str(),
        &response,
        state.qrywell_sync_max_attempts,
    )
    .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            app_logical_name = %app_logical_name,
            entity_logical_name = %entity_logical_name,
            record_id = %response.record_id,
            "qrywell sync enqueue failed after workspace record update"
        );
    }

    Ok(Json(response))
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

    if let Err(error) = state
        .workflow_service
        .drain_runtime_record_workflow_events_inline(
            &user,
            state.workflow_worker_max_claim_limit,
            state.workflow_worker_default_lease_seconds,
        )
        .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            app_logical_name = %app_logical_name,
            entity_logical_name = %entity_logical_name,
            record_id = %record.record_id().as_str(),
            "runtime workflow event drain failed after workspace record update"
        );
    }

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

    if let Err(error) = state
        .workflow_service
        .drain_runtime_record_workflow_events_inline(
            &user,
            state.workflow_worker_max_claim_limit,
            state.workflow_worker_default_lease_seconds,
        )
        .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            app_logical_name = %app_logical_name,
            entity_logical_name = %entity_logical_name,
            record_id = %record_id,
            "runtime workflow event drain failed after workspace record deletion"
        );
    }

    if let Err(error) = crate::qrywell_sync::enqueue_runtime_record_delete(
        &state.postgres_pool,
        user.tenant_id(),
        entity_logical_name.as_str(),
        record_id.as_str(),
        state.qrywell_sync_max_attempts,
    )
    .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            app_logical_name = %app_logical_name,
            entity_logical_name = %entity_logical_name,
            record_id = %record_id,
            "qrywell sync enqueue failed after workspace record deletion"
        );
    }

    Ok(StatusCode::NO_CONTENT)
}
