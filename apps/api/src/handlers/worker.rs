use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;

use qryvanta_application::WorkflowWorkerHeartbeatInput;
use qryvanta_domain::{WorkflowAction, WorkflowStep, WorkflowTrigger};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ApiResult;
use crate::middleware::WorkerIdentity;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ClaimWorkflowJobsRequest {
    pub limit: Option<usize>,
    pub lease_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct WorkerHeartbeatRequest {
    pub claimed_jobs: Option<u32>,
    pub executed_jobs: Option<u32>,
    pub failed_jobs: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowQueueStatsQuery {
    pub active_window_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ClaimedWorkflowJobsResponse {
    pub jobs: Vec<ClaimedWorkflowJobResponse>,
}

#[derive(Debug, Serialize)]
pub struct ClaimedWorkflowJobResponse {
    pub job_id: String,
    pub tenant_id: String,
    pub run_id: String,
    pub workflow_logical_name: String,
    pub workflow_display_name: String,
    pub workflow_description: Option<String>,
    pub workflow_trigger: WorkflowTrigger,
    pub workflow_action: WorkflowAction,
    pub workflow_steps: Option<Vec<WorkflowStep>>,
    pub workflow_max_attempts: u16,
    pub workflow_is_enabled: bool,
    pub trigger_payload: Value,
}

#[derive(Debug, Serialize)]
pub struct WorkflowQueueStatsResponse {
    pub pending_jobs: i64,
    pub leased_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
    pub expired_leases: i64,
    pub active_workers: i64,
}

pub async fn claim_workflow_jobs_handler(
    State(state): State<AppState>,
    Extension(worker): Extension<WorkerIdentity>,
    Json(payload): Json<ClaimWorkflowJobsRequest>,
) -> ApiResult<Json<ClaimedWorkflowJobsResponse>> {
    let requested_limit = payload
        .limit
        .unwrap_or(state.workflow_worker_max_claim_limit);
    let requested_lease_seconds = payload
        .lease_seconds
        .unwrap_or(state.workflow_worker_default_lease_seconds);

    let effective_limit = requested_limit.clamp(1, state.workflow_worker_max_claim_limit);
    let effective_lease_seconds = requested_lease_seconds.max(1);

    let jobs = state
        .workflow_service
        .claim_jobs_for_worker(worker.worker_id(), effective_limit, effective_lease_seconds)
        .await?
        .into_iter()
        .map(|job| ClaimedWorkflowJobResponse {
            job_id: job.job_id,
            tenant_id: job.tenant_id.to_string(),
            run_id: job.run_id,
            workflow_logical_name: job.workflow.logical_name().as_str().to_owned(),
            workflow_display_name: job.workflow.display_name().as_str().to_owned(),
            workflow_description: job.workflow.description().map(ToOwned::to_owned),
            workflow_trigger: job.workflow.trigger().clone(),
            workflow_action: job.workflow.action().clone(),
            workflow_steps: job.workflow.steps().map(ToOwned::to_owned),
            workflow_max_attempts: job.workflow.max_attempts(),
            workflow_is_enabled: job.workflow.is_enabled(),
            trigger_payload: job.trigger_payload,
        })
        .collect();

    Ok(Json(ClaimedWorkflowJobsResponse { jobs }))
}

pub async fn worker_heartbeat_handler(
    State(state): State<AppState>,
    Extension(worker): Extension<WorkerIdentity>,
    Json(payload): Json<WorkerHeartbeatRequest>,
) -> ApiResult<StatusCode> {
    state
        .workflow_service
        .heartbeat_worker(
            worker.worker_id(),
            WorkflowWorkerHeartbeatInput {
                claimed_jobs: payload.claimed_jobs.unwrap_or(0),
                executed_jobs: payload.executed_jobs.unwrap_or(0),
                failed_jobs: payload.failed_jobs.unwrap_or(0),
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn workflow_queue_stats_handler(
    State(state): State<AppState>,
    Extension(_worker): Extension<WorkerIdentity>,
    Query(query): Query<WorkflowQueueStatsQuery>,
) -> ApiResult<Json<WorkflowQueueStatsResponse>> {
    let active_window_seconds = query.active_window_seconds.unwrap_or(120).max(1);
    let stats = state
        .workflow_service
        .queue_stats(active_window_seconds)
        .await?;

    Ok(Json(WorkflowQueueStatsResponse {
        pending_jobs: stats.pending_jobs,
        leased_jobs: stats.leased_jobs,
        completed_jobs: stats.completed_jobs,
        failed_jobs: stats.failed_jobs,
        expired_leases: stats.expired_leases,
        active_workers: stats.active_workers,
    }))
}
