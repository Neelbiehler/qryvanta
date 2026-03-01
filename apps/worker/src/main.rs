//! Qryvanta workflow worker runtime.

#![forbid(unsafe_code)]

use std::env;
use std::sync::Arc;
use std::time::Duration;

use qryvanta_application::{
    AuthorizationService, EmailService, MetadataService, WorkflowClaimPartition,
    WorkflowExecutionMode, WorkflowService, WorkflowWorkerLease, WorkflowWorkerLeaseCoordinator,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    WorkflowAction, WorkflowDefinition, WorkflowDefinitionInput, WorkflowStep, WorkflowTrigger,
};
use qryvanta_infrastructure::{
    ConsoleEmailService, HttpWorkflowActionDispatcher, PostgresAuditRepository,
    PostgresAuthorizationRepository, PostgresMetadataRepository, PostgresWorkflowRepository,
    RedisWorkflowWorkerLeaseCoordinator, SmtpEmailConfig, SmtpEmailService,
};

use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
struct WorkerConfig {
    database_url: String,
    api_base_url: String,
    worker_shared_secret: String,
    worker_id: String,
    redis_url: Option<String>,
    coordination_backend: WorkerCoordinationBackend,
    coordination_lease_seconds: u32,
    coordination_scope_key: String,
    lease_loss_strategy: WorkerLeaseLossStrategy,
    claim_limit: usize,
    max_concurrency: usize,
    lease_seconds: u32,
    poll_interval_ms: u64,
    partition: Option<WorkflowClaimPartition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerCoordinationBackend {
    None,
    Redis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerLeaseLossStrategy {
    AbortAll,
    GracefulDrain,
}

#[derive(Debug, Serialize)]
struct ClaimWorkflowJobsRequest {
    limit: usize,
    lease_seconds: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition_index: Option<u32>,
}

#[derive(Debug, Serialize)]
struct WorkerHeartbeatRequest {
    claimed_jobs: u32,
    executed_jobs: u32,
    failed_jobs: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition_index: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ClaimedWorkflowJobsResponse {
    jobs: Vec<ClaimedWorkflowJobResponse>,
}

#[derive(Debug, Deserialize)]
struct ClaimedWorkflowJobResponse {
    job_id: String,
    lease_token: String,
    tenant_id: String,
    run_id: String,
    workflow_logical_name: String,
    workflow_display_name: String,
    workflow_description: Option<String>,
    workflow_trigger: WorkflowTrigger,
    workflow_action: WorkflowAction,
    workflow_steps: Option<Vec<WorkflowStep>>,
    workflow_max_attempts: u16,
    workflow_is_enabled: bool,
    trigger_payload: Value,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = WorkerConfig::load()?;
    let pool = connect_pool(config.database_url.as_str()).await?;
    let workflow_service = build_workflow_service(pool);
    let lease_coordinator = build_lease_coordinator(&config)?;
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| AppError::Internal(format!("failed to build HTTP client: {error}")))?;

    info!(
        worker_id = %config.worker_id,
        api_base_url = %config.api_base_url,
        coordination_backend = %config.coordination_backend,
        coordination_scope_key = %config.coordination_scope_key,
        coordination_lease_seconds = config.coordination_lease_seconds,
        lease_loss_strategy = %config.lease_loss_strategy,
        claim_limit = config.claim_limit,
        max_concurrency = config.max_concurrency,
        lease_seconds = config.lease_seconds,
        poll_interval_ms = config.poll_interval_ms,
        partition_count = config.partition.map(|value| value.partition_count()),
        partition_index = config.partition.map(|value| value.partition_index()),
        "qryvanta-worker started"
    );

    loop {
        let lease = match &lease_coordinator {
            Some(coordinator) => match coordinator
                .try_acquire_lease(
                    config.coordination_scope_key.as_str(),
                    config.worker_id.as_str(),
                    config.coordination_lease_seconds,
                )
                .await
            {
                Ok(Some(lease)) => Some(lease),
                Ok(None) => {
                    info!(
                        worker_id = %config.worker_id,
                        scope_key = %config.coordination_scope_key,
                        "worker lease not acquired; another worker currently owns scope"
                    );
                    tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
                    continue;
                }
                Err(error) => {
                    warn!(
                        worker_id = %config.worker_id,
                        error = %error,
                        "failed to acquire worker coordination lease"
                    );
                    tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
                    continue;
                }
            },
            None => None,
        };

        let (renewal_stop_tx, renewal_task, cycle_cancel_rx) =
            if let (Some(coordinator), Some(lease)) = (&lease_coordinator, &lease) {
                let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
                let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
                let task = tokio::spawn(run_lease_renewal_loop(
                    coordinator.clone(),
                    lease.clone(),
                    config.coordination_lease_seconds,
                    config.worker_id.clone(),
                    stop_rx,
                    cancel_tx,
                ));
                (Some(stop_tx), Some(task), Some(cancel_rx))
            } else {
                (None, None, None)
            };

        let mut cycle_result = run_worker_cycle(
            &http_client,
            workflow_service.clone(),
            &config,
            cycle_cancel_rx,
        )
        .await;

        if let Some(stop_tx) = renewal_stop_tx {
            let _ = stop_tx.send(true);
        }

        if let Some(task) = renewal_task {
            match task.await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    if cycle_result.is_ok() {
                        cycle_result = Err(error);
                    } else {
                        warn!(
                            worker_id = %config.worker_id,
                            error = %error,
                            "worker coordination renewal failed after cycle error"
                        );
                    }
                }
                Err(error) => {
                    warn!(
                        worker_id = %config.worker_id,
                        error = %error,
                        "worker coordination renewal task join failed"
                    );
                }
            }
        }

        if let (Some(coordinator), Some(lease)) = (&lease_coordinator, &lease)
            && let Err(error) = coordinator.release_lease(lease).await
        {
            warn!(
                worker_id = %config.worker_id,
                error = %error,
                "failed to release worker coordination lease"
            );
        }

        if let Err(error) = cycle_result {
            warn!(
                worker_id = %config.worker_id,
                error = %error,
                "failed to claim workflow jobs"
            );
            tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
        }
    }
}

async fn run_worker_cycle(
    http_client: &reqwest::Client,
    workflow_service: WorkflowService,
    config: &WorkerConfig,
    cancel_signal: Option<tokio::sync::watch::Receiver<bool>>,
) -> AppResult<()> {
    let claimed_jobs = claim_jobs(http_client, config).await?;
    let claimed_job_count = u32::try_from(claimed_jobs.len()).unwrap_or(u32::MAX);

    if claimed_jobs.is_empty() {
        if let Err(error) = send_heartbeat(http_client, config, 0, 0, 0).await {
            warn!(
                worker_id = %config.worker_id,
                error = %error,
                "failed to publish worker heartbeat"
            );
        }
        tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
        return Ok(());
    }

    info!(
        worker_id = %config.worker_id,
        claimed_count = claimed_jobs.len(),
        "claimed workflow jobs"
    );

    let execution_totals = execute_claimed_jobs(
        workflow_service,
        config.worker_id.as_str(),
        claimed_jobs,
        config.max_concurrency,
        config.lease_loss_strategy,
        cancel_signal,
    )
    .await;
    let executed_jobs = execution_totals.executed_jobs;
    let failed_jobs = execution_totals.failed_jobs;

    if let Err(error) = send_heartbeat(
        http_client,
        config,
        claimed_job_count,
        executed_jobs,
        failed_jobs,
    )
    .await
    {
        warn!(
            worker_id = %config.worker_id,
            error = %error,
            "failed to publish worker heartbeat"
        );
    }

    if execution_totals.cancelled_due_to_lease_loss {
        return Err(AppError::Conflict(
            "worker coordination lease lost during execution; cancelled in-flight claimed jobs"
                .to_owned(),
        ));
    }

    Ok(())
}

fn build_lease_coordinator(
    config: &WorkerConfig,
) -> AppResult<Option<Arc<dyn WorkflowWorkerLeaseCoordinator>>> {
    match config.coordination_backend {
        WorkerCoordinationBackend::None => Ok(None),
        WorkerCoordinationBackend::Redis => {
            let redis_url = config.redis_url.as_deref().ok_or_else(|| {
                AppError::Validation(
                    "REDIS_URL is required when WORKER_COORDINATION_BACKEND=redis".to_owned(),
                )
            })?;

            let redis_client = redis::Client::open(redis_url)
                .map_err(|error| AppError::Validation(format!("invalid REDIS_URL: {error}")))?;

            Ok(Some(Arc::new(RedisWorkflowWorkerLeaseCoordinator::new(
                redis_client,
                "qryvanta:workflow_worker_lease",
            ))))
        }
    }
}

async fn run_lease_renewal_loop(
    coordinator: Arc<dyn WorkflowWorkerLeaseCoordinator>,
    lease: WorkflowWorkerLease,
    lease_seconds: u32,
    worker_id: String,
    mut stop_rx: tokio::sync::watch::Receiver<bool>,
    cancel_tx: tokio::sync::watch::Sender<bool>,
) -> AppResult<()> {
    let renew_interval =
        Duration::from_secs(u64::from(lease_renew_interval_seconds(lease_seconds)));

    loop {
        tokio::select! {
            changed = stop_rx.changed() => {
                if changed.is_err() || *stop_rx.borrow() {
                    return Ok(());
                }
            }
            _ = tokio::time::sleep(renew_interval) => {
                match coordinator.renew_lease(&lease, lease_seconds).await {
                    Ok(true) => {}
                    Ok(false) => {
                        let _ = cancel_tx.send(true);
                        return Err(AppError::Conflict(format!(
                            "worker coordination lease ownership lost for scope '{}' and worker '{}'",
                            lease.scope_key,
                            worker_id
                        )));
                    }
                    Err(error) => {
                        let _ = cancel_tx.send(true);
                        return Err(AppError::Internal(format!(
                            "failed to renew worker coordination lease for scope '{}' and worker '{}': {error}",
                            lease.scope_key,
                            worker_id
                        )));
                    }
                }
            }
        }
    }
}

fn lease_renew_interval_seconds(lease_seconds: u32) -> u32 {
    (lease_seconds / 3).max(1)
}

async fn connect_pool(database_url: &str) -> AppResult<PgPool> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|error| AppError::Internal(format!("failed to connect to database: {error}")))
}

fn build_workflow_service(pool: PgPool) -> WorkflowService {
    let metadata_repository = Arc::new(PostgresMetadataRepository::new(pool.clone()));
    let workflow_repository = Arc::new(PostgresWorkflowRepository::new(pool.clone()));
    let authorization_repository = Arc::new(PostgresAuthorizationRepository::new(pool.clone()));
    let audit_repository = Arc::new(PostgresAuditRepository::new(pool));
    let authorization_service =
        AuthorizationService::new(authorization_repository, audit_repository.clone());
    let runtime_record_service = Arc::new(MetadataService::new(
        metadata_repository,
        authorization_service.clone(),
        audit_repository.clone(),
    ));
    let workflow_email_service = build_worker_email_service();
    let workflow_action_dispatcher = Arc::new(HttpWorkflowActionDispatcher::new(
        reqwest::Client::new(),
        workflow_email_service,
        3,
        250,
    ));

    WorkflowService::new(
        authorization_service,
        workflow_repository,
        runtime_record_service,
        audit_repository,
        WorkflowExecutionMode::Queued,
    )
    .with_action_dispatcher(workflow_action_dispatcher)
}

fn build_worker_email_service() -> Arc<dyn EmailService> {
    let provider = env::var("EMAIL_PROVIDER")
        .unwrap_or_else(|_| "console".to_owned())
        .to_lowercase();

    if provider == "smtp" {
        let host = env::var("SMTP_HOST").ok();
        let port = env::var("SMTP_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok());
        let username = env::var("SMTP_USERNAME").ok();
        let password = env::var("SMTP_PASSWORD").ok();
        let from_address = env::var("SMTP_FROM_ADDRESS").ok();

        if let (Some(host), Some(port), Some(username), Some(password), Some(from_address)) =
            (host, port, username, password, from_address)
        {
            let config = SmtpEmailConfig {
                host,
                port,
                username,
                password,
                from_address,
            };

            match SmtpEmailService::new(config) {
                Ok(service) => return Arc::new(service),
                Err(error) => {
                    warn!(
                        error = %error,
                        "failed to initialize SMTP email service for worker; falling back to console"
                    );
                }
            }
        } else {
            warn!(
                "EMAIL_PROVIDER=smtp but SMTP_* environment variables are incomplete; falling back to console"
            );
        }
    }

    Arc::new(ConsoleEmailService::new())
}

async fn claim_jobs(
    http_client: &reqwest::Client,
    config: &WorkerConfig,
) -> AppResult<Vec<ClaimedWorkflowJobResponse>> {
    let endpoint = format!("{}/api/internal/worker/jobs/claim", config.api_base_url);
    let response = http_client
        .post(endpoint)
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", config.worker_shared_secret),
        )
        .header("x-qryvanta-worker-id", config.worker_id.as_str())
        .json(&ClaimWorkflowJobsRequest {
            limit: config.claim_limit,
            lease_seconds: config.lease_seconds,
            partition_count: config.partition.map(|value| value.partition_count()),
            partition_index: config.partition.map(|value| value.partition_index()),
        })
        .send()
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to call worker claim endpoint: {error}"))
        })?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<body unavailable>".to_owned());
        return Err(AppError::Internal(format!(
            "worker claim endpoint returned status {}: {body}",
            status.as_u16()
        )));
    }

    let response_body = response
        .json::<ClaimedWorkflowJobsResponse>()
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to parse worker claim endpoint response body: {error}"
            ))
        })?;

    Ok(response_body.jobs)
}

async fn send_heartbeat(
    http_client: &reqwest::Client,
    config: &WorkerConfig,
    claimed_jobs: u32,
    executed_jobs: u32,
    failed_jobs: u32,
) -> AppResult<()> {
    let endpoint = format!("{}/api/internal/worker/heartbeat", config.api_base_url);
    let response = http_client
        .post(endpoint)
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", config.worker_shared_secret),
        )
        .header("x-qryvanta-worker-id", config.worker_id.as_str())
        .json(&WorkerHeartbeatRequest {
            claimed_jobs,
            executed_jobs,
            failed_jobs,
            partition_count: config.partition.map(|value| value.partition_count()),
            partition_index: config.partition.map(|value| value.partition_index()),
        })
        .send()
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to call worker heartbeat endpoint: {error}"))
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<body unavailable>".to_owned());
        return Err(AppError::Internal(format!(
            "worker heartbeat endpoint returned status {}: {body}",
            status.as_u16()
        )));
    }

    Ok(())
}

impl ClaimedWorkflowJobResponse {
    fn try_into_claimed_job(self) -> AppResult<qryvanta_application::ClaimedWorkflowJob> {
        let tenant_uuid = uuid::Uuid::parse_str(self.tenant_id.as_str()).map_err(|error| {
            AppError::Validation(format!(
                "invalid tenant id '{}' from worker claim response: {error}",
                self.tenant_id
            ))
        })?;

        let workflow = WorkflowDefinition::new(WorkflowDefinitionInput {
            logical_name: self.workflow_logical_name,
            display_name: self.workflow_display_name,
            description: self.workflow_description,
            trigger: self.workflow_trigger,
            action: self.workflow_action,
            steps: self.workflow_steps,
            max_attempts: self.workflow_max_attempts,
            is_enabled: self.workflow_is_enabled,
        })?;

        Ok(qryvanta_application::ClaimedWorkflowJob {
            job_id: self.job_id,
            tenant_id: TenantId::from_uuid(tenant_uuid),
            run_id: self.run_id,
            workflow,
            trigger_payload: self.trigger_payload,
            lease_token: self.lease_token,
        })
    }
}

impl WorkerConfig {
    fn load() -> AppResult<Self> {
        let database_url = required_env("DATABASE_URL")?;
        let api_base_url = env::var("WORKER_API_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3001".to_owned())
            .trim_end_matches('/')
            .to_owned();
        let worker_shared_secret = required_env("WORKER_SHARED_SECRET")?;
        let worker_id = env::var("WORKER_ID")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("worker-{}", std::process::id()));
        let redis_url = env::var("REDIS_URL")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        let coordination_backend = WorkerCoordinationBackend::parse(
            env::var("WORKER_COORDINATION_BACKEND")
                .unwrap_or_else(|_| "none".to_owned())
                .as_str(),
        )?;
        let lease_loss_strategy = WorkerLeaseLossStrategy::parse(
            env::var("WORKER_LEASE_LOSS_STRATEGY")
                .unwrap_or_else(|_| "graceful_drain".to_owned())
                .as_str(),
        )?;
        let coordination_lease_seconds = parse_env_u32("WORKER_COORDINATION_LEASE_SECONDS", 120)?;
        let claim_limit = parse_env_usize("WORKER_CLAIM_LIMIT", 10)?;
        let max_concurrency = parse_env_usize("WORKER_MAX_CONCURRENCY", 4)?;
        let lease_seconds = parse_env_u32("WORKER_LEASE_SECONDS", 30)?;
        let poll_interval_ms = parse_env_u64("WORKER_POLL_INTERVAL_MS", 1500)?;
        let partition_count = parse_optional_env_u32("WORKER_PARTITION_COUNT")?;
        let partition_index = parse_optional_env_u32("WORKER_PARTITION_INDEX")?;

        if claim_limit == 0 {
            return Err(AppError::Validation(
                "WORKER_CLAIM_LIMIT must be greater than zero".to_owned(),
            ));
        }

        if max_concurrency == 0 {
            return Err(AppError::Validation(
                "WORKER_MAX_CONCURRENCY must be greater than zero".to_owned(),
            ));
        }

        if lease_seconds == 0 {
            return Err(AppError::Validation(
                "WORKER_LEASE_SECONDS must be greater than zero".to_owned(),
            ));
        }

        if coordination_lease_seconds == 0 {
            return Err(AppError::Validation(
                "WORKER_COORDINATION_LEASE_SECONDS must be greater than zero".to_owned(),
            ));
        }

        if poll_interval_ms == 0 {
            return Err(AppError::Validation(
                "WORKER_POLL_INTERVAL_MS must be greater than zero".to_owned(),
            ));
        }

        let partition = match (partition_count, partition_index) {
            (None, None) => None,
            (Some(count), Some(index)) => Some(WorkflowClaimPartition::new(count, index)?),
            _ => {
                return Err(AppError::Validation(
                    "WORKER_PARTITION_COUNT and WORKER_PARTITION_INDEX must be provided together"
                        .to_owned(),
                ));
            }
        };

        if matches!(coordination_backend, WorkerCoordinationBackend::Redis) && redis_url.is_none() {
            return Err(AppError::Validation(
                "REDIS_URL is required when WORKER_COORDINATION_BACKEND=redis".to_owned(),
            ));
        }

        let coordination_scope_key = env::var("WORKER_COORDINATION_SCOPE_KEY")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| default_coordination_scope_key(worker_id.as_str(), partition));

        Ok(Self {
            database_url,
            api_base_url,
            worker_shared_secret,
            worker_id,
            redis_url,
            coordination_backend,
            coordination_lease_seconds,
            coordination_scope_key,
            lease_loss_strategy,
            claim_limit,
            max_concurrency,
            lease_seconds,
            poll_interval_ms,
            partition,
        })
    }
}

impl WorkerCoordinationBackend {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Redis => "redis",
        }
    }

    fn parse(value: &str) -> AppResult<Self> {
        if value.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }

        if value.eq_ignore_ascii_case("redis") {
            return Ok(Self::Redis);
        }

        Err(AppError::Validation(format!(
            "WORKER_COORDINATION_BACKEND must be either 'none' or 'redis', got '{value}'"
        )))
    }
}

impl std::fmt::Display for WorkerCoordinationBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl WorkerLeaseLossStrategy {
    fn as_str(self) -> &'static str {
        match self {
            Self::AbortAll => "abort_all",
            Self::GracefulDrain => "graceful_drain",
        }
    }

    fn parse(value: &str) -> AppResult<Self> {
        if value.eq_ignore_ascii_case("abort_all") {
            return Ok(Self::AbortAll);
        }

        if value.eq_ignore_ascii_case("graceful_drain") {
            return Ok(Self::GracefulDrain);
        }

        Err(AppError::Validation(format!(
            "WORKER_LEASE_LOSS_STRATEGY must be either 'abort_all' or 'graceful_drain', got '{value}'"
        )))
    }
}

impl std::fmt::Display for WorkerLeaseLossStrategy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn default_coordination_scope_key(
    worker_id: &str,
    partition: Option<WorkflowClaimPartition>,
) -> String {
    match partition {
        Some(value) => format!(
            "partition:{}:{}",
            value.partition_count(),
            value.partition_index()
        ),
        None => format!("worker:{worker_id}"),
    }
}

fn workflow_has_mutating_effects(workflow: &WorkflowDefinition) -> bool {
    if action_is_mutating(workflow.action()) {
        return true;
    }

    workflow
        .steps()
        .is_some_and(|steps| steps.iter().any(step_is_mutating))
}

fn action_is_mutating(action: &WorkflowAction) -> bool {
    matches!(action, WorkflowAction::CreateRuntimeRecord { .. })
}

fn step_is_mutating(step: &WorkflowStep) -> bool {
    match step {
        WorkflowStep::LogMessage { .. } => false,
        WorkflowStep::CreateRuntimeRecord { .. } => true,
        WorkflowStep::Condition {
            then_steps,
            else_steps,
            ..
        } => then_steps.iter().any(step_is_mutating) || else_steps.iter().any(step_is_mutating),
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct JobExecutionTotals {
    executed_jobs: u32,
    failed_jobs: u32,
    cancelled_due_to_lease_loss: bool,
}

type WorkerExecutionTaskResult = (
    String,
    String,
    String,
    AppResult<qryvanta_application::WorkflowRun>,
);

async fn execute_claimed_jobs(
    workflow_service: WorkflowService,
    worker_id: &str,
    claimed_jobs: Vec<ClaimedWorkflowJobResponse>,
    max_concurrency: usize,
    lease_loss_strategy: WorkerLeaseLossStrategy,
    mut cancel_signal: Option<tokio::sync::watch::Receiver<bool>>,
) -> JobExecutionTotals {
    let mut in_flight = tokio::task::JoinSet::new();
    let mut remaining_jobs = claimed_jobs.into_iter();
    let mut mutating_abort_handles: Vec<tokio::task::AbortHandle> = Vec::new();
    let worker_id = worker_id.to_owned();
    let max_concurrency = max_concurrency.max(1);
    let mut totals = JobExecutionTotals::default();
    let mut lease_loss_detected = false;

    loop {
        while !lease_loss_detected && in_flight.len() < max_concurrency {
            let Some(claimed_job) = remaining_jobs.next() else {
                break;
            };

            let queued_job = match claimed_job.try_into_claimed_job() {
                Ok(job) => job,
                Err(error) => {
                    totals.failed_jobs = totals.failed_jobs.saturating_add(1);
                    warn!(
                        worker_id = %worker_id,
                        error = %error,
                        "failed to parse claimed workflow job payload"
                    );
                    continue;
                }
            };

            let workflow_service = workflow_service.clone();
            let worker_id = worker_id.clone();
            let is_mutating = workflow_has_mutating_effects(&queued_job.workflow);
            let job_id = queued_job.job_id.clone();
            let run_id = queued_job.run_id.clone();
            let abort_handle = in_flight.spawn(async move {
                let result = workflow_service
                    .execute_claimed_job(worker_id.as_str(), queued_job)
                    .await;
                (worker_id, job_id, run_id, result)
            });

            if is_mutating {
                mutating_abort_handles.push(abort_handle);
            }
        }

        if lease_loss_detected && in_flight.is_empty() {
            break;
        }

        if !lease_loss_detected && cancellation_requested(cancel_signal.as_ref()) {
            lease_loss_detected = true;
            totals.cancelled_due_to_lease_loss = true;

            if matches!(lease_loss_strategy, WorkerLeaseLossStrategy::AbortAll) {
                cancel_in_flight_jobs(&mut in_flight, worker_id.as_str()).await;
                return totals;
            }

            abort_mutating_in_flight_jobs(&mut mutating_abort_handles, worker_id.as_str());
            continue;
        }

        let join_result = if let Some(cancel_signal) = cancel_signal.as_mut() {
            tokio::select! {
                changed = cancel_signal.changed() => {
                    if changed.is_ok() && *cancel_signal.borrow() {
                        lease_loss_detected = true;
                        totals.cancelled_due_to_lease_loss = true;

                        if matches!(lease_loss_strategy, WorkerLeaseLossStrategy::AbortAll) {
                            cancel_in_flight_jobs(&mut in_flight, worker_id.as_str()).await;
                            return totals;
                        }

                        abort_mutating_in_flight_jobs(&mut mutating_abort_handles, worker_id.as_str());
                    }
                    continue;
                }
                joined = in_flight.join_next() => joined,
            }
        } else {
            in_flight.join_next().await
        };

        let Some(join_result) = join_result else {
            break;
        };

        match join_result {
            Ok((worker_id, job_id, run_id, result)) => match result {
                Ok(run) => {
                    totals.executed_jobs = totals.executed_jobs.saturating_add(1);
                    info!(
                        worker_id = %worker_id,
                        job_id = %job_id,
                        run_id = %run_id,
                        status = %run.status.as_str(),
                        attempts = run.attempts,
                        "workflow job executed"
                    );
                }
                Err(error) => {
                    totals.failed_jobs = totals.failed_jobs.saturating_add(1);
                    warn!(
                        worker_id = %worker_id,
                        job_id = %job_id,
                        run_id = %run_id,
                        error = %error,
                        "workflow job execution failed"
                    );
                }
            },
            Err(error) => {
                totals.failed_jobs = totals.failed_jobs.saturating_add(1);
                warn!(
                    worker_id = %worker_id,
                    error = %error,
                    "workflow execution task join failed"
                );
            }
        }
    }

    totals
}

fn cancellation_requested(cancel_signal: Option<&tokio::sync::watch::Receiver<bool>>) -> bool {
    cancel_signal.is_some_and(|receiver| *receiver.borrow())
}

fn abort_mutating_in_flight_jobs(
    abort_handles: &mut Vec<tokio::task::AbortHandle>,
    worker_id: &str,
) {
    if abort_handles.is_empty() {
        return;
    }

    let mut aborted = 0_usize;
    for abort_handle in abort_handles.drain(..) {
        abort_handle.abort();
        aborted = aborted.saturating_add(1);
    }

    warn!(
        worker_id = %worker_id,
        aborted,
        "aborted mutating in-flight workflow tasks due to lease loss"
    );
}

async fn cancel_in_flight_jobs(
    worker_tasks: &mut tokio::task::JoinSet<WorkerExecutionTaskResult>,
    worker_id: &str,
) {
    if worker_tasks.is_empty() {
        return;
    }

    warn!(
        worker_id = %worker_id,
        in_flight = worker_tasks.len(),
        "cancelling in-flight workflow job tasks due to lease loss"
    );

    worker_tasks.abort_all();
    while worker_tasks.join_next().await.is_some() {}
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn required_env(name: &str) -> AppResult<String> {
    env::var(name).map_err(|_| AppError::Validation(format!("{name} is required")))
}

fn parse_env_usize(name: &str, default: usize) -> AppResult<usize> {
    match env::var(name) {
        Ok(value) => value.parse::<usize>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

fn parse_env_u32(name: &str, default: u32) -> AppResult<u32> {
    match env::var(name) {
        Ok(value) => value.parse::<u32>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

fn parse_env_u64(name: &str, default: u64) -> AppResult<u64> {
    match env::var(name) {
        Ok(value) => value.parse::<u64>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

fn parse_optional_env_u32(name: &str) -> AppResult<Option<u32>> {
    match env::var(name) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }

            trimmed.parse::<u32>().map(Some).map_err(|error| {
                AppError::Validation(format!("invalid {name} value '{value}': {error}"))
            })
        }
        Err(_) => Ok(None),
    }
}
