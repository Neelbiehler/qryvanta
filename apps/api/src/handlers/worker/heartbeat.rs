use super::*;

#[derive(Debug, Deserialize)]
pub struct WorkerHeartbeatRequest {
    pub claimed_jobs: Option<u32>,
    pub executed_jobs: Option<u32>,
    pub failed_jobs: Option<u32>,
    pub partition_count: Option<u32>,
    pub partition_index: Option<u32>,
}

pub async fn worker_heartbeat_handler(
    State(state): State<AppState>,
    Extension(worker): Extension<WorkerIdentity>,
    Json(payload): Json<WorkerHeartbeatRequest>,
) -> ApiResult<StatusCode> {
    let partition = parse_worker_partition(
        payload.partition_count,
        payload.partition_index,
        state.workflow_worker_max_partition_count,
    )?;

    state
        .workflow_service
        .heartbeat_worker(
            worker.worker_id(),
            WorkflowWorkerHeartbeatInput {
                claimed_jobs: payload.claimed_jobs.unwrap_or(0),
                executed_jobs: payload.executed_jobs.unwrap_or(0),
                failed_jobs: payload.failed_jobs.unwrap_or(0),
                partition,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
