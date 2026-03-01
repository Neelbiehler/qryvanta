use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use tokio::sync::Mutex;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    Permission, WorkflowAction, WorkflowConditionOperator, WorkflowDefinition, WorkflowStep,
    WorkflowTrigger,
};

use crate::workflow_ports::{
    ClaimedWorkflowJob, CompleteWorkflowRunInput, CreateWorkflowRunInput, SaveWorkflowInput,
    WorkflowActionDispatchRequest, WorkflowActionDispatchType, WorkflowActionDispatcher,
    WorkflowClaimPartition, WorkflowExecutionMode, WorkflowQueueStats, WorkflowQueueStatsQuery,
    WorkflowRepository, WorkflowRun, WorkflowRunAttempt, WorkflowRunListQuery, WorkflowRunStatus,
    WorkflowRuntimeRecordService, WorkflowWorkerHeartbeatInput,
};
use crate::{
    AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService, RuntimeFieldGrant,
    TemporaryPermissionGrant,
};

use super::WorkflowService;

#[derive(Default)]
struct FakeAuditRepository;

#[async_trait]
impl AuditRepository for FakeAuditRepository {
    async fn append_event(&self, _event: AuditEvent) -> AppResult<()> {
        Ok(())
    }
}

struct FakeAuthorizationRepository {
    grants: HashMap<(TenantId, String), Vec<Permission>>,
}

#[async_trait]
impl AuthorizationRepository for FakeAuthorizationRepository {
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>> {
        Ok(self
            .grants
            .get(&(tenant_id, subject.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn list_runtime_field_grants_for_subject(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        Ok(Vec::new())
    }

    async fn find_active_temporary_permission_grant(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>> {
        Ok(None)
    }
}

#[derive(Default)]
struct FakeWorkflowRepository {
    workflows: Mutex<HashMap<(TenantId, String), WorkflowDefinition>>,
    runs: Mutex<Vec<WorkflowRun>>,
    attempts: Mutex<Vec<WorkflowRunAttempt>>,
    jobs: Mutex<Vec<FakeQueuedJob>>,
}

#[derive(Clone)]
struct FakeQueuedJob {
    job_id: String,
    tenant_id: TenantId,
    run_id: String,
    leased_by: Option<String>,
    lease_token: Option<String>,
    lease_version: u32,
    completed: bool,
    failed: bool,
}

#[async_trait]
impl WorkflowRepository for FakeWorkflowRepository {
    async fn save_workflow(
        &self,
        tenant_id: TenantId,
        workflow: WorkflowDefinition,
    ) -> AppResult<()> {
        self.workflows.lock().await.insert(
            (tenant_id, workflow.logical_name().as_str().to_owned()),
            workflow,
        );
        Ok(())
    }

    async fn list_workflows(&self, tenant_id: TenantId) -> AppResult<Vec<WorkflowDefinition>> {
        Ok(self
            .workflows
            .lock()
            .await
            .iter()
            .filter(|((stored_tenant_id, _), _)| *stored_tenant_id == tenant_id)
            .map(|(_, workflow)| workflow.clone())
            .collect())
    }

    async fn find_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        Ok(self
            .workflows
            .lock()
            .await
            .get(&(tenant_id, logical_name.to_owned()))
            .cloned())
    }

    async fn list_enabled_workflows_for_trigger(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        Ok(self
            .workflows
            .lock()
            .await
            .iter()
            .filter(|((stored_tenant_id, _), workflow)| {
                *stored_tenant_id == tenant_id
                    && workflow.is_enabled()
                    && workflow.trigger().trigger_type() == trigger.trigger_type()
                    && workflow.trigger().entity_logical_name() == trigger.entity_logical_name()
            })
            .map(|(_, workflow)| workflow.clone())
            .collect())
    }

    async fn create_run(
        &self,
        _tenant_id: TenantId,
        input: CreateWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        let run_id = format!("run-{}", self.runs.lock().await.len() + 1);
        let run = WorkflowRun {
            run_id,
            workflow_logical_name: input.workflow_logical_name,
            trigger_type: input.trigger_type,
            trigger_entity_logical_name: input.trigger_entity_logical_name,
            trigger_payload: input.trigger_payload,
            status: WorkflowRunStatus::Running,
            attempts: 0,
            dead_letter_reason: None,
            started_at: Utc::now(),
            finished_at: None,
        };

        self.runs.lock().await.push(run.clone());
        Ok(run)
    }

    async fn enqueue_run_job(&self, tenant_id: TenantId, run_id: &str) -> AppResult<()> {
        let mut jobs = self.jobs.lock().await;
        let next_id = jobs.len() + 1;
        jobs.push(FakeQueuedJob {
            job_id: format!("job-{next_id}"),
            tenant_id,
            run_id: run_id.to_owned(),
            leased_by: None,
            lease_token: None,
            lease_version: 0,
            completed: false,
            failed: false,
        });
        Ok(())
    }

    async fn claim_jobs(
        &self,
        worker_id: &str,
        limit: usize,
        _lease_seconds: u32,
        _partition: Option<WorkflowClaimPartition>,
    ) -> AppResult<Vec<ClaimedWorkflowJob>> {
        let mut jobs = self.jobs.lock().await;
        let workflows = self.workflows.lock().await;
        let runs = self.runs.lock().await;
        let mut claimed = Vec::new();

        for job in jobs
            .iter_mut()
            .filter(|entry| entry.leased_by.is_none() && !entry.completed && !entry.failed)
            .take(limit)
        {
            let run = runs
                .iter()
                .find(|run| run.run_id == job.run_id)
                .ok_or_else(|| AppError::NotFound(format!("run '{}' not found", job.run_id)))?;
            let workflow = workflows
                .get(&(job.tenant_id, run.workflow_logical_name.clone()))
                .cloned()
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "workflow '{}' not found",
                        run.workflow_logical_name
                    ))
                })?;

            job.leased_by = Some(worker_id.to_owned());
            job.lease_version = job.lease_version.saturating_add(1);
            let lease_token = format!("lease-{}-{}", job.job_id, job.lease_version);
            job.lease_token = Some(lease_token.clone());
            claimed.push(ClaimedWorkflowJob {
                job_id: job.job_id.clone(),
                tenant_id: job.tenant_id,
                run_id: job.run_id.clone(),
                workflow,
                trigger_payload: run.trigger_payload.clone(),
                lease_token,
            });
        }

        Ok(claimed)
    }

    async fn complete_job(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        let mut jobs = self.jobs.lock().await;
        let job = jobs
            .iter_mut()
            .find(|entry| entry.tenant_id == tenant_id && entry.job_id == job_id)
            .ok_or_else(|| AppError::NotFound(format!("job '{job_id}' not found")))?;

        if job.leased_by.as_deref() != Some(worker_id) {
            return Err(AppError::Conflict(format!(
                "job '{job_id}' is not leased by worker '{worker_id}'"
            )));
        }

        if job.lease_token.as_deref() != Some(lease_token) {
            return Err(AppError::Conflict(format!(
                "job '{job_id}' lease token does not match worker claim"
            )));
        }

        job.completed = true;
        job.lease_token = None;
        Ok(())
    }

    async fn fail_job(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
        _error_message: &str,
    ) -> AppResult<()> {
        let mut jobs = self.jobs.lock().await;
        let job = jobs
            .iter_mut()
            .find(|entry| entry.tenant_id == tenant_id && entry.job_id == job_id)
            .ok_or_else(|| AppError::NotFound(format!("job '{job_id}' not found")))?;

        if job.leased_by.as_deref() != Some(worker_id) {
            return Err(AppError::Conflict(format!(
                "job '{job_id}' is not leased by worker '{worker_id}'"
            )));
        }

        if job.lease_token.as_deref() != Some(lease_token) {
            return Err(AppError::Conflict(format!(
                "job '{job_id}' lease token does not match worker claim"
            )));
        }

        job.failed = true;
        job.lease_token = None;
        Ok(())
    }

    async fn upsert_worker_heartbeat(
        &self,
        _worker_id: &str,
        _input: WorkflowWorkerHeartbeatInput,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn queue_stats(&self, _query: WorkflowQueueStatsQuery) -> AppResult<WorkflowQueueStats> {
        Ok(WorkflowQueueStats {
            pending_jobs: 0,
            leased_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            expired_leases: 0,
            active_workers: 0,
        })
    }

    async fn append_run_attempt(
        &self,
        _tenant_id: TenantId,
        attempt: WorkflowRunAttempt,
    ) -> AppResult<()> {
        self.attempts.lock().await.push(attempt);
        Ok(())
    }

    async fn complete_run(
        &self,
        _tenant_id: TenantId,
        input: CompleteWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        let mut runs = self.runs.lock().await;
        let run = runs
            .iter_mut()
            .find(|run| run.run_id == input.run_id)
            .ok_or_else(|| AppError::NotFound(format!("run '{}' not found", input.run_id)))?;

        run.status = input.status;
        run.attempts = input.attempts;
        run.dead_letter_reason = input.dead_letter_reason;
        run.finished_at = Some(Utc::now());
        Ok(run.clone())
    }

    async fn list_runs(
        &self,
        _tenant_id: TenantId,
        _query: WorkflowRunListQuery,
    ) -> AppResult<Vec<WorkflowRun>> {
        Ok(self.runs.lock().await.clone())
    }

    async fn find_run(&self, _tenant_id: TenantId, run_id: &str) -> AppResult<Option<WorkflowRun>> {
        Ok(self
            .runs
            .lock()
            .await
            .iter()
            .find(|run| run.run_id == run_id)
            .cloned())
    }

    async fn list_run_attempts(
        &self,
        _tenant_id: TenantId,
        run_id: &str,
    ) -> AppResult<Vec<WorkflowRunAttempt>> {
        Ok(self
            .attempts
            .lock()
            .await
            .iter()
            .filter(|attempt| attempt.run_id == run_id)
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct FakeRuntimeRecordService {
    failures_remaining: Mutex<i32>,
    created_records: Mutex<Vec<(String, serde_json::Value)>>,
}

#[async_trait]
impl WorkflowRuntimeRecordService for FakeRuntimeRecordService {
    async fn create_runtime_record_unchecked(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
        data: serde_json::Value,
    ) -> AppResult<qryvanta_domain::RuntimeRecord> {
        let mut failures_remaining = self.failures_remaining.lock().await;
        if *failures_remaining > 0 {
            *failures_remaining -= 1;
            return Err(AppError::Internal(
                "simulated workflow action failure".to_owned(),
            ));
        }

        self.created_records
            .lock()
            .await
            .push((entity_logical_name.to_owned(), data));

        qryvanta_domain::RuntimeRecord::new("record-1", "contact", json!({"name": "Alice"}))
    }
}

#[derive(Default)]
struct FakeActionDispatcher {
    dispatched_requests: Mutex<Vec<WorkflowActionDispatchRequest>>,
}

#[async_trait]
impl WorkflowActionDispatcher for FakeActionDispatcher {
    async fn dispatch_action(&self, request: WorkflowActionDispatchRequest) -> AppResult<()> {
        self.dispatched_requests.lock().await.push(request);
        Ok(())
    }
}

fn build_service(
    grants: HashMap<(TenantId, String), Vec<Permission>>,
    repository: Arc<FakeWorkflowRepository>,
    runtime_service: Arc<FakeRuntimeRecordService>,
    execution_mode: WorkflowExecutionMode,
    action_dispatcher: Option<Arc<FakeActionDispatcher>>,
) -> WorkflowService {
    let audit_repository = Arc::new(FakeAuditRepository);
    let authorization_service = AuthorizationService::new(
        Arc::new(FakeAuthorizationRepository { grants }),
        audit_repository.clone(),
    );

    let service = WorkflowService::new(
        authorization_service,
        repository,
        runtime_service,
        audit_repository,
        execution_mode,
    );

    if let Some(dispatcher) = action_dispatcher {
        return service.with_action_dispatcher(dispatcher);
    }

    service
}

#[tokio::test]
async fn execute_workflow_dead_letters_after_max_attempts() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    *runtime_service.failures_remaining.lock().await = 3;

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "create_contact".to_owned(),
                display_name: "Create Contact".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({"name": "Alice"}),
                },
                steps: None,
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "create_contact", json!({"manual": true}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::DeadLettered);
    assert_eq!(run.attempts, 2);

    let attempts = repository
        .list_run_attempts(tenant_id, run.run_id.as_str())
        .await;
    assert!(attempts.is_ok());
    assert_eq!(attempts.unwrap_or_default().len(), 2);
}

#[tokio::test]
async fn retry_run_step_retries_failed_action_without_new_run() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    *runtime_service.failures_remaining.lock().await = 1;

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "retry_failed_step".to_owned(),
                display_name: "Retry Failed Step".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({"name": "Alice"}),
                },
                steps: Some(vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({"name": "Alice"}),
                }]),
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "retry_failed_step", json!({"manual": true}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::DeadLettered);
    assert_eq!(run.attempts, 1);

    let retried = service
        .retry_run_step(&actor, "retry_failed_step", run.run_id.as_str(), "0")
        .await;
    assert!(retried.is_ok());
    let retried = retried.unwrap_or_else(|_| unreachable!());
    assert_eq!(retried.status, WorkflowRunStatus::Succeeded);
    assert_eq!(retried.attempts, 2);

    let attempts = repository
        .list_run_attempts(tenant_id, run.run_id.as_str())
        .await;
    assert!(attempts.is_ok());
    let attempts = attempts.unwrap_or_default();
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[1].step_traces.len(), 1);
    assert_eq!(attempts[1].step_traces[0].status, "succeeded");
}

#[tokio::test]
async fn dispatch_runtime_record_created_executes_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "on_contact_created".to_owned(),
                display_name: "On Contact Created".to_owned(),
                description: None,
                trigger: WorkflowTrigger::RuntimeRecordCreated {
                    entity_logical_name: "contact".to_owned(),
                },
                action: WorkflowAction::LogMessage {
                    message: "created".to_owned(),
                },
                steps: None,
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let dispatched = service
        .dispatch_runtime_record_created(&actor, "contact", "record-1", &json!({"name": "Alice"}))
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);
}

#[tokio::test]
async fn dispatch_runtime_record_updated_executes_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "on_contact_updated".to_owned(),
                display_name: "On Contact Updated".to_owned(),
                description: None,
                trigger: WorkflowTrigger::RuntimeRecordUpdated {
                    entity_logical_name: "contact".to_owned(),
                },
                action: WorkflowAction::LogMessage {
                    message: "updated".to_owned(),
                },
                steps: None,
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let dispatched = service
        .dispatch_runtime_record_updated(
            &actor,
            "contact",
            "record-1",
            Some(&json!({"status": "open"})),
            &json!({"status": "closed"}),
        )
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);
}

#[tokio::test]
async fn dispatch_schedule_tick_executes_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "hourly_digest".to_owned(),
                display_name: "Hourly Digest".to_owned(),
                description: None,
                trigger: WorkflowTrigger::ScheduleTick {
                    schedule_key: "hourly".to_owned(),
                },
                action: WorkflowAction::LogMessage {
                    message: "schedule".to_owned(),
                },
                steps: None,
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let dispatched = service
        .dispatch_schedule_tick(
            &actor,
            "hourly",
            Some(json!({"tick": "2026-03-01T00:00:00Z"})),
        )
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);
}

#[tokio::test]
async fn execute_workflow_dispatches_external_integration_actions_with_idempotency_key() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        Some(action_dispatcher.clone()),
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "dispatch_http".to_owned(),
                display_name: "Dispatch HTTP".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::LogMessage {
                    message: "legacy".to_owned(),
                },
                steps: Some(vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "integration_http_request".to_owned(),
                    data: json!({
                        "method": "POST",
                        "url": "https://example.org/hook",
                        "body": {"record_id": "{{trigger.payload.record_id}}"}
                    }),
                }]),
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let run = service
        .execute_workflow(&actor, "dispatch_http", json!({"record_id": "rec-17"}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 1);
    assert_eq!(
        dispatched[0].dispatch_type,
        WorkflowActionDispatchType::HttpRequest
    );
    assert_eq!(dispatched[0].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(dispatched[0].payload["body"]["record_id"], json!("rec-17"));
}

#[tokio::test]
async fn execute_workflow_condition_branch_uses_trigger_payload() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "branch_by_status".to_owned(),
                display_name: "Branch By Status".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::LogMessage {
                    message: "fallback".to_owned(),
                },
                steps: Some(vec![WorkflowStep::Condition {
                    field_path: "status".to_owned(),
                    operator: WorkflowConditionOperator::Equals,
                    value: Some(json!("open")),
                    then_label: Some("Matched".to_owned()),
                    else_label: Some("Unmatched".to_owned()),
                    then_steps: vec![WorkflowStep::LogMessage {
                        message: "open-path".to_owned(),
                    }],
                    else_steps: vec![WorkflowStep::CreateRuntimeRecord {
                        entity_logical_name: "task".to_owned(),
                        data: json!({"title": "follow-up"}),
                    }],
                }]),
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let run = service
        .execute_workflow(&actor, "branch_by_status", json!({"status": "open"}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::Succeeded);
    assert_eq!(run.attempts, 1);
}

#[tokio::test]
async fn execute_workflow_interpolates_trigger_and_run_tokens_in_actions() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "interpolate_runtime_tokens".to_owned(),
                display_name: "Interpolate Runtime Tokens".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::CreateRuntimeRecord {
                    entity_logical_name: "outbox".to_owned(),
                    data: json!({"title": "fallback"}),
                },
                steps: Some(vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "{{trigger.payload.target_entity}}".to_owned(),
                    data: json!({
                        "title": "Record {{trigger.payload.record_id}}",
                        "record_id": "{{trigger.payload.record_id}}",
                        "run_id": "{{run.id}}",
                        "attempt": "{{run.attempt}}",
                        "owner": "{{trigger.payload.owner}}",
                    }),
                }]),
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let run = service
        .execute_workflow(
            &actor,
            "interpolate_runtime_tokens",
            json!({
                "target_entity": "contact",
                "record_id": "rec-42",
                "owner": "ops@qryvanta.test"
            }),
        )
        .await;
    assert!(run.is_ok());

    let created_records = runtime_service.created_records.lock().await.clone();
    assert_eq!(created_records.len(), 1);
    assert_eq!(created_records[0].0, "contact");
    assert_eq!(created_records[0].1["title"], json!("Record rec-42"));
    assert_eq!(created_records[0].1["record_id"], json!("rec-42"));
    assert_eq!(created_records[0].1["attempt"], json!(1));
    assert_eq!(created_records[0].1["owner"], json!("ops@qryvanta.test"));
    assert!(created_records[0].1["run_id"].as_str().is_some());
}

#[tokio::test]
async fn queued_mode_enqueues_and_worker_executes_claimed_job() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Queued,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "queued_contact_create".to_owned(),
                display_name: "Queued Contact Create".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::LogMessage {
                    message: "queued".to_owned(),
                },
                steps: None,
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let enqueued_run = service
        .execute_workflow(&actor, "queued_contact_create", json!({"source": "test"}))
        .await;
    assert!(enqueued_run.is_ok());
    let enqueued_run = enqueued_run.unwrap_or_else(|_| unreachable!());
    assert_eq!(enqueued_run.status, WorkflowRunStatus::Running);

    let claimed_jobs = service
        .claim_jobs_for_worker("worker-alpha", 10, 30, None)
        .await;
    assert!(claimed_jobs.is_ok());
    let mut claimed_jobs = claimed_jobs.unwrap_or_default();
    assert_eq!(claimed_jobs.len(), 1);

    let completed = service
        .execute_claimed_job("worker-alpha", claimed_jobs.remove(0))
        .await;
    assert!(completed.is_ok());
    let completed = completed.unwrap_or_else(|_| unreachable!());
    assert_eq!(completed.status, WorkflowRunStatus::Succeeded);
}

#[tokio::test]
async fn queued_mode_rejects_claimed_job_with_empty_lease_token() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![
                Permission::MetadataFieldWrite,
                Permission::MetadataFieldRead,
            ],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Queued,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "queued_contact_create_empty_token".to_owned(),
                display_name: "Queued Contact Create Empty Token".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::LogMessage {
                    message: "queued".to_owned(),
                },
                steps: None,
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let enqueued_run = service
        .execute_workflow(
            &actor,
            "queued_contact_create_empty_token",
            json!({"source": "test"}),
        )
        .await;
    assert!(enqueued_run.is_ok());

    let claimed_jobs = service
        .claim_jobs_for_worker("worker-alpha", 10, 30, None)
        .await;
    assert!(claimed_jobs.is_ok());
    let mut claimed_jobs = claimed_jobs.unwrap_or_default();
    assert_eq!(claimed_jobs.len(), 1);

    let mut claimed_job = claimed_jobs.remove(0);
    claimed_job.lease_token = String::new();

    let completed = service
        .execute_claimed_job("worker-alpha", claimed_job)
        .await;
    assert!(completed.is_err());
}

#[test]
fn workflow_claim_partition_rejects_invalid_index() {
    let partition = WorkflowClaimPartition::new(4, 4);
    assert!(partition.is_err());
}

#[test]
fn workflow_claim_partition_accepts_valid_values() {
    let partition = WorkflowClaimPartition::new(8, 3);
    assert!(partition.is_ok());
    let partition = partition.unwrap_or_else(|_| unreachable!());
    assert_eq!(partition.partition_count(), 8);
    assert_eq!(partition.partition_index(), 3);
}

#[tokio::test]
async fn queued_mode_supports_worker_heartbeat_and_queue_stats() {
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::new(),
        repository,
        runtime_service,
        WorkflowExecutionMode::Queued,
        None,
    );

    let heartbeat = service
        .heartbeat_worker(
            "worker-alpha",
            WorkflowWorkerHeartbeatInput {
                claimed_jobs: 2,
                executed_jobs: 2,
                failed_jobs: 0,
                partition: None,
            },
        )
        .await;
    assert!(heartbeat.is_ok());

    let stats = service.queue_stats(120).await;
    assert!(stats.is_ok());
    let stats = stats.unwrap_or_else(|_| unreachable!());
    assert_eq!(stats.pending_jobs, 0);
    assert_eq!(stats.active_workers, 0);
}
