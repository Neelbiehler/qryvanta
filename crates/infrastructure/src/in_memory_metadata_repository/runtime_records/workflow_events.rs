use super::*;

impl InMemoryMetadataRepository {
    pub(in super::super) async fn enqueue_runtime_record_workflow_event_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        workflow_event: Option<RuntimeRecordWorkflowEventInput>,
    ) {
        let Some(workflow_event) = workflow_event else {
            return;
        };

        let event_id = Uuid::new_v4().to_string();
        self.runtime_workflow_events.write().await.insert(
            event_id.clone(),
            InMemoryRuntimeWorkflowEvent {
                event_id,
                tenant_id,
                trigger: workflow_event.trigger,
                record_id: record_id.to_owned(),
                payload: normalized_runtime_record_workflow_payload(
                    workflow_event.payload,
                    entity_logical_name,
                    record_id,
                ),
                emitted_by_subject: workflow_event.emitted_by_subject,
                status: InMemoryRuntimeWorkflowEventStatus::Pending,
                leased_by: None,
                lease_token: None,
                attempt_count: 0,
            },
        );
    }

    pub(in super::super) async fn claim_runtime_record_workflow_events_impl(
        &self,
        worker_id: &str,
        limit: usize,
        _lease_seconds: u32,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedRuntimeRecordWorkflowEvent>> {
        let mut events = self.runtime_workflow_events.write().await;
        let mut candidate_ids = events
            .values()
            .filter(|event| {
                event.status == InMemoryRuntimeWorkflowEventStatus::Pending
                    && tenant_filter
                        .map(|tenant_id| tenant_id == event.tenant_id)
                        .unwrap_or(true)
            })
            .map(|event| event.event_id.clone())
            .collect::<Vec<_>>();
        candidate_ids.sort();

        let mut claimed = Vec::new();
        for event_id in candidate_ids.into_iter().take(limit) {
            let Some(event) = events.get_mut(&event_id) else {
                continue;
            };
            let lease_token = Uuid::new_v4().to_string();
            event.status = InMemoryRuntimeWorkflowEventStatus::Leased;
            event.leased_by = Some(worker_id.to_owned());
            event.lease_token = Some(lease_token.clone());
            claimed.push(ClaimedRuntimeRecordWorkflowEvent {
                event_id: event.event_id.clone(),
                tenant_id: event.tenant_id,
                trigger: event.trigger.clone(),
                record_id: event.record_id.clone(),
                payload: event.payload.clone(),
                emitted_by_subject: event.emitted_by_subject.clone(),
                lease_token,
            });
        }

        Ok(claimed)
    }

    pub(in super::super) async fn complete_runtime_record_workflow_event_impl(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        let mut events = self.runtime_workflow_events.write().await;
        let event = events.get_mut(event_id).ok_or_else(|| {
            AppError::NotFound(format!(
                "runtime workflow event '{event_id}' does not exist"
            ))
        })?;
        ensure_matching_runtime_workflow_lease(event, tenant_id, event_id, worker_id, lease_token)?;

        event.status = InMemoryRuntimeWorkflowEventStatus::Completed;
        event.leased_by = None;
        event.lease_token = None;
        Ok(())
    }

    pub(in super::super) async fn release_runtime_record_workflow_event_impl(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
        _error_message: &str,
    ) -> AppResult<()> {
        let mut events = self.runtime_workflow_events.write().await;
        let event = events.get_mut(event_id).ok_or_else(|| {
            AppError::NotFound(format!(
                "runtime workflow event '{event_id}' does not exist"
            ))
        })?;
        ensure_matching_runtime_workflow_lease(event, tenant_id, event_id, worker_id, lease_token)?;

        event.status = InMemoryRuntimeWorkflowEventStatus::Pending;
        event.leased_by = None;
        event.lease_token = None;
        event.attempt_count = event.attempt_count.saturating_add(1);
        Ok(())
    }
}

fn ensure_matching_runtime_workflow_lease(
    event: &InMemoryRuntimeWorkflowEvent,
    tenant_id: TenantId,
    event_id: &str,
    worker_id: &str,
    lease_token: &str,
) -> AppResult<()> {
    if event.tenant_id != tenant_id
        || event.status != InMemoryRuntimeWorkflowEventStatus::Leased
        || event.leased_by.as_deref() != Some(worker_id)
        || event.lease_token.as_deref() != Some(lease_token)
    {
        return Err(AppError::Conflict(format!(
            "runtime workflow event '{event_id}' is not currently leased by worker '{worker_id}' with matching lease token"
        )));
    }

    Ok(())
}
