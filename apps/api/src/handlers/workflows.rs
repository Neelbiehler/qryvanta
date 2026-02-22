use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use qryvanta_core::UserIdentity;

use crate::dto::{
    ExecuteWorkflowRequest, SaveWorkflowRequest, WorkflowResponse, WorkflowRunAttemptResponse,
    WorkflowRunResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct WorkflowRunListQueryRequest {
    pub workflow_logical_name: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_workflows_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<WorkflowResponse>>> {
    let workflows = state
        .workflow_service
        .list_workflows(&user)
        .await?
        .into_iter()
        .map(WorkflowResponse::from)
        .collect();

    Ok(Json(workflows))
}

pub async fn save_workflow_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<SaveWorkflowRequest>,
) -> ApiResult<(StatusCode, Json<WorkflowResponse>)> {
    let workflow = state
        .workflow_service
        .save_workflow(&user, payload.try_into()?)
        .await?;

    Ok((StatusCode::CREATED, Json(WorkflowResponse::from(workflow))))
}

pub async fn execute_workflow_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(workflow_logical_name): Path<String>,
    Json(payload): Json<ExecuteWorkflowRequest>,
) -> ApiResult<Json<WorkflowRunResponse>> {
    let run = state
        .workflow_service
        .execute_workflow(
            &user,
            workflow_logical_name.as_str(),
            payload.trigger_payload,
        )
        .await?;

    Ok(Json(WorkflowRunResponse::from(run)))
}

pub async fn list_workflow_runs_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<WorkflowRunListQueryRequest>,
) -> ApiResult<Json<Vec<WorkflowRunResponse>>> {
    let runs = state
        .workflow_service
        .list_runs(
            &user,
            qryvanta_application::WorkflowRunListQuery {
                workflow_logical_name: query.workflow_logical_name,
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
            },
        )
        .await?
        .into_iter()
        .map(WorkflowRunResponse::from)
        .collect();

    Ok(Json(runs))
}

pub async fn list_workflow_run_attempts_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(run_id): Path<String>,
) -> ApiResult<Json<Vec<WorkflowRunAttemptResponse>>> {
    let attempts = state
        .workflow_service
        .list_run_attempts(&user, run_id.as_str())
        .await?
        .into_iter()
        .map(WorkflowRunAttemptResponse::from)
        .collect();

    Ok(Json(attempts))
}
