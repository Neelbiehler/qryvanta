use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use ipnet::IpNet;
use qryvanta_application::WorkflowExecutionMode;
use qryvanta_core::{AppError, SecretFingerprintRecord, TenantId};

#[derive(Debug, Clone)]
pub struct SmtpRuntimeConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

#[derive(Debug, Clone)]
pub enum EmailProviderConfig {
    Console,
    Smtp(SmtpRuntimeConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitStoreConfig {
    Postgres,
    Redis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowQueueStatsCacheBackend {
    InMemory,
    Redis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStoreBackend {
    Postgres,
    Redis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIsolationMode {
    Shared,
    TenantPerSchema,
    TenantPerDatabase,
}

impl PhysicalIsolationMode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::TenantPerSchema => "tenant_per_schema",
            Self::TenantPerDatabase => "tenant_per_database",
        }
    }
}

#[derive(Debug, Clone)]
pub enum TotpEncryptionConfig {
    StaticKey {
        key_hex: String,
    },
    AwsKmsEnvelope {
        kms_key_id: String,
        legacy_static_key_hex: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub migrate_only: bool,
    pub database_url: String,
    pub frontend_url: String,
    pub bootstrap_token: String,
    pub _session_secret: String,
    pub api_host: String,
    pub api_port: u16,
    pub session_store_backend: SessionStoreBackend,
    pub webauthn_rp_id: String,
    pub webauthn_rp_origin: String,
    pub cookie_secure: bool,
    pub trust_proxy_headers: bool,
    pub trusted_proxy_cidrs: Vec<IpNet>,
    pub bootstrap_tenant_id: Option<TenantId>,
    pub totp_encryption: TotpEncryptionConfig,
    pub email_provider: EmailProviderConfig,
    pub workflow_execution_mode: WorkflowExecutionMode,
    pub worker_shared_secret: Option<String>,
    pub redis_url: Option<String>,
    pub rate_limit_store: RateLimitStoreConfig,
    pub workflow_queue_stats_cache_backend: WorkflowQueueStatsCacheBackend,
    pub workflow_worker_default_lease_seconds: u32,
    pub workflow_worker_max_claim_limit: usize,
    pub workflow_worker_max_partition_count: u32,
    pub workflow_queue_stats_cache_ttl_seconds: u32,
    pub runtime_query_max_limit: usize,
    pub runtime_query_max_in_flight: usize,
    pub workflow_burst_max_in_flight: usize,
    pub audit_immutable_mode: bool,
    pub slow_request_threshold_ms: u64,
    pub slow_query_threshold_ms: u64,
    pub physical_isolation_mode: PhysicalIsolationMode,
    pub physical_isolation_tenant_id: Option<TenantId>,
    pub physical_isolation_schema_template: Option<String>,
    pub physical_isolation_database_url_template: Option<String>,
    pub qrywell_api_base_url: Option<String>,
    pub qrywell_api_key: Option<String>,
    pub qrywell_sync_poll_interval_ms: u64,
    pub qrywell_sync_batch_size: usize,
    pub qrywell_sync_max_attempts: i32,
}

impl ApiConfig {
    #[must_use]
    pub fn requires_redis(&self) -> bool {
        matches!(self.rate_limit_store, RateLimitStoreConfig::Redis)
            || matches!(
                self.workflow_queue_stats_cache_backend,
                WorkflowQueueStatsCacheBackend::Redis
            )
            || matches!(self.session_store_backend, SessionStoreBackend::Redis)
    }

    pub fn socket_address(&self) -> Result<SocketAddr, AppError> {
        let host = IpAddr::from_str(&self.api_host).map_err(|error| {
            AppError::Internal(format!("invalid API_HOST '{}': {error}", self.api_host))
        })?;
        Ok(SocketAddr::from((host, self.api_port)))
    }

    #[must_use]
    pub fn secret_fingerprint_records(&self, environment: &str) -> Vec<SecretFingerprintRecord> {
        let mut records = vec![
            SecretFingerprintRecord::from_secret(
                environment,
                "AUTH_BOOTSTRAP_TOKEN",
                &self.bootstrap_token,
            ),
            SecretFingerprintRecord::from_secret(
                environment,
                "SESSION_SECRET",
                &self._session_secret,
            ),
        ];

        match &self.totp_encryption {
            TotpEncryptionConfig::StaticKey { key_hex } => {
                records.push(SecretFingerprintRecord::from_secret(
                    environment,
                    "TOTP_ENCRYPTION_KEY",
                    key_hex,
                ));
            }
            TotpEncryptionConfig::AwsKmsEnvelope {
                legacy_static_key_hex: Some(key_hex),
                ..
            } => {
                records.push(SecretFingerprintRecord::from_secret(
                    environment,
                    "TOTP_ENCRYPTION_KEY",
                    key_hex,
                ));
            }
            TotpEncryptionConfig::AwsKmsEnvelope {
                legacy_static_key_hex: None,
                ..
            } => {}
        }

        if let Some(worker_shared_secret) = &self.worker_shared_secret {
            records.push(SecretFingerprintRecord::from_secret(
                environment,
                "WORKER_SHARED_SECRET",
                worker_shared_secret,
            ));
        }

        records
    }
}

mod load;
mod tracing;

pub use tracing::init_tracing;
