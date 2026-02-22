//! Qryvanta workflow worker runtime.

#![forbid(unsafe_code)]

use std::env;
use std::sync::Arc;
use std::time::Duration;

use qryvanta_application::{
    AuthorizationService, MetadataService, WorkflowExecutionMode, WorkflowService,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    WorkflowAction, WorkflowDefinition, WorkflowDefinitionInput, WorkflowStep, WorkflowTrigger,
};
use qryvanta_infrastructure::{
    PostgresAuditRepository, PostgresAuthorizationRepository, PostgresMetadataRepository,
    PostgresWorkflowRepository,
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
    claim_limit: usize,
    lease_seconds: u32,
    poll_interval_ms: u64,
}

#[derive(Debug, Serialize)]
struct ClaimWorkflowJobsRequest {
    limit: usize,
    lease_seconds: u32,
}

#[derive(Debug, Serialize)]
struct WorkerHeartbeatRequest {
    claimed_jobs: u32,
    executed_jobs: u32,
    failed_jobs: u32,
}

#[derive(Debug, Deserialize)]
struct ClaimedWorkflowJobsResponse {
    jobs: Vec<ClaimedWorkflowJobResponse>,
}

#[derive(Debug, Deserialize)]
struct ClaimedWorkflowJobResponse {
    job_id: String,
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
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| AppError::Internal(format!("failed to build HTTP client: {error}")))?;

    info!(
        worker_id = %config.worker_id,
        api_base_url = %config.api_base_url,
        claim_limit = config.claim_limit,
        lease_seconds = config.lease_seconds,
        poll_interval_ms = config.poll_interval_ms,
        "qryvanta-worker started"
    );

    loop {
        match claim_jobs(&http_client, &config).await {
            Ok(claimed_jobs) => {
                let claimed_job_count = u32::try_from(claimed_jobs.len()).unwrap_or(u32::MAX);
                let mut executed_jobs = 0_u32;
                let mut failed_jobs = 0_u32;

                if claimed_jobs.is_empty() {
                    if let Err(error) = send_heartbeat(&http_client, &config, 0, 0, 0).await {
                        warn!(
                            worker_id = %config.worker_id,
                            error = %error,
                            "failed to publish worker heartbeat"
                        );
                    }
                    tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
                    continue;
                }

                info!(
                    worker_id = %config.worker_id,
                    claimed_count = claimed_jobs.len(),
                    "claimed workflow jobs"
                );

                for claimed_job in claimed_jobs {
                    let queued_job = match claimed_job.try_into_claimed_job() {
                        Ok(job) => job,
                        Err(error) => {
                            warn!(
                                worker_id = %config.worker_id,
                                error = %error,
                                "failed to parse claimed workflow job payload"
                            );
                            continue;
                        }
                    };
                    let job_id = queued_job.job_id.clone();
                    let run_id = queued_job.run_id.clone();

                    match workflow_service
                        .execute_claimed_job(config.worker_id.as_str(), queued_job)
                        .await
                    {
                        Ok(run) => {
                            executed_jobs = executed_jobs.saturating_add(1);
                            info!(
                                worker_id = %config.worker_id,
                                job_id = %job_id,
                                run_id = %run_id,
                                status = %run.status.as_str(),
                                attempts = run.attempts,
                                "workflow job executed"
                            );
                        }
                        Err(error) => {
                            failed_jobs = failed_jobs.saturating_add(1);
                            warn!(
                                worker_id = %config.worker_id,
                                job_id = %job_id,
                                run_id = %run_id,
                                error = %error,
                                "workflow job execution failed"
                            );
                        }
                    }
                }

                if let Err(error) = send_heartbeat(
                    &http_client,
                    &config,
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
            }
            Err(error) => {
                warn!(
                    worker_id = %config.worker_id,
                    error = %error,
                    "failed to claim workflow jobs"
                );
                tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
            }
        }
    }
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

    WorkflowService::new(
        authorization_service,
        workflow_repository,
        runtime_record_service,
        audit_repository,
        WorkflowExecutionMode::Queued,
    )
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
        let claim_limit = parse_env_usize("WORKER_CLAIM_LIMIT", 10)?;
        let lease_seconds = parse_env_u32("WORKER_LEASE_SECONDS", 30)?;
        let poll_interval_ms = parse_env_u64("WORKER_POLL_INTERVAL_MS", 1500)?;

        if claim_limit == 0 {
            return Err(AppError::Validation(
                "WORKER_CLAIM_LIMIT must be greater than zero".to_owned(),
            ));
        }

        if lease_seconds == 0 {
            return Err(AppError::Validation(
                "WORKER_LEASE_SECONDS must be greater than zero".to_owned(),
            ));
        }

        if poll_interval_ms == 0 {
            return Err(AppError::Validation(
                "WORKER_POLL_INTERVAL_MS must be greater than zero".to_owned(),
            ));
        }

        Ok(Self {
            database_url,
            api_base_url,
            worker_shared_secret,
            worker_id,
            claim_limit,
            lease_seconds,
            poll_interval_ms,
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
