use std::env;

use qryvanta_application::WorkflowClaimPartition;
use qryvanta_core::{
    AppError, AppResult, SecretFingerprintRecord, TenantId, detect_reused_secret_fingerprints,
    optional_secret, required_secret,
};

#[derive(Debug, Clone)]
pub(crate) struct WorkerConfig {
    pub(crate) database_url: String,
    pub(crate) api_base_url: String,
    pub(crate) worker_shared_secret: String,
    pub(crate) worker_id: String,
    pub(crate) redis_url: Option<String>,
    pub(crate) coordination_backend: WorkerCoordinationBackend,
    pub(crate) coordination_lease_seconds: u32,
    pub(crate) coordination_scope_key: String,
    pub(crate) lease_loss_strategy: WorkerLeaseLossStrategy,
    pub(crate) claim_limit: usize,
    pub(crate) max_concurrency: usize,
    pub(crate) lease_seconds: u32,
    pub(crate) poll_interval_ms: u64,
    pub(crate) partition: Option<WorkflowClaimPartition>,
    pub(crate) physical_isolation_mode: WorkerPhysicalIsolationMode,
    pub(crate) physical_isolation_tenant_id: Option<TenantId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerCoordinationBackend {
    None,
    Redis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerLeaseLossStrategy {
    AbortAll,
    GracefulDrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerPhysicalIsolationMode {
    Shared,
    TenantPerSchema,
    TenantPerDatabase,
}

impl WorkerConfig {
    pub(crate) fn load() -> AppResult<Self> {
        let database_url = required_env("DATABASE_URL")?;
        let api_base_url = env::var("WORKER_API_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3001".to_owned())
            .trim_end_matches('/')
            .to_owned();
        let worker_shared_secret = required_env("WORKER_SHARED_SECRET")?;
        let deployment_environment =
            optional_secret("DEPLOYMENT_ENVIRONMENT")?.map(|value| value.trim().to_owned());
        let secret_reuse_guard_records = parse_secret_reuse_guard_records()?;
        validate_secret_reuse_guard(
            deployment_environment.as_deref(),
            secret_reuse_guard_records.as_slice(),
            build_worker_secret_fingerprint_records(
                deployment_environment.as_deref(),
                worker_shared_secret.as_str(),
            )
            .as_slice(),
        )?;
        let worker_id = env::var("WORKER_ID")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("worker-{}", std::process::id()));
        let redis_url = optional_secret("REDIS_URL")?;
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
        let physical_isolation_mode = WorkerPhysicalIsolationMode::parse(
            env::var("PHYSICAL_ISOLATION_MODE")
                .unwrap_or_else(|_| "shared".to_owned())
                .as_str(),
        )?;
        let physical_isolation_tenant_id =
            parse_optional_tenant_id_env("PHYSICAL_ISOLATION_TENANT_ID")?;

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

        if !matches!(physical_isolation_mode, WorkerPhysicalIsolationMode::Shared)
            && physical_isolation_tenant_id.is_none()
        {
            return Err(AppError::Validation(
                "PHYSICAL_ISOLATION_TENANT_ID is required when PHYSICAL_ISOLATION_MODE is tenant_per_schema or tenant_per_database"
                    .to_owned(),
            ));
        }

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
            physical_isolation_mode,
            physical_isolation_tenant_id,
        })
    }

    pub(crate) fn secret_fingerprint_records(
        &self,
        environment: &str,
    ) -> Vec<SecretFingerprintRecord> {
        vec![SecretFingerprintRecord::from_secret(
            environment,
            "WORKER_SHARED_SECRET",
            &self.worker_shared_secret,
        )]
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

impl WorkerPhysicalIsolationMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::TenantPerSchema => "tenant_per_schema",
            Self::TenantPerDatabase => "tenant_per_database",
        }
    }

    fn parse(value: &str) -> AppResult<Self> {
        if value.eq_ignore_ascii_case("shared") {
            return Ok(Self::Shared);
        }

        if value.eq_ignore_ascii_case("tenant_per_schema") {
            return Ok(Self::TenantPerSchema);
        }

        if value.eq_ignore_ascii_case("tenant_per_database") {
            return Ok(Self::TenantPerDatabase);
        }

        Err(AppError::Validation(format!(
            "PHYSICAL_ISOLATION_MODE must be one of 'shared', 'tenant_per_schema', or 'tenant_per_database', got '{value}'"
        )))
    }
}

impl std::fmt::Display for WorkerPhysicalIsolationMode {
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

fn required_env(name: &str) -> AppResult<String> {
    required_secret(name)
}

fn parse_secret_reuse_guard_records() -> AppResult<Vec<SecretFingerprintRecord>> {
    let Some(raw_value) = optional_secret("SECRET_REUSE_GUARD_FINGERPRINTS")? else {
        return Ok(Vec::new());
    };

    serde_json::from_str::<Vec<SecretFingerprintRecord>>(raw_value.as_str()).map_err(|error| {
        AppError::Validation(format!(
            "invalid SECRET_REUSE_GUARD_FINGERPRINTS JSON: {error}"
        ))
    })
}

fn validate_secret_reuse_guard(
    deployment_environment: Option<&str>,
    guard_records: &[SecretFingerprintRecord],
    current_records: &[SecretFingerprintRecord],
) -> AppResult<()> {
    if guard_records.is_empty() {
        return Ok(());
    }

    let deployment_environment = deployment_environment.ok_or_else(|| {
        AppError::Validation(
            "DEPLOYMENT_ENVIRONMENT is required when SECRET_REUSE_GUARD_FINGERPRINTS is configured"
                .to_owned(),
        )
    })?;

    detect_reused_secret_fingerprints(deployment_environment, current_records, guard_records)
}

fn build_worker_secret_fingerprint_records(
    deployment_environment: Option<&str>,
    worker_shared_secret: &str,
) -> Vec<SecretFingerprintRecord> {
    let Some(deployment_environment) = deployment_environment else {
        return Vec::new();
    };

    vec![SecretFingerprintRecord::from_secret(
        deployment_environment,
        "WORKER_SHARED_SECRET",
        worker_shared_secret,
    )]
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

fn parse_optional_tenant_id_env(name: &str) -> AppResult<Option<TenantId>> {
    match env::var(name) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }

            let tenant_uuid = uuid::Uuid::parse_str(trimmed).map_err(|error| {
                AppError::Validation(format!("invalid {name} value '{value}': {error}"))
            })?;
            Ok(Some(TenantId::from_uuid(tenant_uuid)))
        }
        Err(_) => Ok(None),
    }
}
