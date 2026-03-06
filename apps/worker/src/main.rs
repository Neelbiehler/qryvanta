//! Qryvanta workflow worker runtime.

#![forbid(unsafe_code)]

use std::env;
use std::sync::Arc;
use std::time::Duration;

use qryvanta_application::{
    AuthorizationService, EmailService, MetadataService, WorkflowExecutionMode, WorkflowService,
    WorkflowWorkerLease, WorkflowWorkerLeaseCoordinator,
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

mod config;
mod job_execution;

use config::{WorkerConfig, WorkerCoordinationBackend};
use job_execution::execute_claimed_jobs;

#[derive(Debug, Serialize)]
struct ClaimWorkflowJobsRequest {
    limit: usize,
    lease_seconds: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_id: Option<String>,
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
    let args = env::args().collect::<Vec<_>>();
    let command = args.get(1).map(String::as_str);

    let config = WorkerConfig::load()?;
    if command == Some("print-secret-fingerprints") {
        print_secret_fingerprints(&config)?;
        return Ok(());
    }
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
        physical_isolation_mode = %config.physical_isolation_mode,
        physical_isolation_tenant_id = config.physical_isolation_tenant_id.map(|value| value.to_string()),
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
        .header(
            "x-trace-id",
            next_worker_trace_id(config.worker_id.as_str()),
        )
        .json(&ClaimWorkflowJobsRequest {
            limit: config.claim_limit,
            lease_seconds: config.lease_seconds,
            partition_count: config.partition.map(|value| value.partition_count()),
            partition_index: config.partition.map(|value| value.partition_index()),
            tenant_id: config
                .physical_isolation_tenant_id
                .map(|tenant_id| tenant_id.to_string()),
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
        .header(
            "x-trace-id",
            next_worker_trace_id(config.worker_id.as_str()),
        )
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

fn next_worker_trace_id(worker_id: &str) -> String {
    format!("worker-{worker_id}-{}", uuid::Uuid::new_v4())
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

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn print_secret_fingerprints(config: &WorkerConfig) -> Result<(), AppError> {
    let deployment_environment = env::var("DEPLOYMENT_ENVIRONMENT")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Validation(
                "DEPLOYMENT_ENVIRONMENT is required for print-secret-fingerprints".to_owned(),
            )
        })?;
    let fingerprints = config.secret_fingerprint_records(deployment_environment.as_str());
    let output = serde_json::to_string_pretty(&fingerprints).map_err(|error| {
        AppError::Internal(format!("failed to serialize fingerprints: {error}"))
    })?;
    println!("{output}");
    Ok(())
}
