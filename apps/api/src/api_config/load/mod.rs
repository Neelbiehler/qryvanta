use std::env;

use qryvanta_application::WorkflowExecutionMode;
use qryvanta_core::AppError;

use self::choices::{
    parse_email_provider_config, parse_rate_limit_store, parse_session_store_backend,
    parse_workflow_execution_mode, parse_workflow_queue_stats_cache_backend,
};
use self::env_parse::{
    parse_env_bool, parse_env_i32, parse_env_u32, parse_env_u64, parse_env_usize,
    parse_optional_non_empty_env, parse_optional_tenant_id_env, required_env,
};
use self::isolation::{parse_physical_isolation_mode, validate_physical_isolation_config};
use self::validation::validate_backpressure_config;
use super::{ApiConfig, RateLimitStoreConfig, SessionStoreBackend, WorkflowQueueStatsCacheBackend};

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
        let cookie_secure = env::var("SESSION_COOKIE_SECURE")
            .unwrap_or_else(|_| "false".to_owned())
            .eq_ignore_ascii_case("true");

        let bootstrap_tenant_id = parse_optional_tenant_id_env("DEV_DEFAULT_TENANT_ID")?;

        let totp_encryption_key =
            env::var("TOTP_ENCRYPTION_KEY").unwrap_or_else(|_| "0".repeat(64));

        let email_provider = parse_email_provider_config()?;
        let workflow_execution_mode = parse_workflow_execution_mode()?;

        let worker_shared_secret = parse_optional_non_empty_env("WORKER_SHARED_SECRET");
        let redis_url = parse_optional_non_empty_env("REDIS_URL");
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
        let qrywell_api_base_url = parse_optional_non_empty_env("QRYWELL_API_BASE_URL");
        let qrywell_api_key = parse_optional_non_empty_env("QRYWELL_API_KEY");
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
            parse_optional_non_empty_env("PHYSICAL_ISOLATION_SCHEMA_TEMPLATE");
        let physical_isolation_database_url_template =
            parse_optional_non_empty_env("PHYSICAL_ISOLATION_DATABASE_URL_TEMPLATE");
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
            bootstrap_tenant_id,
            totp_encryption_key,
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
}
