use std::env;

use qryvanta_application::WorkflowExecutionMode;
use qryvanta_core::AppError;

use crate::api_config::{
    EmailProviderConfig, RateLimitStoreConfig, SessionStoreBackend, SmtpRuntimeConfig,
    WorkflowQueueStatsCacheBackend,
};

use super::env_parse::required_non_empty_env;

pub(super) fn parse_session_store_backend() -> Result<SessionStoreBackend, AppError> {
    match env::var("SESSION_STORE").unwrap_or_else(|_| "postgres".to_owned()) {
        value if value.eq_ignore_ascii_case("postgres") => Ok(SessionStoreBackend::Postgres),
        value if value.eq_ignore_ascii_case("redis") => Ok(SessionStoreBackend::Redis),
        other => Err(AppError::Validation(format!(
            "SESSION_STORE must be either 'postgres' or 'redis', got '{other}'"
        ))),
    }
}

pub(super) fn parse_email_provider_config() -> Result<EmailProviderConfig, AppError> {
    match env::var("EMAIL_PROVIDER")
        .unwrap_or_else(|_| "console".to_owned())
        .as_str()
    {
        "console" => Ok(EmailProviderConfig::Console),
        "smtp" => {
            let port = required_non_empty_env("SMTP_PORT")?
                .parse::<u16>()
                .map_err(|error| AppError::Validation(format!("invalid SMTP_PORT: {error}")))?;
            Ok(EmailProviderConfig::Smtp(SmtpRuntimeConfig {
                host: required_non_empty_env("SMTP_HOST")?,
                port,
                username: required_non_empty_env("SMTP_USERNAME")?,
                password: required_non_empty_env("SMTP_PASSWORD")?,
                from_address: required_non_empty_env("SMTP_FROM_ADDRESS")?,
            }))
        }
        other => Err(AppError::Validation(format!(
            "EMAIL_PROVIDER must be either 'console' or 'smtp', got '{other}'"
        ))),
    }
}

pub(super) fn parse_workflow_execution_mode() -> Result<WorkflowExecutionMode, AppError> {
    match env::var("WORKFLOW_EXECUTION_MODE").unwrap_or_else(|_| "inline".to_owned()) {
        value if value.eq_ignore_ascii_case("inline") => Ok(WorkflowExecutionMode::Inline),
        value if value.eq_ignore_ascii_case("queued") => Ok(WorkflowExecutionMode::Queued),
        other => Err(AppError::Validation(format!(
            "WORKFLOW_EXECUTION_MODE must be either 'inline' or 'queued', got '{other}'"
        ))),
    }
}

pub(super) fn parse_rate_limit_store() -> Result<RateLimitStoreConfig, AppError> {
    match env::var("RATE_LIMIT_STORE").unwrap_or_else(|_| "postgres".to_owned()) {
        value if value.eq_ignore_ascii_case("postgres") => Ok(RateLimitStoreConfig::Postgres),
        value if value.eq_ignore_ascii_case("redis") => Ok(RateLimitStoreConfig::Redis),
        other => Err(AppError::Validation(format!(
            "RATE_LIMIT_STORE must be either 'postgres' or 'redis', got '{other}'"
        ))),
    }
}

pub(super) fn parse_workflow_queue_stats_cache_backend()
-> Result<WorkflowQueueStatsCacheBackend, AppError> {
    match env::var("WORKFLOW_QUEUE_STATS_CACHE_BACKEND").unwrap_or_else(|_| "in_memory".to_owned())
    {
        value if value.eq_ignore_ascii_case("in_memory") => {
            Ok(WorkflowQueueStatsCacheBackend::InMemory)
        }
        value if value.eq_ignore_ascii_case("redis") => Ok(WorkflowQueueStatsCacheBackend::Redis),
        other => Err(AppError::Validation(format!(
            "WORKFLOW_QUEUE_STATS_CACHE_BACKEND must be either 'in_memory' or 'redis', got '{other}'"
        ))),
    }
}
