use super::*;

#[derive(Debug, Deserialize)]
pub struct ClaimWorkflowJobsRequest {
    pub limit: Option<usize>,
    pub lease_seconds: Option<u32>,
    pub partition_count: Option<u32>,
    pub partition_index: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ClaimedWorkflowJobsResponse {
    pub jobs: Vec<ClaimedWorkflowJobResponse>,
}

#[derive(Debug, Serialize)]
pub struct ClaimedWorkflowJobResponse {
    pub job_id: String,
    pub lease_token: String,
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
    let partition = parse_worker_partition(
        payload.partition_count,
        payload.partition_index,
        state.workflow_worker_max_partition_count,
    )?;

    let jobs = state
        .workflow_service
        .claim_jobs_for_worker(
            worker.worker_id(),
            effective_limit,
            effective_lease_seconds,
            partition,
        )
        .await?
        .into_iter()
        .map(|job| ClaimedWorkflowJobResponse {
            job_id: job.job_id,
            lease_token: job.lease_token,
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
