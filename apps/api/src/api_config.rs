use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use qryvanta_application::WorkflowExecutionMode;
use qryvanta_core::{AppError, TenantId};
use tracing_subscriber::EnvFilter;

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

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub migrate_only: bool,
    pub database_url: String,
    pub frontend_url: String,
    pub bootstrap_token: String,
    pub _session_secret: String,
    pub api_host: String,
    pub api_port: u16,
    pub webauthn_rp_id: String,
    pub webauthn_rp_origin: String,
    pub cookie_secure: bool,
    pub bootstrap_tenant_id: Option<TenantId>,
    pub totp_encryption_key: String,
    pub email_provider: EmailProviderConfig,
    pub workflow_execution_mode: WorkflowExecutionMode,
    pub worker_shared_secret: Option<String>,
    pub workflow_worker_default_lease_seconds: u32,
    pub workflow_worker_max_claim_limit: usize,
}

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

        let webauthn_rp_id = env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_owned());
        let webauthn_rp_origin =
            env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| frontend_url.clone());
        let cookie_secure = env::var("SESSION_COOKIE_SECURE")
            .unwrap_or_else(|_| "false".to_owned())
            .eq_ignore_ascii_case("true");

        let bootstrap_tenant_id = env::var("DEV_DEFAULT_TENANT_ID")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                uuid::Uuid::parse_str(value.as_str())
                    .map(TenantId::from_uuid)
                    .map_err(|error| {
                        AppError::Validation(format!("invalid DEV_DEFAULT_TENANT_ID: {error}"))
                    })
            })
            .transpose()?;

        let totp_encryption_key =
            env::var("TOTP_ENCRYPTION_KEY").unwrap_or_else(|_| "0".repeat(64));

        let email_provider = match env::var("EMAIL_PROVIDER")
            .unwrap_or_else(|_| "console".to_owned())
            .as_str()
        {
            "console" => EmailProviderConfig::Console,
            "smtp" => {
                let port = required_non_empty_env("SMTP_PORT")?
                    .parse::<u16>()
                    .map_err(|error| AppError::Validation(format!("invalid SMTP_PORT: {error}")))?;
                EmailProviderConfig::Smtp(SmtpRuntimeConfig {
                    host: required_non_empty_env("SMTP_HOST")?,
                    port,
                    username: required_non_empty_env("SMTP_USERNAME")?,
                    password: required_non_empty_env("SMTP_PASSWORD")?,
                    from_address: required_non_empty_env("SMTP_FROM_ADDRESS")?,
                })
            }
            other => {
                return Err(AppError::Validation(format!(
                    "EMAIL_PROVIDER must be either 'console' or 'smtp', got '{other}'"
                )));
            }
        };

        let workflow_execution_mode =
            match env::var("WORKFLOW_EXECUTION_MODE").unwrap_or_else(|_| "inline".to_owned()) {
                value if value.eq_ignore_ascii_case("inline") => WorkflowExecutionMode::Inline,
                value if value.eq_ignore_ascii_case("queued") => WorkflowExecutionMode::Queued,
                other => {
                    return Err(AppError::Validation(format!(
                        "WORKFLOW_EXECUTION_MODE must be either 'inline' or 'queued', got '{other}'"
                    )));
                }
            };

        let worker_shared_secret = env::var("WORKER_SHARED_SECRET")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());

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

        Ok(Self {
            migrate_only,
            database_url,
            frontend_url,
            bootstrap_token,
            _session_secret: session_secret,
            api_host,
            api_port,
            webauthn_rp_id,
            webauthn_rp_origin,
            cookie_secure,
            bootstrap_tenant_id,
            totp_encryption_key,
            email_provider,
            workflow_execution_mode,
            worker_shared_secret,
            workflow_worker_default_lease_seconds,
            workflow_worker_max_claim_limit,
        })
    }

    pub fn socket_address(&self) -> Result<SocketAddr, AppError> {
        let host = IpAddr::from_str(&self.api_host).map_err(|error| {
            AppError::Internal(format!("invalid API_HOST '{}': {error}", self.api_host))
        })?;
        Ok(SocketAddr::from((host, self.api_port)))
    }
}

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn required_env(name: &str) -> Result<String, AppError> {
    env::var(name).map_err(|_| AppError::Validation(format!("{name} is required")))
}

fn required_non_empty_env(name: &str) -> Result<String, AppError> {
    let value = required_env(name)?;
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{name} must not be empty")));
    }

    Ok(value)
}

fn parse_env_u32(name: &str, default: u32) -> Result<u32, AppError> {
    match env::var(name) {
        Ok(value) => value.parse::<u32>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

fn parse_env_usize(name: &str, default: usize) -> Result<usize, AppError> {
    match env::var(name) {
        Ok(value) => value.parse::<usize>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}
