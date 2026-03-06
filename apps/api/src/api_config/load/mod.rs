use std::env;
use std::net::IpAddr;
use std::str::FromStr;

use ipnet::IpNet;
use qryvanta_application::WorkflowExecutionMode;
use qryvanta_core::{AppError, SecretFingerprintRecord, detect_reused_secret_fingerprints};

use self::choices::{
    parse_email_provider_config, parse_rate_limit_store, parse_session_store_backend,
    parse_workflow_execution_mode, parse_workflow_queue_stats_cache_backend,
};
use self::env_parse::{
    parse_env_bool, parse_env_i32, parse_env_u32, parse_env_u64, parse_env_usize,
    parse_optional_non_empty_env, parse_optional_tenant_id_env, required_env,
    required_non_empty_env,
};
use self::isolation::{parse_physical_isolation_mode, validate_physical_isolation_config};
use self::validation::validate_backpressure_config;
use super::{
    ApiConfig, RateLimitStoreConfig, SessionStoreBackend, TotpEncryptionConfig,
    WorkflowQueueStatsCacheBackend,
};

mod choices;
mod env_parse;
mod isolation;
mod validation;

impl ApiConfig {
    pub fn load() -> Result<Self, AppError> {
        let migrate_only = env::args().nth(1).as_deref() == Some("migrate");

        let database_url = required_env("DATABASE_URL")?;
        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned());
        let bootstrap_token = required_env("AUTH_BOOTSTRAP_TOKEN")?;
        let session_secret = required_env("SESSION_SECRET")?;
        if session_secret.len() < 32 {
            return Err(AppError::Validation(
                "SESSION_SECRET must be at least 32 characters".to_owned(),
            ));
        }

        let api_host = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let api_port = env::var("API_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(3001);
        let session_store_backend = parse_session_store_backend()?;

        let webauthn_rp_id = env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_owned());
        let webauthn_rp_origin =
            env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| frontend_url.clone());
        let cookie_secure = parse_env_bool("SESSION_COOKIE_SECURE", false)?;
        let trust_proxy_headers = parse_env_bool("TRUST_PROXY_HEADERS", false)?;
        let trusted_proxy_cidrs = parse_trusted_proxy_cidrs(
            parse_optional_non_empty_env("TRUSTED_PROXY_CIDRS")?.as_deref(),
        )?;

        if trust_proxy_headers && trusted_proxy_cidrs.is_empty() {
            return Err(AppError::Validation(
                "TRUSTED_PROXY_CIDRS is required when TRUST_PROXY_HEADERS=true".to_owned(),
            ));
        }

        let bootstrap_tenant_id = parse_optional_tenant_id_env("DEV_DEFAULT_TENANT_ID")?;

        let totp_encryption = parse_totp_encryption_config()?;

        let email_provider = parse_email_provider_config()?;
        let workflow_execution_mode = parse_workflow_execution_mode()?;

        let worker_shared_secret = parse_optional_non_empty_env("WORKER_SHARED_SECRET")?;
        let deployment_environment = parse_optional_non_empty_env("DEPLOYMENT_ENVIRONMENT")?
            .map(|value| value.trim().to_owned());
        let secret_reuse_guard_records = parse_secret_reuse_guard_records()?;
        validate_secret_reuse_guard(
            deployment_environment.as_deref(),
            secret_reuse_guard_records.as_slice(),
            &build_api_secret_fingerprint_records(
                deployment_environment.as_deref(),
                bootstrap_token.as_str(),
                session_secret.as_str(),
                &totp_encryption,
                worker_shared_secret.as_deref(),
            ),
        )?;
        let redis_url = parse_optional_non_empty_env("REDIS_URL")?;
        let rate_limit_store = parse_rate_limit_store()?;
        let workflow_queue_stats_cache_backend = parse_workflow_queue_stats_cache_backend()?;

        if matches!(workflow_execution_mode, WorkflowExecutionMode::Queued)
            && worker_shared_secret.is_none()
        {
            return Err(AppError::Validation(
                "WORKER_SHARED_SECRET is required when WORKFLOW_EXECUTION_MODE=queued".to_owned(),
            ));
        }

        let workflow_worker_default_lease_seconds =
            parse_env_u32("WORKFLOW_WORKER_DEFAULT_LEASE_SECONDS", 30)?;
        let workflow_worker_max_claim_limit =
            parse_env_usize("WORKFLOW_WORKER_MAX_CLAIM_LIMIT", 25)?;
        let workflow_worker_max_partition_count =
            parse_env_u32("WORKFLOW_WORKER_MAX_PARTITION_COUNT", 128)?;
        let workflow_queue_stats_cache_ttl_seconds =
            parse_env_u32("WORKFLOW_QUEUE_STATS_CACHE_TTL_SECONDS", 0)?;
        let runtime_query_max_limit = parse_env_usize("RUNTIME_QUERY_MAX_LIMIT", 200)?;
        let runtime_query_max_in_flight = parse_env_usize("RUNTIME_QUERY_MAX_IN_FLIGHT", 64)?;
        let workflow_burst_max_in_flight = parse_env_usize("WORKFLOW_BURST_MAX_IN_FLIGHT", 32)?;
        let audit_immutable_mode = parse_env_bool("AUDIT_IMMUTABLE_MODE", false)?;
        let slow_request_threshold_ms = parse_env_u64("SLOW_REQUEST_THRESHOLD_MS", 1000)?;
        let slow_query_threshold_ms = parse_env_u64("SLOW_QUERY_THRESHOLD_MS", 250)?;
        let qrywell_api_base_url = parse_optional_non_empty_env("QRYWELL_API_BASE_URL")?;
        let qrywell_api_key = parse_optional_non_empty_env("QRYWELL_API_KEY")?;
        let qrywell_sync_poll_interval_ms = parse_env_u64("QRYWELL_SYNC_POLL_INTERVAL_MS", 3000)?;
        let qrywell_sync_batch_size = parse_env_usize("QRYWELL_SYNC_BATCH_SIZE", 25)?;
        let qrywell_sync_max_attempts = parse_env_i32("QRYWELL_SYNC_MAX_ATTEMPTS", 12)?;
        let physical_isolation_mode = parse_physical_isolation_mode(
            env::var("PHYSICAL_ISOLATION_MODE")
                .unwrap_or_else(|_| "shared".to_owned())
                .as_str(),
        )?;
        let physical_isolation_tenant_id =
            parse_optional_tenant_id_env("PHYSICAL_ISOLATION_TENANT_ID")?;
        let physical_isolation_schema_template =
            parse_optional_non_empty_env("PHYSICAL_ISOLATION_SCHEMA_TEMPLATE")?;
        let physical_isolation_database_url_template =
            parse_optional_non_empty_env("PHYSICAL_ISOLATION_DATABASE_URL_TEMPLATE")?;
        validate_physical_isolation_config(
            physical_isolation_mode,
            physical_isolation_tenant_id,
            physical_isolation_schema_template.as_deref(),
            physical_isolation_database_url_template.as_deref(),
        )?;

        if qrywell_sync_batch_size == 0 {
            return Err(AppError::Validation(
                "QRYWELL_SYNC_BATCH_SIZE must be greater than zero".to_owned(),
            ));
        }
        if qrywell_sync_max_attempts <= 0 {
            return Err(AppError::Validation(
                "QRYWELL_SYNC_MAX_ATTEMPTS must be greater than zero".to_owned(),
            ));
        }

        if workflow_worker_max_partition_count == 0 {
            return Err(AppError::Validation(
                "WORKFLOW_WORKER_MAX_PARTITION_COUNT must be greater than zero".to_owned(),
            ));
        }
        validate_backpressure_config(
            runtime_query_max_limit,
            runtime_query_max_in_flight,
            workflow_burst_max_in_flight,
        )?;

        let redis_required = matches!(rate_limit_store, RateLimitStoreConfig::Redis)
            || matches!(
                workflow_queue_stats_cache_backend,
                WorkflowQueueStatsCacheBackend::Redis
            )
            || matches!(session_store_backend, SessionStoreBackend::Redis);
        if redis_required && redis_url.is_none() {
            return Err(AppError::Validation(
                "REDIS_URL is required when RATE_LIMIT_STORE=redis or WORKFLOW_QUEUE_STATS_CACHE_BACKEND=redis"
                    .to_owned(),
            ));
        }

        Ok(Self {
            migrate_only,
            database_url,
            frontend_url,
            bootstrap_token,
            _session_secret: session_secret,
            api_host,
            api_port,
            session_store_backend,
            webauthn_rp_id,
            webauthn_rp_origin,
            cookie_secure,
            trust_proxy_headers,
            trusted_proxy_cidrs,
            bootstrap_tenant_id,
            totp_encryption,
            email_provider,
            workflow_execution_mode,
            worker_shared_secret,
            redis_url,
            rate_limit_store,
            workflow_queue_stats_cache_backend,
            workflow_worker_default_lease_seconds,
            workflow_worker_max_claim_limit,
            workflow_worker_max_partition_count,
            workflow_queue_stats_cache_ttl_seconds,
            runtime_query_max_limit,
            runtime_query_max_in_flight,
            workflow_burst_max_in_flight,
            audit_immutable_mode,
            slow_request_threshold_ms,
            slow_query_threshold_ms,
            physical_isolation_mode,
            physical_isolation_tenant_id,
            physical_isolation_schema_template,
            physical_isolation_database_url_template,
            qrywell_api_base_url,
            qrywell_api_key,
            qrywell_sync_poll_interval_ms,
            qrywell_sync_batch_size,
            qrywell_sync_max_attempts,
        })
    }
}

fn parse_trusted_proxy_cidrs(value: Option<&str>) -> Result<Vec<IpNet>, AppError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };

    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(parse_trusted_proxy_entry)
        .collect()
}

fn parse_trusted_proxy_entry(entry: &str) -> Result<IpNet, AppError> {
    IpNet::from_str(entry)
        .or_else(|_| IpAddr::from_str(entry).map(IpNet::from))
        .map_err(|_| {
            AppError::Validation(format!(
                "invalid TRUSTED_PROXY_CIDRS entry '{entry}': expected IP or CIDR notation"
            ))
        })
}

fn validate_totp_encryption_key(value: &str) -> Result<(), AppError> {
    if value == "0".repeat(64) {
        return Err(AppError::Validation(
            "TOTP_ENCRYPTION_KEY must not use the all-zero placeholder value".to_owned(),
        ));
    }

    Ok(())
}

fn parse_totp_encryption_config() -> Result<TotpEncryptionConfig, AppError> {
    let mode = env::var("TOTP_ENCRYPTION_MODE").unwrap_or_else(|_| "static".to_owned());
    let encryption_key = parse_optional_non_empty_env("TOTP_ENCRYPTION_KEY")?;
    let kms_key_id = if mode.eq_ignore_ascii_case("aws_kms_envelope") {
        Some(required_non_empty_env("TOTP_KMS_KEY_ID")?)
    } else {
        None
    };

    parse_totp_encryption_config_from_values(
        mode.as_str(),
        encryption_key.as_deref(),
        kms_key_id.as_deref(),
    )
}

fn parse_totp_encryption_config_from_values(
    mode: &str,
    encryption_key: Option<&str>,
    kms_key_id: Option<&str>,
) -> Result<TotpEncryptionConfig, AppError> {
    if mode.eq_ignore_ascii_case("static") {
        let key_hex = encryption_key.ok_or_else(|| {
            AppError::Validation(
                "TOTP_ENCRYPTION_KEY is required when TOTP_ENCRYPTION_MODE=static".to_owned(),
            )
        })?;
        validate_totp_encryption_key(key_hex)?;
        return Ok(TotpEncryptionConfig::StaticKey {
            key_hex: key_hex.to_owned(),
        });
    }

    if mode.eq_ignore_ascii_case("aws_kms_envelope") {
        if let Some(key_hex) = encryption_key {
            validate_totp_encryption_key(key_hex)?;
        }

        let kms_key_id = kms_key_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AppError::Validation(
                    "TOTP_KMS_KEY_ID is required when TOTP_ENCRYPTION_MODE=aws_kms_envelope"
                        .to_owned(),
                )
            })?;

        return Ok(TotpEncryptionConfig::AwsKmsEnvelope {
            kms_key_id: kms_key_id.to_owned(),
            legacy_static_key_hex: encryption_key.map(str::to_owned),
        });
    }

    Err(AppError::Validation(format!(
        "TOTP_ENCRYPTION_MODE must be either 'static' or 'aws_kms_envelope', got '{mode}'"
    )))
}

fn parse_secret_reuse_guard_records() -> Result<Vec<SecretFingerprintRecord>, AppError> {
    let Some(raw_value) = parse_optional_non_empty_env("SECRET_REUSE_GUARD_FINGERPRINTS")? else {
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
) -> Result<(), AppError> {
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

fn build_api_secret_fingerprint_records(
    deployment_environment: Option<&str>,
    bootstrap_token: &str,
    session_secret: &str,
    totp_encryption: &TotpEncryptionConfig,
    worker_shared_secret: Option<&str>,
) -> Vec<SecretFingerprintRecord> {
    let Some(deployment_environment) = deployment_environment else {
        return Vec::new();
    };

    let mut records = vec![
        SecretFingerprintRecord::from_secret(
            deployment_environment,
            "AUTH_BOOTSTRAP_TOKEN",
            bootstrap_token,
        ),
        SecretFingerprintRecord::from_secret(
            deployment_environment,
            "SESSION_SECRET",
            session_secret,
        ),
    ];

    match totp_encryption {
        TotpEncryptionConfig::StaticKey { key_hex } => {
            records.push(SecretFingerprintRecord::from_secret(
                deployment_environment,
                "TOTP_ENCRYPTION_KEY",
                key_hex,
            ))
        }
        TotpEncryptionConfig::AwsKmsEnvelope {
            legacy_static_key_hex: Some(key_hex),
            ..
        } => records.push(SecretFingerprintRecord::from_secret(
            deployment_environment,
            "TOTP_ENCRYPTION_KEY",
            key_hex,
        )),
        TotpEncryptionConfig::AwsKmsEnvelope {
            legacy_static_key_hex: None,
            ..
        } => {}
    }

    if let Some(worker_shared_secret) = worker_shared_secret {
        records.push(SecretFingerprintRecord::from_secret(
            deployment_environment,
            "WORKER_SHARED_SECRET",
            worker_shared_secret,
        ));
    }

    records
}

#[cfg(test)]
mod tests {
    use super::isolation::{parse_physical_isolation_mode, validate_physical_isolation_config};
    use super::*;
    use crate::api_config::PhysicalIsolationMode;

    #[test]
    fn physical_isolation_mode_parser_accepts_supported_values() {
        assert_eq!(
            parse_physical_isolation_mode("shared").unwrap_or_else(|_| unreachable!()),
            PhysicalIsolationMode::Shared
        );
        assert_eq!(
            parse_physical_isolation_mode("tenant_per_schema").unwrap_or_else(|_| unreachable!()),
            PhysicalIsolationMode::TenantPerSchema
        );
        assert_eq!(
            parse_physical_isolation_mode("tenant_per_database").unwrap_or_else(|_| unreachable!()),
            PhysicalIsolationMode::TenantPerDatabase
        );
    }

    #[test]
    fn physical_isolation_validation_requires_tenant_id_and_templates() {
        let tenant_id = qryvanta_core::TenantId::new();
        let tenant_per_schema = validate_physical_isolation_config(
            PhysicalIsolationMode::TenantPerSchema,
            Some(tenant_id),
            Some("tenant_{tenant_id}"),
            None,
        );
        assert!(tenant_per_schema.is_ok());

        let tenant_per_database = validate_physical_isolation_config(
            PhysicalIsolationMode::TenantPerDatabase,
            Some(tenant_id),
            None,
            Some("postgres://user:pass@host/db_{tenant_id}"),
        );
        assert!(tenant_per_database.is_ok());

        let missing_template = validate_physical_isolation_config(
            PhysicalIsolationMode::TenantPerSchema,
            Some(tenant_id),
            None,
            None,
        );
        assert!(missing_template.is_err());

        let missing_placeholder = validate_physical_isolation_config(
            PhysicalIsolationMode::TenantPerDatabase,
            Some(tenant_id),
            None,
            Some("postgres://user:pass@host/isolation"),
        );
        assert!(missing_placeholder.is_err());
    }

    #[test]
    fn backpressure_config_requires_positive_limits() {
        assert!(validate_backpressure_config(200, 64, 32).is_ok());
        assert!(validate_backpressure_config(0, 64, 32).is_err());
        assert!(validate_backpressure_config(200, 0, 32).is_err());
        assert!(validate_backpressure_config(200, 64, 0).is_err());
    }

    #[test]
    fn totp_encryption_key_rejects_all_zero_placeholder() {
        let result = validate_totp_encryption_key(&"0".repeat(64));
        assert!(result.is_err());
    }

    #[test]
    fn totp_encryption_key_accepts_non_placeholder_secret() {
        let result = validate_totp_encryption_key(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn totp_encryption_parser_accepts_static_mode_with_valid_key() {
        let result = parse_totp_encryption_config_from_values(
            "static",
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            None,
        );

        assert!(matches!(result, Ok(TotpEncryptionConfig::StaticKey { .. })));
    }

    #[test]
    fn totp_encryption_parser_accepts_aws_kms_mode_with_legacy_fallback() {
        let result = parse_totp_encryption_config_from_values(
            "aws_kms_envelope",
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            Some("alias/qryvanta-totp"),
        );

        assert!(matches!(
            result,
            Ok(TotpEncryptionConfig::AwsKmsEnvelope { .. })
        ));
    }

    #[test]
    fn totp_encryption_parser_rejects_unknown_modes() {
        let result = parse_totp_encryption_config_from_values("bad", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn totp_encryption_parser_requires_kms_key_id_in_aws_mode() {
        let result = parse_totp_encryption_config_from_values("aws_kms_envelope", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn secret_reuse_guard_requires_deployment_environment() {
        let guard_records = vec![SecretFingerprintRecord::from_secret(
            "staging",
            "SESSION_SECRET",
            "shared-secret",
        )];
        let current_records = vec![SecretFingerprintRecord::from_secret(
            "production",
            "SESSION_SECRET",
            "shared-secret",
        )];

        let result = validate_secret_reuse_guard(None, &guard_records, &current_records);
        assert!(result.is_err());
    }

    #[test]
    fn secret_reuse_guard_rejects_cross_environment_collisions() {
        let guard_records = vec![SecretFingerprintRecord::from_secret(
            "staging",
            "SESSION_SECRET",
            "shared-secret",
        )];
        let current_records = vec![SecretFingerprintRecord::from_secret(
            "production",
            "SESSION_SECRET",
            "shared-secret",
        )];

        let result =
            validate_secret_reuse_guard(Some("production"), &guard_records, &current_records);
        assert!(result.is_err());
    }

    #[test]
    fn trusted_proxy_parser_accepts_cidrs_and_single_ips() {
        let parsed = parse_trusted_proxy_cidrs(Some("127.0.0.1, 10.0.0.0/24, 2001:db8::/64"))
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].to_string(), "127.0.0.1/32");
        assert_eq!(parsed[1].to_string(), "10.0.0.0/24");
        assert_eq!(parsed[2].to_string(), "2001:db8::/64");
    }

    #[test]
    fn trusted_proxy_parser_rejects_invalid_entries() {
        let parsed = parse_trusted_proxy_cidrs(Some("not-a-cidr"));
        assert!(parsed.is_err());
    }
}
