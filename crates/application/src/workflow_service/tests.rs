use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use tokio::sync::Mutex;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    Permission, WorkflowConditionOperator, WorkflowDefinition, WorkflowLifecycleState,
    WorkflowStep, WorkflowTrigger,
};

use crate::workflow_ports::{
    ClaimedRuntimeRecordWorkflowEvent, ClaimedWorkflowJob, CompleteWorkflowRunInput,
    CreateWorkflowRunInput, SaveWorkflowInput, WorkflowActionDispatchRequest,
    WorkflowActionDispatchType, WorkflowActionDispatcher, WorkflowClaimPartition,
    WorkflowDelayService, WorkflowExecutionMode, WorkflowQueueStats, WorkflowQueueStatsQuery,
    WorkflowRepository, WorkflowRun, WorkflowRunAttempt, WorkflowRunAttemptStatus,
    WorkflowRunListQuery, WorkflowRunStatus, WorkflowRuntimeRecordService,
    WorkflowScheduledTrigger, WorkflowWorkerHeartbeatInput,
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
    published_workflows: Mutex<HashMap<(TenantId, String, i32), WorkflowDefinition>>,
    runs: Mutex<Vec<WorkflowRun>>,
    attempts: Mutex<Vec<WorkflowRunAttempt>>,
    jobs: Mutex<Vec<FakeQueuedJob>>,
    schedule_ticks: Mutex<Vec<FakeScheduleTick>>,
    fail_list_enabled_workflows_remaining: Mutex<i32>,
}

#[derive(Clone)]
struct FakeQueuedJob {
    job_id: String,
    tenant_id: TenantId,
    run_id: String,
    workflow_version: i32,
    leased_by: Option<String>,
    lease_token: Option<String>,
    lease_version: u32,
    completed: bool,
    failed: bool,
}

#[derive(Clone)]
struct FakeScheduleTick {
    tenant_id: TenantId,
    schedule_key: String,
    slot_key: String,
    scheduled_for: chrono::DateTime<Utc>,
    leased_by: Option<String>,
    lease_token: Option<String>,
    lease_version: u32,
    completed: bool,
    last_error: Option<String>,
}

#[async_trait]
impl WorkflowRepository for FakeWorkflowRepository {
    async fn save_workflow(
        &self,
        tenant_id: TenantId,
        workflow: WorkflowDefinition,
    ) -> AppResult<()> {
        let key = (tenant_id, workflow.logical_name().as_str().to_owned());
        let workflow = if let Some(existing) = self.workflows.lock().await.get(&key).cloned() {
            workflow.with_publish_state(existing.lifecycle_state(), existing.published_version())?
        } else {
            workflow
        };

        self.workflows.lock().await.insert(key, workflow);
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

    async fn find_published_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        let workflows = self.workflows.lock().await;
        let Some(current) = workflows
            .get(&(tenant_id, logical_name.to_owned()))
            .cloned()
        else {
            return Ok(None);
        };
        let Some(version) = current.published_version() else {
            return Ok(None);
        };
        drop(workflows);

        let published = self
            .published_workflows
            .lock()
            .await
            .get(&(tenant_id, logical_name.to_owned(), version))
            .cloned()
            .map(|workflow| workflow.with_publish_state(current.lifecycle_state(), Some(version)))
            .transpose()?;

        Ok(published)
    }

    async fn find_published_workflow_version(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        version: i32,
    ) -> AppResult<Option<WorkflowDefinition>> {
        let current = self
            .workflows
            .lock()
            .await
            .get(&(tenant_id, logical_name.to_owned()))
            .cloned();
        let lifecycle_state = current
            .map(|workflow| workflow.lifecycle_state())
            .unwrap_or(WorkflowLifecycleState::Disabled);

        self.published_workflows
            .lock()
            .await
            .get(&(tenant_id, logical_name.to_owned(), version))
            .cloned()
            .map(|workflow| workflow.with_publish_state(lifecycle_state, Some(version)))
            .transpose()
    }

    async fn publish_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
        _published_by: &str,
    ) -> AppResult<WorkflowDefinition> {
        let key = (tenant_id, logical_name.to_owned());
        let draft = self
            .workflows
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow '{}' does not exist for tenant '{}'",
                    logical_name, tenant_id
                ))
            })?;
        let next_version = draft.published_version().unwrap_or(0) + 1;
        let published =
            draft.with_publish_state(WorkflowLifecycleState::Published, Some(next_version))?;

        self.workflows
            .lock()
            .await
            .insert(key.clone(), published.clone());
        self.published_workflows.lock().await.insert(
            (tenant_id, logical_name.to_owned(), next_version),
            published.clone(),
        );

        Ok(published)
    }

    async fn disable_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<WorkflowDefinition> {
        let key = (tenant_id, logical_name.to_owned());
        let workflow = self
            .workflows
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow '{}' does not exist for tenant '{}'",
                    logical_name, tenant_id
                ))
            })?;
        let version = workflow.published_version().ok_or_else(|| {
            AppError::Conflict(format!(
                "workflow '{}' does not have a published version to disable",
                logical_name
            ))
        })?;
        let disabled =
            workflow.with_publish_state(WorkflowLifecycleState::Disabled, Some(version))?;
        self.workflows.lock().await.insert(key, disabled.clone());

        Ok(disabled)
    }

    async fn list_enabled_workflows_for_trigger(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        let mut failures_remaining = self.fail_list_enabled_workflows_remaining.lock().await;
        if *failures_remaining > 0 {
            *failures_remaining -= 1;
            return Err(AppError::Internal(
                "simulated trigger dispatch lookup failure".to_owned(),
            ));
        }

        let workflows = self.workflows.lock().await.clone();
        let published_workflows = self.published_workflows.lock().await.clone();

        Ok(workflows
            .iter()
            .filter_map(|((stored_tenant_id, logical_name), workflow)| {
                if *stored_tenant_id != tenant_id || !workflow.is_enabled() {
                    return None;
                }
                if workflow.trigger().trigger_type() != trigger.trigger_type()
                    || workflow.trigger().entity_logical_name() != trigger.entity_logical_name()
                {
                    return None;
                }

                workflow.published_version().and_then(|version| {
                    published_workflows
                        .get(&(*stored_tenant_id, logical_name.clone(), version))
                        .cloned()
                })
            })
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
            workflow_version: input.workflow_version,
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

    async fn list_enabled_schedule_triggers(
        &self,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<WorkflowScheduledTrigger>> {
        let workflows = self.workflows.lock().await.clone();
        let published_workflows = self.published_workflows.lock().await.clone();
        let mut seen = HashSet::new();
        let mut triggers = Vec::new();

        for ((stored_tenant_id, _), workflow) in workflows.iter() {
            if !workflow.is_enabled() {
                continue;
            }

            let Some(version) = workflow.published_version() else {
                continue;
            };
            let Some(published) = published_workflows.get(&(
                *stored_tenant_id,
                workflow.logical_name().as_str().to_owned(),
                version,
            )) else {
                continue;
            };

            let qryvanta_domain::WorkflowTrigger::ScheduleTick { schedule_key } =
                published.trigger()
            else {
                continue;
            };

            if tenant_filter
                .map(|selected_tenant_id| *stored_tenant_id == selected_tenant_id)
                .unwrap_or(true)
                && seen.insert((*stored_tenant_id, schedule_key.clone()))
            {
                triggers.push(WorkflowScheduledTrigger {
                    tenant_id: *stored_tenant_id,
                    schedule_key: schedule_key.clone(),
                });
            }
        }

        Ok(triggers)
    }

    async fn claim_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        scheduled_for: chrono::DateTime<Utc>,
        worker_id: &str,
        _lease_seconds: u32,
    ) -> AppResult<Option<crate::workflow_ports::ClaimedWorkflowScheduleTick>> {
        let mut ticks = self.schedule_ticks.lock().await;
        let tick = if let Some(existing) = ticks.iter_mut().find(|entry| {
            entry.tenant_id == tenant_id
                && entry.schedule_key == schedule_key
                && entry.slot_key == slot_key
        }) {
            existing
        } else {
            ticks.push(FakeScheduleTick {
                tenant_id,
                schedule_key: schedule_key.to_owned(),
                slot_key: slot_key.to_owned(),
                scheduled_for,
                leased_by: None,
                lease_token: None,
                lease_version: 0,
                completed: false,
                last_error: None,
            });
            ticks.last_mut().ok_or_else(|| {
                AppError::Internal("failed to store fake workflow schedule tick".to_owned())
            })?
        };

        if tick.completed || tick.leased_by.is_some() {
            return Ok(None);
        }

        tick.lease_version = tick.lease_version.saturating_add(1);
        let lease_token = format!(
            "schedule-lease-{}-{}-{}",
            schedule_key, slot_key, tick.lease_version
        );
        tick.leased_by = Some(worker_id.to_owned());
        tick.lease_token = Some(lease_token.clone());
        tick.last_error = None;

        Ok(Some(crate::workflow_ports::ClaimedWorkflowScheduleTick {
            tenant_id,
            schedule_key: schedule_key.to_owned(),
            slot_key: slot_key.to_owned(),
            scheduled_for,
            worker_id: worker_id.to_owned(),
            lease_token,
        }))
    }

    async fn complete_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        let mut ticks = self.schedule_ticks.lock().await;
        let tick = ticks
            .iter_mut()
            .find(|entry| {
                entry.tenant_id == tenant_id
                    && entry.schedule_key == schedule_key
                    && entry.slot_key == slot_key
            })
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "schedule tick '{schedule_key}/{slot_key}' not found"
                ))
            })?;

        if tick.leased_by.as_deref() != Some(worker_id) {
            return Err(AppError::Conflict(format!(
                "schedule tick '{schedule_key}/{slot_key}' is not leased by worker '{worker_id}'"
            )));
        }

        if tick.lease_token.as_deref() != Some(lease_token) {
            return Err(AppError::Conflict(format!(
                "schedule tick '{schedule_key}/{slot_key}' lease token does not match worker claim"
            )));
        }

        tick.completed = true;
        tick.leased_by = None;
        tick.lease_token = None;
        Ok(())
    }

    async fn release_schedule_tick(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        let mut ticks = self.schedule_ticks.lock().await;
        let tick = ticks
            .iter_mut()
            .find(|entry| {
                entry.tenant_id == tenant_id
                    && entry.schedule_key == schedule_key
                    && entry.slot_key == slot_key
            })
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "schedule tick '{schedule_key}/{slot_key}' not found"
                ))
            })?;

        if tick.leased_by.as_deref() != Some(worker_id) {
            return Err(AppError::Conflict(format!(
                "schedule tick '{schedule_key}/{slot_key}' is not leased by worker '{worker_id}'"
            )));
        }

        if tick.lease_token.as_deref() != Some(lease_token) {
            return Err(AppError::Conflict(format!(
                "schedule tick '{schedule_key}/{slot_key}' lease token does not match worker claim"
            )));
        }

        tick.leased_by = None;
        tick.lease_token = None;
        tick.last_error = Some(error_message.to_owned());
        Ok(())
    }

    async fn enqueue_run_job(&self, tenant_id: TenantId, run_id: &str) -> AppResult<()> {
        let mut jobs = self.jobs.lock().await;
        let runs = self.runs.lock().await;
        let workflow_version = runs
            .iter()
            .find(|run| run.run_id == run_id)
            .map(|run| run.workflow_version)
            .ok_or_else(|| AppError::NotFound(format!("run '{run_id}' not found")))?;
        let next_id = jobs.len() + 1;
        jobs.push(FakeQueuedJob {
            job_id: format!("job-{next_id}"),
            tenant_id,
            run_id: run_id.to_owned(),
            workflow_version,
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
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedWorkflowJob>> {
        let mut jobs = self.jobs.lock().await;
        let published_workflows = self.published_workflows.lock().await;
        let runs = self.runs.lock().await;
        let mut claimed = Vec::new();

        for job in jobs
            .iter_mut()
            .filter(|entry| {
                entry.leased_by.is_none()
                    && !entry.completed
                    && !entry.failed
                    && tenant_filter
                        .map(|selected_tenant_id| entry.tenant_id == selected_tenant_id)
                        .unwrap_or(true)
            })
            .take(limit)
        {
            let run = runs
                .iter()
                .find(|run| run.run_id == job.run_id)
                .ok_or_else(|| AppError::NotFound(format!("run '{}' not found", job.run_id)))?;
            let workflow = published_workflows
                .get(&(
                    job.tenant_id,
                    run.workflow_logical_name.clone(),
                    job.workflow_version,
                ))
                .cloned()
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "published workflow '{}@v{}' not found",
                        run.workflow_logical_name, job.workflow_version
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
                workflow_version: job.workflow_version,
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

struct FakeRuntimeRecordService {
    assume_entities_published: bool,
    failures_remaining: Mutex<i32>,
    published_entities: Mutex<HashSet<String>>,
    created_records: Mutex<Vec<(String, serde_json::Value)>>,
    updated_records: Mutex<Vec<(String, String, serde_json::Value)>>,
    deleted_records: Mutex<Vec<(String, String)>>,
    queued_events: Mutex<Vec<ClaimedRuntimeRecordWorkflowEvent>>,
    leased_events: Mutex<HashMap<String, ClaimedRuntimeRecordWorkflowEvent>>,
    completed_event_ids: Mutex<Vec<String>>,
    released_event_ids: Mutex<Vec<String>>,
}

impl Default for FakeRuntimeRecordService {
    fn default() -> Self {
        Self {
            assume_entities_published: true,
            failures_remaining: Mutex::new(0),
            published_entities: Mutex::new(HashSet::new()),
            created_records: Mutex::new(Vec::new()),
            updated_records: Mutex::new(Vec::new()),
            deleted_records: Mutex::new(Vec::new()),
            queued_events: Mutex::new(Vec::new()),
            leased_events: Mutex::new(HashMap::new()),
            completed_event_ids: Mutex::new(Vec::new()),
            released_event_ids: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl WorkflowRuntimeRecordService for FakeRuntimeRecordService {
    async fn has_published_entity_schema(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<bool> {
        Ok(self.assume_entities_published
            || self
                .published_entities
                .lock()
                .await
                .contains(entity_logical_name))
    }

    async fn update_runtime_record_unchecked(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: serde_json::Value,
    ) -> AppResult<qryvanta_domain::RuntimeRecord> {
        let mut failures_remaining = self.failures_remaining.lock().await;
        if *failures_remaining > 0 {
            *failures_remaining -= 1;
            return Err(AppError::Internal(
                "simulated workflow action failure".to_owned(),
            ));
        }

        self.updated_records.lock().await.push((
            entity_logical_name.to_owned(),
            record_id.to_owned(),
            data.clone(),
        ));

        qryvanta_domain::RuntimeRecord::new(record_id, entity_logical_name, data)
    }

    async fn delete_runtime_record_unchecked(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        let mut failures_remaining = self.failures_remaining.lock().await;
        if *failures_remaining > 0 {
            *failures_remaining -= 1;
            return Err(AppError::Internal(
                "simulated workflow action failure".to_owned(),
            ));
        }

        self.deleted_records
            .lock()
            .await
            .push((entity_logical_name.to_owned(), record_id.to_owned()));

        Ok(())
    }

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

    async fn claim_runtime_record_workflow_events(
        &self,
        _worker_id: &str,
        limit: usize,
        _lease_seconds: u32,
        _tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<ClaimedRuntimeRecordWorkflowEvent>> {
        let mut queued_events = self.queued_events.lock().await;
        let count = limit.min(queued_events.len());
        let claimed: Vec<_> = queued_events.drain(..count).collect();
        drop(queued_events);

        let mut leased_events = self.leased_events.lock().await;
        for event in &claimed {
            leased_events.insert(event.event_id.clone(), event.clone());
        }

        Ok(claimed)
    }

    async fn complete_runtime_record_workflow_event(
        &self,
        _tenant_id: TenantId,
        event_id: &str,
        _worker_id: &str,
        _lease_token: &str,
    ) -> AppResult<()> {
        self.leased_events.lock().await.remove(event_id);
        self.completed_event_ids
            .lock()
            .await
            .push(event_id.to_owned());
        Ok(())
    }

    async fn release_runtime_record_workflow_event(
        &self,
        _tenant_id: TenantId,
        event_id: &str,
        _worker_id: &str,
        _lease_token: &str,
        _error_message: &str,
    ) -> AppResult<()> {
        if let Some(event) = self.leased_events.lock().await.remove(event_id) {
            self.queued_events.lock().await.push(event);
        }
        self.released_event_ids
            .lock()
            .await
            .push(event_id.to_owned());
        Ok(())
    }
}

#[derive(Default)]
struct FakeActionDispatcher {
    dispatched_requests: Mutex<Vec<WorkflowActionDispatchRequest>>,
    failures_remaining: Mutex<i32>,
    failure_messages: Mutex<Vec<String>>,
}

#[async_trait]
impl WorkflowActionDispatcher for FakeActionDispatcher {
    async fn dispatch_action(&self, request: WorkflowActionDispatchRequest) -> AppResult<()> {
        self.dispatched_requests.lock().await.push(request);

        let mut failure_messages = self.failure_messages.lock().await;
        if let Some(message) = (!failure_messages.is_empty()).then(|| failure_messages.remove(0)) {
            return Err(AppError::Internal(message));
        }

        let mut failures_remaining = self.failures_remaining.lock().await;
        if *failures_remaining > 0 {
            *failures_remaining -= 1;
            return Err(AppError::Internal(
                "simulated integration dispatch failure".to_owned(),
            ));
        }

        Ok(())
    }
}

#[derive(Default)]
struct FakeDelayService {
    sleep_calls: Mutex<Vec<u64>>,
}

#[async_trait]
impl WorkflowDelayService for FakeDelayService {
    async fn sleep(&self, duration_ms: u64) -> AppResult<()> {
        self.sleep_calls.lock().await.push(duration_ms);
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
    )
    .with_delay_service(Arc::new(FakeDelayService::default()));

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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({"name": "Alice"}),
                }],
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({"name": "Alice"}),
                }],
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
async fn replay_run_reconstructs_ordered_timeline_and_stable_checksum() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    *runtime_service.failures_remaining.lock().await = 1;

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "replay_flow".to_owned(),
                display_name: "Replay Flow".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({"name": "Alice"}),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "replay_flow", json!({"manual": true}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::Succeeded);
    assert_eq!(run.attempts, 2);

    let replay = service
        .replay_run(&actor, "replay_flow", run.run_id.as_str())
        .await;
    assert!(replay.is_ok());
    let replay = replay.unwrap_or_else(|_| unreachable!());

    assert_eq!(replay.run.run_id, run.run_id);
    assert_eq!(replay.attempts.len(), 2);
    assert_eq!(replay.attempts[0].attempt_number, 1);
    assert_eq!(replay.attempts[1].attempt_number, 2);
    assert_eq!(replay.timeline.len(), 2);
    assert_eq!(replay.timeline[0].sequence, 1);
    assert_eq!(replay.timeline[0].attempt_number, 1);
    assert_eq!(replay.timeline[0].attempt_status.as_str(), "failed");
    assert_eq!(replay.timeline[1].sequence, 2);
    assert_eq!(replay.timeline[1].attempt_number, 2);
    assert_eq!(replay.timeline[1].attempt_status.as_str(), "succeeded");
    assert_eq!(replay.timeline[0].step_path, "0");
    assert_eq!(replay.timeline[1].step_path, "0");

    let replay_again = service
        .replay_run(&actor, "replay_flow", run.run_id.as_str())
        .await;
    assert!(replay_again.is_ok());
    let replay_again = replay_again.unwrap_or_else(|_| unreachable!());
    assert_eq!(replay_again.checksum_sha256, replay.checksum_sha256);
    assert_eq!(replay_again.timeline, replay.timeline);
}

#[tokio::test]
async fn replay_run_rejects_mismatched_workflow_name() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "replay_name_guard".to_owned(),
                display_name: "Replay Name Guard".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::LogMessage {
                    message: "ok".to_owned(),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "replay_name_guard", json!({"manual": true}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());

    let replay = service
        .replay_run(&actor, "other_workflow", run.run_id.as_str())
        .await;
    match replay {
        Ok(_) => panic!("expected replay_run to reject mismatched workflow name"),
        Err(error) => assert!(matches!(error, AppError::Validation(_))),
    }
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::LogMessage {
                    message: "created".to_owned(),
                }],
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::LogMessage {
                    message: "updated".to_owned(),
                }],
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::LogMessage {
                    message: "schedule".to_owned(),
                }],
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
async fn dispatch_schedule_tick_normalizes_timestamp_timezone_and_clock_skew() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "hourly_digest_policy".to_owned(),
                display_name: "Hourly Digest Policy".to_owned(),
                description: None,
                trigger: WorkflowTrigger::ScheduleTick {
                    schedule_key: "hourly".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "schedule".to_owned(),
                }],
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
            Some(json!({
                "tick_at": "2026-03-03T12:00:00+02:00",
                "timezone": "Europe/Berlin",
                "source": "scheduler-a",
            })),
        )
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);

    let runs = repository.runs.lock().await.clone();
    assert_eq!(runs.len(), 1);
    let payload = &runs[0].trigger_payload;
    assert_eq!(payload["schedule_key"], json!("hourly"));
    assert_eq!(payload["event"], json!("schedule_tick"));
    assert_eq!(payload["tick_at_utc"], json!("2026-03-03T10:00:00+00:00"));
    assert_eq!(payload["tick_source"], json!("payload"));
    assert_eq!(payload["timezone"], json!("Europe/Berlin"));
    assert_eq!(payload["clock_skew_tolerance_seconds"], json!(300));
    assert!(payload["clock_skew_seconds"].as_i64().is_some());
    assert!(payload["clock_skew_within_tolerance"].is_boolean());
    assert_eq!(payload["data"]["source"], json!("scheduler-a"));
}

#[tokio::test]
async fn dispatch_schedule_tick_rejects_invalid_tick_timestamp() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                logical_name: "hourly_digest_invalid_tick".to_owned(),
                display_name: "Hourly Digest Invalid Tick".to_owned(),
                description: None,
                trigger: WorkflowTrigger::ScheduleTick {
                    schedule_key: "hourly".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "schedule".to_owned(),
                }],
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
            Some(json!({
                "tick_at": "not-a-rfc3339-timestamp",
            })),
        )
        .await;

    match dispatched {
        Ok(_) => panic!("expected dispatch_schedule_tick to fail for invalid tick_at"),
        Err(error) => assert!(matches!(error, AppError::Validation(_))),
    }
}

#[tokio::test]
async fn dispatch_webhook_received_executes_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "incoming_webhook".to_owned(),
                display_name: "Incoming Webhook".to_owned(),
                description: None,
                trigger: WorkflowTrigger::WebhookReceived {
                    webhook_key: "customer_created".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "webhook".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let dispatched = service
        .dispatch_webhook_received(
            tenant_id,
            "customer_created",
            json!({
                "request": {
                    "method": "POST",
                    "headers": {
                        "content-type": "application/json"
                    },
                    "query": {
                        "source": "crm"
                    }
                },
                "payload": {
                    "customer_id": "cust-1"
                },
                "data": {
                    "customer_id": "cust-1"
                }
            }),
        )
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);

    let runs = repository.runs.lock().await.clone();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].trigger_type, "webhook_received");
    assert_eq!(
        runs[0].trigger_entity_logical_name.as_deref(),
        Some("customer_created")
    );
    assert_eq!(runs[0].trigger_payload["event"], json!("webhook_received"));
    assert_eq!(
        runs[0].trigger_payload["payload"]["customer_id"],
        json!("cust-1")
    );
}

#[tokio::test]
async fn dispatch_form_submitted_executes_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "lead_capture_form".to_owned(),
                display_name: "Lead Capture Form".to_owned(),
                description: None,
                trigger: WorkflowTrigger::FormSubmitted {
                    form_key: "lead_capture".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "form".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let dispatched = service
        .dispatch_form_submitted(
            tenant_id,
            "lead_capture",
            json!({
                "submission": {
                    "source": "landing_page"
                },
                "payload": {
                    "email": "alice@example.com"
                },
                "data": {
                    "email": "alice@example.com"
                }
            }),
        )
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);

    let runs = repository.runs.lock().await.clone();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].trigger_type, "form_submitted");
    assert_eq!(
        runs[0].trigger_entity_logical_name.as_deref(),
        Some("lead_capture")
    );
    assert_eq!(runs[0].trigger_payload["event"], json!("form_submitted"));
    assert_eq!(
        runs[0].trigger_payload["payload"]["email"],
        json!("alice@example.com")
    );
}

#[tokio::test]
async fn dispatch_inbound_email_received_executes_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "support_mailbox".to_owned(),
                display_name: "Support Mailbox".to_owned(),
                description: None,
                trigger: WorkflowTrigger::InboundEmailReceived {
                    mailbox_key: "support".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "email".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let dispatched = service
        .dispatch_inbound_email_received(
            tenant_id,
            "support",
            json!({
                "message": {
                    "from": "alice@example.com",
                    "subject": "Need help",
                    "text_body": "Please assist."
                },
                "payload": {
                    "from": "alice@example.com",
                    "subject": "Need help",
                    "text_body": "Please assist."
                },
                "data": {
                    "from": "alice@example.com",
                    "subject": "Need help",
                    "text_body": "Please assist."
                }
            }),
        )
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);

    let runs = repository.runs.lock().await.clone();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].trigger_type, "inbound_email_received");
    assert_eq!(
        runs[0].trigger_entity_logical_name.as_deref(),
        Some("support")
    );
    assert_eq!(
        runs[0].trigger_payload["event"],
        json!("inbound_email_received")
    );
    assert_eq!(
        runs[0].trigger_payload["payload"]["subject"],
        json!("Need help")
    );
}

#[tokio::test]
async fn dispatch_approval_event_received_executes_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "approval_events".to_owned(),
                display_name: "Approval Events".to_owned(),
                description: None,
                trigger: WorkflowTrigger::ApprovalEventReceived {
                    approval_key: "manager_signoff".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "approval".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let dispatched = service
        .dispatch_approval_event_received(
            tenant_id,
            "manager_signoff",
            json!({
                "approval": {
                    "request_id": "req-1",
                    "status": "approved",
                    "approver_id": "manager-7",
                    "comment": "looks good"
                },
                "payload": {
                    "request_id": "req-1",
                    "status": "approved",
                    "approver_id": "manager-7",
                    "comment": "looks good"
                },
                "data": {
                    "request_id": "req-1",
                    "status": "approved",
                    "approver_id": "manager-7",
                    "comment": "looks good"
                }
            }),
        )
        .await;

    assert!(dispatched.is_ok());
    assert_eq!(dispatched.unwrap_or_default(), 1);

    let runs = repository.runs.lock().await.clone();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].trigger_type, "approval_event_received");
    assert_eq!(
        runs[0].trigger_entity_logical_name.as_deref(),
        Some("manager_signoff")
    );
    assert_eq!(
        runs[0].trigger_payload["event"],
        json!("approval_event_received")
    );
    assert_eq!(
        runs[0].trigger_payload["payload"]["status"],
        json!("approved")
    );
}

#[tokio::test]
async fn dispatch_due_schedule_ticks_enqueues_due_runs_once_per_slot() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Queued,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "hourly_scheduler_worker".to_owned(),
                display_name: "Hourly Scheduler Worker".to_owned(),
                description: None,
                trigger: WorkflowTrigger::ScheduleTick {
                    schedule_key: "hourly".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "tick".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let first = service
        .dispatch_due_schedule_ticks("worker-alpha", 30, Some(tenant_id))
        .await;
    assert!(first.is_ok());
    let first = first.unwrap_or_default();
    assert_eq!(first.claimed_ticks, 1);
    assert_eq!(first.dispatched_workflows, 1);
    assert_eq!(first.released_ticks, 0);

    let second = service
        .dispatch_due_schedule_ticks("worker-alpha", 30, Some(tenant_id))
        .await;
    assert!(second.is_ok());
    let second = second.unwrap_or_default();
    assert_eq!(second.claimed_ticks, 0);
    assert_eq!(second.dispatched_workflows, 0);
    assert_eq!(second.released_ticks, 0);

    let runs = repository.runs.lock().await.clone();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].trigger_type, "schedule_tick");
    assert_eq!(runs[0].trigger_payload["schedule_key"], json!("hourly"));

    let jobs = repository.jobs.lock().await.clone();
    assert_eq!(jobs.len(), 1);

    let ticks = repository.schedule_ticks.lock().await.clone();
    assert_eq!(ticks.len(), 1);
    assert!(ticks[0].completed);
    assert_eq!(ticks[0].leased_by, None);
    assert!(ticks[0].scheduled_for <= Utc::now());
}

#[tokio::test]
async fn dispatch_due_schedule_ticks_skips_non_matching_tenant_scope() {
    let tenant_a = TenantId::new();
    let tenant_b = TenantId::new();
    let actor_a = UserIdentity::new("maker-a", "maker-a", None, tenant_a);
    let actor_b = UserIdentity::new("maker-b", "maker-b", None, tenant_b);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([
            (
                (tenant_a, "maker-a".to_owned()),
                vec![Permission::WorkflowManage, Permission::WorkflowRead],
            ),
            (
                (tenant_b, "maker-b".to_owned()),
                vec![Permission::WorkflowManage, Permission::WorkflowRead],
            ),
        ]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Queued,
        None,
    );

    for (actor, logical_name, schedule_key) in [
        (&actor_a, "tenant_a_schedule", "hourly"),
        (&actor_b, "tenant_b_schedule", "hourly"),
    ] {
        let save_result = service
            .save_workflow(
                actor,
                SaveWorkflowInput {
                    logical_name: logical_name.to_owned(),
                    display_name: logical_name.to_owned(),
                    description: None,
                    trigger: WorkflowTrigger::ScheduleTick {
                        schedule_key: schedule_key.to_owned(),
                    },
                    steps: vec![WorkflowStep::LogMessage {
                        message: "tick".to_owned(),
                    }],
                    max_attempts: 2,
                    is_enabled: true,
                },
            )
            .await;
        assert!(save_result.is_ok());
    }

    let result = service
        .dispatch_due_schedule_ticks("worker-tenant-a", 30, Some(tenant_a))
        .await;
    assert!(result.is_ok());
    let result = result.unwrap_or_default();
    assert_eq!(result.claimed_ticks, 1);
    assert_eq!(result.dispatched_workflows, 1);

    let runs = repository.runs.lock().await.clone();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].workflow_logical_name, "tenant_a_schedule");

    let ticks = repository.schedule_ticks.lock().await.clone();
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].tenant_id, tenant_a);
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::HttpRequest {
                    method: "POST".to_owned(),
                    url: "https://example.org/hook".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    body: Some(json!({
                        "record_id": "{{trigger.payload.record_id}}"
                    })),
                }],
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
async fn external_integration_idempotency_key_is_stable_across_run_retries() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());
    *action_dispatcher.failures_remaining.lock().await = 1;

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        Some(action_dispatcher.clone()),
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "retry_http_dispatch".to_owned(),
                display_name: "Retry HTTP Dispatch".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::HttpRequest {
                    method: "POST".to_owned(),
                    url: "https://example.org/retry".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    body: None,
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "retry_http_dispatch", json!({"source": "wf-06"}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::Succeeded);
    assert_eq!(run.attempts, 2);

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 2);
    assert_eq!(dispatched[0].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(dispatched[1].idempotency_key, format!("{}:0", run.run_id));
}

#[tokio::test]
async fn external_integration_idempotency_key_is_stable_for_step_retry() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());
    *action_dispatcher.failures_remaining.lock().await = 1;

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        Some(action_dispatcher.clone()),
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "retry_webhook_dispatch".to_owned(),
                display_name: "Retry Webhook Dispatch".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::Webhook {
                    endpoint: "https://example.org/retry-webhook".to_owned(),
                    event: "updated".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    payload: json!({"source": "{{trigger.payload.source}}"}),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "retry_webhook_dispatch", json!({"source": "wf-06"}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::DeadLettered);
    assert_eq!(run.attempts, 1);

    let retried = service
        .retry_run_step(&actor, "retry_webhook_dispatch", run.run_id.as_str(), "0")
        .await;
    assert!(retried.is_ok());
    let retried = retried.unwrap_or_else(|_| unreachable!());
    assert_eq!(retried.status, WorkflowRunStatus::Succeeded);
    assert_eq!(retried.attempts, 2);

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 2);
    assert_eq!(dispatched[0].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(dispatched[1].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(
        dispatched[0].dispatch_type,
        WorkflowActionDispatchType::Webhook
    );
    assert_eq!(
        dispatched[1].dispatch_type,
        WorkflowActionDispatchType::Webhook
    );
}

#[tokio::test]
async fn outbound_http_action_dead_letters_after_repeated_429_failures() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());
    action_dispatcher.failure_messages.lock().await.extend([
        "429 Too Many Requests from downstream".to_owned(),
        "429 Too Many Requests from downstream".to_owned(),
    ]);

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        Some(action_dispatcher.clone()),
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "dead_letter_http_429".to_owned(),
                display_name: "Dead Letter HTTP 429".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::HttpRequest {
                    method: "POST".to_owned(),
                    url: "https://example.org/rate-limited".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    body: Some(json!({ "record_id": "{{trigger.payload.record_id}}" })),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(
            &actor,
            "dead_letter_http_429",
            json!({ "record_id": "rec-429" }),
        )
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::DeadLettered);
    assert_eq!(run.attempts, 2);
    assert_eq!(
        run.dead_letter_reason.as_deref(),
        Some("internal error: 429 Too Many Requests from downstream")
    );

    let attempts = service
        .list_run_attempts(&actor, run.run_id.as_str())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(attempts.len(), 2);
    assert!(
        attempts
            .iter()
            .all(|attempt| attempt.status == WorkflowRunAttemptStatus::Failed)
    );
    assert!(attempts.iter().all(|attempt| {
        attempt.error_message.as_deref()
            == Some("internal error: 429 Too Many Requests from downstream")
    }));

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 2);
    assert_eq!(dispatched[0].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(dispatched[1].idempotency_key, format!("{}:0", run.run_id));
}

#[tokio::test]
async fn outbound_webhook_action_dead_letters_after_repeated_5xx_failures() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());
    action_dispatcher.failure_messages.lock().await.extend([
        "502 Bad Gateway from downstream".to_owned(),
        "503 Service Unavailable from downstream".to_owned(),
    ]);

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        Some(action_dispatcher.clone()),
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "dead_letter_webhook_5xx".to_owned(),
                display_name: "Dead Letter Webhook 5xx".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::Webhook {
                    endpoint: "https://example.org/downstream-webhook".to_owned(),
                    event: "contact.created".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    payload: json!({
                        "record_id": "{{trigger.payload.record_id}}",
                        "status": "{{trigger.payload.status}}"
                    }),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(
            &actor,
            "dead_letter_webhook_5xx",
            json!({ "record_id": "rec-5xx", "status": "new" }),
        )
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::DeadLettered);
    assert_eq!(run.attempts, 2);
    assert_eq!(
        run.dead_letter_reason.as_deref(),
        Some("internal error: 503 Service Unavailable from downstream")
    );

    let attempts = service
        .list_run_attempts(&actor, run.run_id.as_str())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].status, WorkflowRunAttemptStatus::Failed);
    assert_eq!(attempts[1].status, WorkflowRunAttemptStatus::Failed);
    assert_eq!(
        attempts[0].error_message.as_deref(),
        Some("internal error: 502 Bad Gateway from downstream")
    );
    assert_eq!(
        attempts[1].error_message.as_deref(),
        Some("internal error: 503 Service Unavailable from downstream")
    );

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 2);
    assert_eq!(dispatched[0].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(dispatched[1].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(
        dispatched[0].dispatch_type,
        WorkflowActionDispatchType::Webhook
    );
    assert_eq!(
        dispatched[1].dispatch_type,
        WorkflowActionDispatchType::Webhook
    );
}

#[tokio::test]
async fn outbound_email_action_dead_letters_after_repeated_provider_failures() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());
    action_dispatcher.failure_messages.lock().await.extend([
        "502 Bad Gateway from email provider".to_owned(),
        "503 Service Unavailable from email provider".to_owned(),
    ]);

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        Some(action_dispatcher.clone()),
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "dead_letter_email_provider_5xx".to_owned(),
                display_name: "Dead Letter Email Provider 5xx".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::SendEmail {
                    to: "ops@qryvanta.test".to_owned(),
                    subject: "Workflow delivery failed".to_owned(),
                    body: "record {{trigger.payload.record_id}} failed".to_owned(),
                    html_body: None,
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(
            &actor,
            "dead_letter_email_provider_5xx",
            json!({ "record_id": "rec-email-5xx" }),
        )
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::DeadLettered);
    assert_eq!(run.attempts, 2);
    assert_eq!(
        run.dead_letter_reason.as_deref(),
        Some("internal error: 503 Service Unavailable from email provider")
    );

    let attempts = service
        .list_run_attempts(&actor, run.run_id.as_str())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].status, WorkflowRunAttemptStatus::Failed);
    assert_eq!(attempts[1].status, WorkflowRunAttemptStatus::Failed);
    assert_eq!(
        attempts[0].error_message.as_deref(),
        Some("internal error: 502 Bad Gateway from email provider")
    );
    assert_eq!(
        attempts[1].error_message.as_deref(),
        Some("internal error: 503 Service Unavailable from email provider")
    );

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 2);
    assert_eq!(
        dispatched[0].dispatch_type,
        WorkflowActionDispatchType::Email
    );
    assert_eq!(
        dispatched[1].dispatch_type,
        WorkflowActionDispatchType::Email
    );
    assert_eq!(dispatched[0].idempotency_key, format!("{}:0", run.run_id));
    assert_eq!(dispatched[1].idempotency_key, format!("{}:0", run.run_id));
}

#[tokio::test]
async fn external_integration_idempotency_key_uses_deterministic_nested_step_path() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        Some(action_dispatcher.clone()),
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "nested_email_dispatch".to_owned(),
                display_name: "Nested Email Dispatch".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::Condition {
                    field_path: "status".to_owned(),
                    operator: WorkflowConditionOperator::Equals,
                    value: Some(json!("open")),
                    then_label: None,
                    else_label: None,
                    then_steps: vec![WorkflowStep::SendEmail {
                        to: "ops@qryvanta.test".to_owned(),
                        subject: "Workflow update".to_owned(),
                        body: "status changed".to_owned(),
                        html_body: None,
                    }],
                    else_steps: Vec::new(),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "nested_email_dispatch", json!({"status": "open"}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::Succeeded);

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 1);
    assert_eq!(
        dispatched[0].dispatch_type,
        WorkflowActionDispatchType::Email
    );
    assert_eq!(
        dispatched[0].idempotency_key,
        format!("{}:0.then.0", run.run_id)
    );
    assert_eq!(dispatched[0].step_path, "0.then.0");
}

#[tokio::test]
async fn native_update_record_step_updates_runtime_record() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "update_contact".to_owned(),
                display_name: "Update Contact".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::UpdateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    record_id: "{{trigger.payload.record_id}}".to_owned(),
                    data: json!({"status": "qualified"}),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let executed = service
        .execute_workflow(&actor, "update_contact", json!({"record_id": "rec-42"}))
        .await;
    assert!(executed.is_ok());

    let updated = runtime_service.updated_records.lock().await.clone();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].0, "contact");
    assert_eq!(updated[0].1, "rec-42");
    assert_eq!(updated[0].2["status"], json!("qualified"));
}

#[tokio::test]
async fn native_delete_record_step_deletes_runtime_record() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "delete_contact".to_owned(),
                display_name: "Delete Contact".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::DeleteRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    record_id: "rec-7".to_owned(),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let executed = service
        .execute_workflow(&actor, "delete_contact", json!({}))
        .await;
    assert!(executed.is_ok());

    let deleted = runtime_service.deleted_records.lock().await.clone();
    assert_eq!(
        deleted.as_slice(),
        [("contact".to_owned(), "rec-7".to_owned())]
    );
}

#[tokio::test]
async fn native_assign_owner_step_creates_assignment_record() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "assign_owner".to_owned(),
                display_name: "Assign Owner".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::AssignOwner {
                    entity_logical_name: "lead".to_owned(),
                    record_id: "lead-1".to_owned(),
                    owner_id: "triage_queue".to_owned(),
                    reason: Some("auto routing".to_owned()),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let executed = service
        .execute_workflow(&actor, "assign_owner", json!({}))
        .await;
    assert!(executed.is_ok());

    let created = runtime_service.created_records.lock().await.clone();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].0, "record_assignment");
    assert_eq!(created[0].1["owner_id"], json!("triage_queue"));
    assert_eq!(created[0].1["source_entity"], json!("lead"));
}

#[tokio::test]
async fn native_approval_request_step_creates_approval_record() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "approval_request".to_owned(),
                display_name: "Approval Request".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::ApprovalRequest {
                    entity_logical_name: "quote".to_owned(),
                    record_id: "quote-2".to_owned(),
                    request_type: "discount_override".to_owned(),
                    requested_by: None,
                    approver_id: Some("manager-7".to_owned()),
                    reason: Some("requires manager approval".to_owned()),
                    payload: Some(json!({"discount": 20})),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let executed = service
        .execute_workflow(&actor, "approval_request", json!({}))
        .await;
    assert!(executed.is_ok());

    let created = runtime_service.created_records.lock().await.clone();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].0, "approval_request");
    assert_eq!(created[0].1["status"], json!("pending"));
    assert_eq!(created[0].1["approver_id"], json!("manager-7"));
    assert_eq!(created[0].1["requested_by"], json!("workflow-runtime"));
}

#[tokio::test]
async fn native_delay_step_executes_successfully() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "delay_native".to_owned(),
                display_name: "Delay Native".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::Delay {
                    duration_ms: 1,
                    reason: Some("wait for consistency".to_owned()),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let run = service
        .execute_workflow(&actor, "delay_native", json!({}))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::Succeeded);
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::Condition {
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
                }],
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "{{trigger.payload.target_entity}}".to_owned(),
                    data: json!({
                        "title": "Record {{trigger.payload.record_id}}",
                        "record_id": "{{trigger.payload.record_id}}",
                        "run_id": "{{run.id}}",
                        "attempt": "{{run.attempt}}",
                        "owner": "{{trigger.payload.owner}}",
                    }),
                }],
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::LogMessage {
                    message: "queued".to_owned(),
                }],
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
        .claim_jobs_for_worker("worker-alpha", 10, 30, None, None)
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
async fn queued_runtime_event_flow_covers_outbox_job_execution_and_replay_history() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());
    runtime_service
        .queued_events
        .lock()
        .await
        .push(ClaimedRuntimeRecordWorkflowEvent {
            event_id: "event-e2e-1".to_owned(),
            tenant_id,
            trigger: WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: "contact".to_owned(),
            },
            record_id: "record-e2e-1".to_owned(),
            payload: json!({
                "entity_logical_name": "contact",
                "record_id": "record-e2e-1",
                "id": "record-e2e-1",
                "name": "Alice",
                "status": "new",
                "record": {"name": "Alice", "status": "new"},
                "data": {"name": "Alice", "status": "new"},
                "event": "created"
            }),
            emitted_by_subject: "maker".to_owned(),
            lease_token: "lease-e2e-1".to_owned(),
        });
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service.clone(),
        WorkflowExecutionMode::Queued,
        Some(action_dispatcher.clone()),
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "contact_created_http_e2e".to_owned(),
                display_name: "Contact Created HTTP E2E".to_owned(),
                description: None,
                trigger: WorkflowTrigger::RuntimeRecordCreated {
                    entity_logical_name: "contact".to_owned(),
                },
                steps: vec![WorkflowStep::HttpRequest {
                    method: "POST".to_owned(),
                    url: "https://example.org/contact-created".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    body: Some(json!({
                        "record_id": "{{trigger.payload.record_id}}",
                        "name": "{{trigger.payload.name}}",
                        "status": "{{trigger.payload.status}}"
                    })),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let drain_result = service
        .drain_runtime_record_workflow_events_for_worker("worker-alpha", 10, 30, Some(tenant_id))
        .await;
    assert!(drain_result.is_ok());
    let drain_result = drain_result.unwrap_or_else(|_| unreachable!());
    assert_eq!(drain_result.claimed_events, 1);
    assert_eq!(drain_result.dispatched_workflows, 1);
    assert_eq!(drain_result.released_events, 0);
    assert_eq!(
        runtime_service.completed_event_ids.lock().await.as_slice(),
        ["event-e2e-1"]
    );

    let claimed_jobs = service
        .claim_jobs_for_worker("worker-beta", 10, 30, None, Some(tenant_id))
        .await;
    assert!(claimed_jobs.is_ok());
    let mut claimed_jobs = claimed_jobs.unwrap_or_default();
    assert_eq!(claimed_jobs.len(), 1);

    let run = service
        .execute_claimed_job("worker-beta", claimed_jobs.remove(0))
        .await;
    assert!(run.is_ok());
    let run = run.unwrap_or_else(|_| unreachable!());
    assert_eq!(run.status, WorkflowRunStatus::Succeeded);
    assert_eq!(run.attempts, 1);
    assert_eq!(run.trigger_payload["record_id"], json!("record-e2e-1"));

    let dispatched_requests = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched_requests.len(), 1);
    assert_eq!(
        dispatched_requests[0].dispatch_type,
        WorkflowActionDispatchType::HttpRequest
    );
    assert_eq!(
        dispatched_requests[0].payload["body"]["record_id"],
        json!("record-e2e-1")
    );
    assert_eq!(
        dispatched_requests[0].payload["body"]["name"],
        json!("Alice")
    );
    assert_eq!(
        dispatched_requests[0].payload["body"]["status"],
        json!("new")
    );

    let attempts = service
        .list_run_attempts(&actor, run.run_id.as_str())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].status, WorkflowRunAttemptStatus::Succeeded);
    assert_eq!(attempts[0].step_traces.len(), 1);
    assert_eq!(attempts[0].step_traces[0].step_path, "0");

    let replay = service
        .replay_run(&actor, "contact_created_http_e2e", run.run_id.as_str())
        .await;
    assert!(replay.is_ok());
    let replay = replay.unwrap_or_else(|_| unreachable!());
    assert_eq!(replay.run.run_id, run.run_id);
    assert_eq!(replay.attempts.len(), 1);
    assert_eq!(replay.timeline.len(), 1);
    assert_eq!(replay.timeline[0].step_path, "0");
    assert_eq!(replay.timeline[0].attempt_status.as_str(), "succeeded");
}

#[tokio::test]
async fn queued_mode_claims_can_be_filtered_to_one_tenant() {
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();
    let left_actor = UserIdentity::new("left-maker", "left-maker", None, left_tenant);
    let right_actor = UserIdentity::new("right-maker", "right-maker", None, right_tenant);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([
            (
                (left_tenant, "left-maker".to_owned()),
                vec![Permission::WorkflowManage, Permission::WorkflowRead],
            ),
            (
                (right_tenant, "right-maker".to_owned()),
                vec![Permission::WorkflowManage, Permission::WorkflowRead],
            ),
        ]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Queued,
        None,
    );

    let left_saved = service
        .save_workflow(
            &left_actor,
            SaveWorkflowInput {
                logical_name: "queued_contact_create".to_owned(),
                display_name: "Queued Contact Create".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::LogMessage {
                    message: "queued".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(left_saved.is_ok());

    let right_saved = service
        .save_workflow(
            &right_actor,
            SaveWorkflowInput {
                logical_name: "queued_contact_create".to_owned(),
                display_name: "Queued Contact Create".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::LogMessage {
                    message: "queued".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(right_saved.is_ok());

    let left_enqueued = service
        .execute_workflow(
            &left_actor,
            "queued_contact_create",
            json!({"source": "left"}),
        )
        .await;
    assert!(left_enqueued.is_ok());

    let right_enqueued = service
        .execute_workflow(
            &right_actor,
            "queued_contact_create",
            json!({"source": "right"}),
        )
        .await;
    assert!(right_enqueued.is_ok());

    let claimed_jobs = service
        .claim_jobs_for_worker("worker-alpha", 10, 30, None, Some(left_tenant))
        .await;
    assert!(claimed_jobs.is_ok());
    let claimed_jobs = claimed_jobs.unwrap_or_default();
    assert_eq!(claimed_jobs.len(), 1);
    assert_eq!(claimed_jobs[0].tenant_id, left_tenant);
    assert_eq!(claimed_jobs[0].trigger_payload["source"], json!("left"));
}

#[tokio::test]
async fn queued_mode_does_not_double_claim_same_job_while_lease_is_active() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                logical_name: "queued_contact_create_single_claim".to_owned(),
                display_name: "Queued Contact Create Single Claim".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::LogMessage {
                    message: "queued".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let enqueued_run = service
        .execute_workflow(
            &actor,
            "queued_contact_create_single_claim",
            json!({"source": "test"}),
        )
        .await;
    assert!(enqueued_run.is_ok());

    let first_claim = service
        .claim_jobs_for_worker("worker-alpha", 10, 30, None, None)
        .await;
    assert!(first_claim.is_ok());
    let first_claim = first_claim.unwrap_or_default();
    assert_eq!(first_claim.len(), 1);

    let second_claim = service
        .claim_jobs_for_worker("worker-beta", 10, 30, None, None)
        .await;
    assert!(second_claim.is_ok());
    let second_claim = second_claim.unwrap_or_default();
    assert!(second_claim.is_empty());
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
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
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
                steps: vec![WorkflowStep::LogMessage {
                    message: "queued".to_owned(),
                }],
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
        .claim_jobs_for_worker("worker-alpha", 10, 30, None, None)
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

#[tokio::test]
async fn queued_mode_rejects_claimed_job_with_stale_lease_token() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service,
        WorkflowExecutionMode::Queued,
        None,
    );

    let save_result = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "queued_contact_create_stale_token".to_owned(),
                display_name: "Queued Contact Create Stale Token".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::LogMessage {
                    message: "queued".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await;
    assert!(save_result.is_ok());

    let enqueued_run = service
        .execute_workflow(
            &actor,
            "queued_contact_create_stale_token",
            json!({"source": "test"}),
        )
        .await;
    assert!(enqueued_run.is_ok());

    let claimed_jobs = service
        .claim_jobs_for_worker("worker-alpha", 10, 30, None, None)
        .await;
    assert!(claimed_jobs.is_ok());
    let mut claimed_jobs = claimed_jobs.unwrap_or_default();
    assert_eq!(claimed_jobs.len(), 1);

    let mut claimed_job = claimed_jobs.remove(0);
    claimed_job.lease_token = "stale-lease-token".to_owned();

    let completed = service
        .execute_claimed_job("worker-alpha", claimed_job)
        .await;
    assert!(completed.is_err());

    let jobs = repository.jobs.lock().await.clone();
    assert_eq!(jobs.len(), 1);
    assert!(!jobs[0].completed);
    assert!(!jobs[0].failed);
    assert_eq!(jobs[0].leased_by.as_deref(), Some("worker-alpha"));
}

#[tokio::test]
async fn drain_runtime_record_workflow_events_dispatches_matching_workflows() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    runtime_service
        .queued_events
        .lock()
        .await
        .push(ClaimedRuntimeRecordWorkflowEvent {
            event_id: "event-1".to_owned(),
            tenant_id,
            trigger: WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: "contact".to_owned(),
            },
            record_id: "record-1".to_owned(),
            payload: json!({
                "entity_logical_name": "contact",
                "record_id": "record-1",
                "id": "record-1",
                "record": {"name": "Alice"},
                "data": {"name": "Alice"},
                "event": "created"
            }),
            emitted_by_subject: "maker".to_owned(),
            lease_token: "lease-1".to_owned(),
        });
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "contact_created_log".to_owned(),
                display_name: "Contact Created Log".to_owned(),
                description: None,
                trigger: WorkflowTrigger::RuntimeRecordCreated {
                    entity_logical_name: "contact".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "created".to_owned(),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let result = service
        .drain_runtime_record_workflow_events_for_worker("worker-alpha", 10, 30, Some(tenant_id))
        .await;
    assert!(result.is_ok());
    let result = result.unwrap_or_else(|_| unreachable!());
    assert_eq!(result.claimed_events, 1);
    assert_eq!(result.dispatched_workflows, 1);
    assert_eq!(result.released_events, 0);
    assert_eq!(
        runtime_service.completed_event_ids.lock().await.as_slice(),
        ["event-1"]
    );
    assert!(runtime_service.released_event_ids.lock().await.is_empty());
    assert_eq!(repository.runs.lock().await.len(), 1);
}

#[tokio::test]
async fn drain_runtime_record_workflow_events_completes_after_workflow_dead_letters() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    runtime_service
        .queued_events
        .lock()
        .await
        .push(ClaimedRuntimeRecordWorkflowEvent {
            event_id: "event-2".to_owned(),
            tenant_id,
            trigger: WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: "contact".to_owned(),
            },
            record_id: "record-2".to_owned(),
            payload: json!({
                "entity_logical_name": "contact",
                "record_id": "record-2",
                "id": "record-2",
                "record": {"name": "Alice"},
                "data": {"name": "Alice"},
                "event": "created"
            }),
            emitted_by_subject: "maker".to_owned(),
            lease_token: "lease-2".to_owned(),
        });
    *runtime_service.failures_remaining.lock().await = 1;
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "contact_created_create".to_owned(),
                display_name: "Contact Created Create".to_owned(),
                description: None,
                trigger: WorkflowTrigger::RuntimeRecordCreated {
                    entity_logical_name: "contact".to_owned(),
                },
                steps: vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({"name": "Follow Up"}),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let result = service
        .drain_runtime_record_workflow_events_for_worker("worker-alpha", 10, 30, Some(tenant_id))
        .await;
    assert!(result.is_ok());
    let result = result.unwrap_or_else(|_| unreachable!());
    assert_eq!(result.claimed_events, 1);
    assert_eq!(result.dispatched_workflows, 1);
    assert_eq!(result.released_events, 0);
    assert_eq!(
        runtime_service.completed_event_ids.lock().await.as_slice(),
        ["event-2"]
    );
    assert!(runtime_service.released_event_ids.lock().await.is_empty());
}

#[tokio::test]
async fn drain_runtime_record_workflow_events_releases_then_retries_transient_dispatch_failures() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    runtime_service
        .queued_events
        .lock()
        .await
        .push(ClaimedRuntimeRecordWorkflowEvent {
            event_id: "event-retry-1".to_owned(),
            tenant_id,
            trigger: WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: "contact".to_owned(),
            },
            record_id: "record-retry-1".to_owned(),
            payload: json!({
                "entity_logical_name": "contact",
                "record_id": "record-retry-1",
                "id": "record-retry-1",
                "record": {"name": "Alice"},
                "data": {"name": "Alice"},
                "event": "created"
            }),
            emitted_by_subject: "maker".to_owned(),
            lease_token: "lease-retry-1".to_owned(),
        });
    *repository
        .fail_list_enabled_workflows_remaining
        .lock()
        .await = 1;

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository.clone(),
        runtime_service.clone(),
        WorkflowExecutionMode::Inline,
        None,
    );

    let saved = service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "contact_created_retry_log".to_owned(),
                display_name: "Contact Created Retry Log".to_owned(),
                description: None,
                trigger: WorkflowTrigger::RuntimeRecordCreated {
                    entity_logical_name: "contact".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "created".to_owned(),
                }],
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await;
    assert!(saved.is_ok());

    let first = service
        .drain_runtime_record_workflow_events_for_worker("worker-alpha", 10, 30, Some(tenant_id))
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(first.claimed_events, 1);
    assert_eq!(first.dispatched_workflows, 0);
    assert_eq!(first.released_events, 1);
    assert!(runtime_service.completed_event_ids.lock().await.is_empty());
    assert_eq!(
        runtime_service.released_event_ids.lock().await.as_slice(),
        ["event-retry-1"]
    );
    assert_eq!(repository.runs.lock().await.len(), 0);
    assert_eq!(runtime_service.queued_events.lock().await.len(), 1);

    let second = service
        .drain_runtime_record_workflow_events_for_worker("worker-alpha", 10, 30, Some(tenant_id))
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(second.claimed_events, 1);
    assert_eq!(second.dispatched_workflows, 1);
    assert_eq!(second.released_events, 0);
    assert_eq!(
        runtime_service.completed_event_ids.lock().await.as_slice(),
        ["event-retry-1"]
    );
    assert_eq!(
        runtime_service.released_event_ids.lock().await.as_slice(),
        ["event-retry-1"]
    );
    assert_eq!(repository.runs.lock().await.len(), 1);
    assert!(runtime_service.queued_events.lock().await.is_empty());
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

#[tokio::test]
async fn draft_save_does_not_dispatch_until_workflow_is_published() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "lead_capture".to_owned(),
                display_name: "Lead Capture".to_owned(),
                description: None,
                trigger: WorkflowTrigger::WebhookReceived {
                    webhook_key: "leads".to_owned(),
                },
                steps: vec![WorkflowStep::LogMessage {
                    message: "captured".to_owned(),
                }],
                max_attempts: 2,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let before_publish = service
        .dispatch_webhook_received(tenant_id, "leads", json!({"lead_id": "lead-1"}))
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(before_publish, 0);

    service
        .publish_workflow(&actor, "lead_capture")
        .await
        .unwrap_or_else(|_| unreachable!());

    let after_publish = service
        .dispatch_webhook_received(tenant_id, "leads", json!({"lead_id": "lead-2"}))
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(after_publish, 1);
}

#[tokio::test]
async fn metadata_permissions_do_not_grant_workflow_access() {
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
                logical_name: "blocked_workflow".to_owned(),
                display_name: "Blocked Workflow".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::LogMessage {
                    message: "blocked".to_owned(),
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await;
    assert!(
        matches!(save_result, Err(AppError::Forbidden(message)) if message.contains("workflow.manage"))
    );

    let list_result = service.list_workflows(&actor).await;
    assert!(
        matches!(list_result, Err(AppError::Forbidden(message)) if message.contains("workflow.read"))
    );
}

#[tokio::test]
async fn workflow_permissions_allow_access_without_metadata_permissions() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "granted_workflow".to_owned(),
                display_name: "Granted Workflow".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::LogMessage {
                    message: "allowed".to_owned(),
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let workflows = service
        .list_workflows(&actor)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0].logical_name().as_str(), "granted_workflow");
}

#[tokio::test]
async fn workflow_publish_checks_report_unpublished_entity_dependencies() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService {
        assume_entities_published: false,
        ..Default::default()
    });
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "contact_flow".to_owned(),
                display_name: "Contact Flow".to_owned(),
                description: None,
                trigger: WorkflowTrigger::RuntimeRecordCreated {
                    entity_logical_name: "contact".to_owned(),
                },
                steps: vec![WorkflowStep::AssignOwner {
                    entity_logical_name: "contact".to_owned(),
                    record_id: "{{trigger.record_id}}".to_owned(),
                    owner_id: "owner-1".to_owned(),
                    reason: None,
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let errors = service
        .publish_checks(&actor, "contact_flow")
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("workflow 'contact_flow' -> entity 'contact'"));
}

#[tokio::test]
async fn workflow_publish_checks_allow_selected_unpublished_entities() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "account_followup".to_owned(),
                display_name: "Account Followup".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::CreateRuntimeRecord {
                    entity_logical_name: "account".to_owned(),
                    data: json!({"name": "Acme"}),
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let errors = service
        .publish_checks_with_allowed_unpublished_entities(
            &actor,
            "account_followup",
            &["account".to_owned()],
        )
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(errors.is_empty());
}

#[tokio::test]
async fn workflow_publish_checks_report_inline_credential_headers() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "external_sync".to_owned(),
                display_name: "External Sync".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::HttpRequest {
                    method: "POST".to_owned(),
                    url: "https://example.com/sync".to_owned(),
                    headers: Some(json!({
                        "authorization": "Bearer secret-value",
                        "content-type": "application/json",
                    })),
                    header_secret_refs: None,
                    body: None,
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let errors = service
        .publish_checks(&actor, "external_sync")
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("disallowed inline credential header 'authorization'"));
}

#[tokio::test]
async fn workflow_publish_rejects_inline_credential_headers() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "blocked_publish".to_owned(),
                display_name: "Blocked Publish".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::Webhook {
                    endpoint: "https://example.com/webhook".to_owned(),
                    event: "lead.created".to_owned(),
                    headers: Some(json!({
                        "x-api-key": "top-secret"
                    })),
                    header_secret_refs: None,
                    payload: json!({"lead_id": "lead-1"}),
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let publish_result = service.publish_workflow(&actor, "blocked_publish").await;
    assert!(
        matches!(publish_result, Err(AppError::Validation(message)) if message.contains("disallowed inline credential header 'x-api-key'"))
    );
}

#[tokio::test]
async fn workflow_publish_allows_sensitive_headers_when_backed_by_secret_refs() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "secret_backed_publish".to_owned(),
                display_name: "Secret Backed Publish".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::Webhook {
                    endpoint: "https://example.com/webhook".to_owned(),
                    event: "lead.created".to_owned(),
                    headers: Some(json!({
                        "content-type": "application/json"
                    })),
                    header_secret_refs: Some(json!({
                        "authorization": "op://vault/item/password"
                    })),
                    payload: json!({"lead_id": "lead-1"}),
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let errors = service
        .publish_checks(&actor, "secret_backed_publish")
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(errors.is_empty());
}

#[tokio::test]
async fn workflow_publish_step_up_detection_tracks_outbound_drafts() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "ops_notify".to_owned(),
                display_name: "Ops Notify".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::SendEmail {
                    to: "ops@example.com".to_owned(),
                    subject: "Alert".to_owned(),
                    body: "Check workflow".to_owned(),
                    html_body: None,
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(
        service
            .publish_requires_recent_step_up(&actor, "ops_notify")
            .await
            .unwrap_or_else(|_| unreachable!())
    );
}

#[tokio::test]
async fn workflow_disable_step_up_detection_tracks_active_outbound_versions() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    );

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "ops_webhook".to_owned(),
                display_name: "Ops Webhook".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::Webhook {
                    endpoint: "https://example.com/notify".to_owned(),
                    event: "incident.created".to_owned(),
                    headers: None,
                    header_secret_refs: None,
                    payload: json!({"severity": "high"}),
                }],
                max_attempts: 1,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());
    service
        .publish_workflow(&actor, "ops_webhook")
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(
        service
            .disable_requires_recent_step_up(&actor, "ops_webhook")
            .await
            .unwrap_or_else(|_| unreachable!())
    );
}

#[tokio::test]
async fn draft_changes_do_not_replace_current_published_workflow_until_publish() {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let repository = Arc::new(FakeWorkflowRepository::default());
    let runtime_service = Arc::new(FakeRuntimeRecordService::default());
    let action_dispatcher = Arc::new(FakeActionDispatcher::default());

    let service = build_service(
        HashMap::from([(
            (tenant_id, "maker".to_owned()),
            vec![Permission::WorkflowManage, Permission::WorkflowRead],
        )]),
        repository,
        runtime_service,
        WorkflowExecutionMode::Inline,
        None,
    )
    .with_action_dispatcher(action_dispatcher.clone());

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "versioned_email".to_owned(),
                display_name: "Versioned Email".to_owned(),
                description: None,
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::SendEmail {
                    to: "ops@example.com".to_owned(),
                    subject: "v1".to_owned(),
                    body: "first".to_owned(),
                    html_body: None,
                }],
                max_attempts: 2,
                is_enabled: true,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .save_workflow(
            &actor,
            SaveWorkflowInput {
                logical_name: "versioned_email".to_owned(),
                display_name: "Versioned Email".to_owned(),
                description: Some("draft update".to_owned()),
                trigger: WorkflowTrigger::Manual,
                steps: vec![WorkflowStep::SendEmail {
                    to: "ops@example.com".to_owned(),
                    subject: "v2".to_owned(),
                    body: "second".to_owned(),
                    html_body: None,
                }],
                max_attempts: 2,
                is_enabled: false,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .execute_workflow(&actor, "versioned_email", json!({}))
        .await
        .unwrap_or_else(|_| unreachable!());

    let dispatched = action_dispatcher.dispatched_requests.lock().await.clone();
    assert_eq!(dispatched.len(), 1);
    assert_eq!(dispatched[0].payload["subject"], json!("v1"));
}
