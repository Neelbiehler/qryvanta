use super::*;

#[derive(Debug, Deserialize)]
pub struct WorkflowQueueStatsQuery {
    pub active_window_seconds: Option<u32>,
    pub partition_count: Option<u32>,
    pub partition_index: Option<u32>,
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

pub async fn workflow_queue_stats_handler(
    State(state): State<AppState>,
    Extension(_worker): Extension<WorkerIdentity>,
    Query(query): Query<WorkflowQueueStatsQuery>,
) -> ApiResult<Json<WorkflowQueueStatsResponse>> {
    let active_window_seconds = query.active_window_seconds.unwrap_or(120).max(1);
    let partition = parse_worker_partition(
        query.partition_count,
        query.partition_index,
        state.workflow_worker_max_partition_count,
    )?;

    let stats = state
        .workflow_service
        .queue_stats_with_partition(active_window_seconds, partition)
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
