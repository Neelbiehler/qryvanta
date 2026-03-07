use super::*;

#[derive(Debug, Deserialize)]
pub struct DrainRuntimeRecordWorkflowEventsRequest {
    pub limit: Option<usize>,
    pub lease_seconds: Option<u32>,
    pub tenant_id: Option<String>,
}

pub async fn drain_runtime_record_workflow_events_handler(
    State(state): State<AppState>,
    Extension(worker): Extension<WorkerIdentity>,
    Json(payload): Json<DrainRuntimeRecordWorkflowEventsRequest>,
) -> ApiResult<Json<RuntimeRecordWorkflowEventDrainResponse>> {
    let requested_limit = payload
        .limit
        .unwrap_or(state.workflow_worker_max_claim_limit);
    let requested_lease_seconds = payload
        .lease_seconds
        .unwrap_or(state.workflow_worker_default_lease_seconds);

    let effective_limit = requested_limit.clamp(1, state.workflow_worker_max_claim_limit);
    let effective_lease_seconds = requested_lease_seconds.max(1);
    let requested_tenant_filter = payload
        .tenant_id
        .as_deref()
        .map(claim::parse_tenant_id)
        .transpose()?;
    let tenant_filter = match state.physical_isolation_mode {
        crate::api_config::PhysicalIsolationMode::Shared => requested_tenant_filter,
        crate::api_config::PhysicalIsolationMode::TenantPerSchema
        | crate::api_config::PhysicalIsolationMode::TenantPerDatabase => {
            let scoped_tenant_id = state.physical_isolation_tenant_id.ok_or_else(|| {
                AppError::Validation(
                    "physical isolation tenant id is required for non-shared isolation mode"
                        .to_owned(),
                )
            })?;
            if let Some(requested_tenant_id) = requested_tenant_filter
                && requested_tenant_id != scoped_tenant_id
            {
                return Err(AppError::Validation(format!(
                    "worker tenant filter '{}' does not match configured physical isolation tenant '{}'",
                    requested_tenant_id, scoped_tenant_id
                ))
                .into());
            }

            Some(scoped_tenant_id)
        }
    };

    let result = state
        .workflow_service
        .drain_runtime_record_workflow_events_for_worker(
            worker.worker_id(),
            effective_limit,
            effective_lease_seconds,
            tenant_filter,
        )
        .await?;

    Ok(Json(RuntimeRecordWorkflowEventDrainResponse::from(result)))
}
