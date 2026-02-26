use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use qryvanta_core::{AppError, AppResult, UserIdentity};
use qryvanta_domain::{
    AuditAction, Permission, RuntimeRecord, WorkflowAction, WorkflowConditionOperator,
    WorkflowDefinition, WorkflowDefinitionInput, WorkflowStep, WorkflowTrigger,
};
use serde_json::Value;

use crate::metadata_service::MetadataService;
use crate::workflow_ports::{
    ClaimedWorkflowJob, CompleteWorkflowRunInput, CreateWorkflowRunInput, SaveWorkflowInput,
    WorkflowClaimPartition, WorkflowExecutionMode, WorkflowQueueStats, WorkflowQueueStatsCache,
    WorkflowQueueStatsQuery, WorkflowRepository, WorkflowRun, WorkflowRunAttempt,
    WorkflowRunAttemptStatus, WorkflowRunListQuery, WorkflowRunStatus,
    WorkflowRuntimeRecordService, WorkflowWorkerHeartbeatInput,
};
use crate::{AuditEvent, AuditRepository, AuthorizationService};

mod definitions;
mod dispatch;
mod execution;
mod queue;

#[async_trait]
impl WorkflowRuntimeRecordService for MetadataService {
    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.create_runtime_record_unchecked(actor, entity_logical_name, data)
            .await
    }
}

/// Workflow runtime service for trigger dispatch and execution history.
#[derive(Clone)]
pub struct WorkflowService {
    authorization_service: AuthorizationService,
    repository: Arc<dyn WorkflowRepository>,
    runtime_record_service: Arc<dyn WorkflowRuntimeRecordService>,
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
}

#[cfg(test)]
mod tests;
