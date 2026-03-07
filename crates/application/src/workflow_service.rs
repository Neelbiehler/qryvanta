use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AuditAction, Permission, RuntimeRecord, WorkflowConditionOperator, WorkflowDefinition,
    WorkflowDefinitionInput, WorkflowStep, WorkflowTrigger, is_sensitive_workflow_header_name,
    redact_sensitive_workflow_headers, redact_workflow_header_secret_refs,
};
use serde_json::Value;

use crate::metadata_service::MetadataService;
use crate::workflow_ports::{
    ClaimedRuntimeRecordWorkflowEvent, ClaimedWorkflowJob, CompleteWorkflowRunInput,
    CreateWorkflowRunInput, SaveWorkflowInput, WorkflowActionDispatcher, WorkflowClaimPartition,
    WorkflowDelayService, WorkflowExecutionMode, WorkflowQueueStats, WorkflowQueueStatsCache,
    WorkflowQueueStatsQuery, WorkflowRepository, WorkflowRun, WorkflowRunAttempt,
    WorkflowRunAttemptStatus, WorkflowRunListQuery, WorkflowRunReplay,
    WorkflowRunReplayTimelineEvent, WorkflowRunStatus, WorkflowRunStepTrace,
    WorkflowRuntimeRecordService, WorkflowWorkerHeartbeatInput,
};
use crate::{AuditEvent, AuditRepository, AuthorizationService};

mod definitions;
mod dispatch;
mod execution;
mod queue;

#[async_trait]
impl WorkflowRuntimeRecordService for MetadataService {
    async fn has_published_entity_schema(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<bool> {
        Ok(self
            .latest_published_schema_unchecked(actor, entity_logical_name)
            .await?
            .is_some())
    }

    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.create_runtime_record_unchecked(actor, entity_logical_name, data)
            .await
    }

    async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.update_runtime_record_unchecked(actor, entity_logical_name, record_id, data)
            .await
    }

    async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.delete_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }

    async fn claim_runtime_record_workflow_events(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedRuntimeRecordWorkflowEvent>> {
        self.claim_runtime_record_workflow_events(worker_id, limit, lease_seconds, tenant_filter)
            .await
    }

    async fn complete_runtime_record_workflow_event(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        self.complete_runtime_record_workflow_event(tenant_id, event_id, worker_id, lease_token)
            .await
    }

    async fn release_runtime_record_workflow_event(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        self.release_runtime_record_workflow_event(
            tenant_id,
            event_id,
            worker_id,
            lease_token,
            error_message,
        )
        .await
    }
}

/// Workflow runtime service for trigger dispatch and execution history.
#[derive(Clone)]
pub struct WorkflowService {
    authorization_service: AuthorizationService,
    repository: Arc<dyn WorkflowRepository>,
    runtime_record_service: Arc<dyn WorkflowRuntimeRecordService>,
    action_dispatcher: Option<Arc<dyn WorkflowActionDispatcher>>,
    delay_service: Option<Arc<dyn WorkflowDelayService>>,
    audit_repository: Arc<dyn AuditRepository>,
    execution_mode: WorkflowExecutionMode,
    queue_stats_cache: Option<Arc<dyn WorkflowQueueStatsCache>>,
    queue_stats_cache_ttl_seconds: u32,
}

impl WorkflowService {
    /// Creates a workflow service.
    #[must_use]
    pub fn new(
        authorization_service: AuthorizationService,
        repository: Arc<dyn WorkflowRepository>,
        runtime_record_service: Arc<dyn WorkflowRuntimeRecordService>,
        audit_repository: Arc<dyn AuditRepository>,
        execution_mode: WorkflowExecutionMode,
    ) -> Self {
        Self {
            authorization_service,
            repository,
            runtime_record_service,
            action_dispatcher: None,
            delay_service: None,
            audit_repository,
            execution_mode,
            queue_stats_cache: None,
            queue_stats_cache_ttl_seconds: 0,
        }
    }

    /// Adds optional queue stats caching behavior.
    #[must_use]
    pub fn with_queue_stats_cache(
        mut self,
        queue_stats_cache: Arc<dyn WorkflowQueueStatsCache>,
        ttl_seconds: u32,
    ) -> Self {
        self.queue_stats_cache = Some(queue_stats_cache);
        self.queue_stats_cache_ttl_seconds = ttl_seconds;
        self
    }

    /// Adds optional external action dispatcher for integration actions.
    #[must_use]
    pub fn with_action_dispatcher(
        mut self,
        action_dispatcher: Arc<dyn WorkflowActionDispatcher>,
    ) -> Self {
        self.action_dispatcher = Some(action_dispatcher);
        self
    }

    /// Adds optional delay execution behavior for native delay steps.
    #[must_use]
    pub fn with_delay_service(mut self, delay_service: Arc<dyn WorkflowDelayService>) -> Self {
        self.delay_service = Some(delay_service);
        self
    }
}

#[cfg(test)]
mod tests;
