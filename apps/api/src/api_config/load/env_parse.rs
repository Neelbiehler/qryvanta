use std::env;

use qryvanta_core::{
    AppError, TenantId, optional_secret, required_non_empty_secret, required_secret,
};

pub(super) fn required_env(name: &str) -> Result<String, AppError> {
    required_secret(name)
}

pub(super) fn required_non_empty_env(name: &str) -> Result<String, AppError> {
    required_non_empty_secret(name)
}

pub(super) fn parse_optional_non_empty_env(name: &str) -> Result<Option<String>, AppError> {
    optional_secret(name)
}

pub(super) fn parse_optional_tenant_id_env(name: &str) -> Result<Option<TenantId>, AppError> {
    parse_optional_non_empty_env(name)?
        .map(|value| {
            uuid::Uuid::parse_str(value.as_str())
                .map(TenantId::from_uuid)
                .map_err(|error| AppError::Validation(format!("invalid {name}: {error}")))
        })
        .transpose()
}

pub(super) fn parse_env_u32(name: &str, default: u32) -> Result<u32, AppError> {
    match env::var(name) {
        Ok(value) => value.parse::<u32>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

pub(super) fn parse_env_usize(name: &str, default: usize) -> Result<usize, AppError> {
    match env::var(name) {
        Ok(value) => value.parse::<usize>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

pub(super) fn parse_env_u64(name: &str, default: u64) -> Result<u64, AppError> {
    match env::var(name) {
        Ok(value) => value.parse::<u64>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

pub(super) fn parse_env_i32(name: &str, default: i32) -> Result<i32, AppError> {
    match env::var(name) {
        Ok(value) => value.parse::<i32>().map_err(|error| {
            AppError::Validation(format!("invalid {name} value '{value}': {error}"))
        }),
        Err(_) => Ok(default),
    }
}

pub(super) fn parse_env_bool(name: &str, default: bool) -> Result<bool, AppError> {
    match env::var(name) {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "1" | "true" | "yes" | "on" => Ok(true),
                "0" | "false" | "no" | "off" => Ok(false),
                _ => Err(AppError::Validation(format!(
                    "invalid {name} value '{value}': expected boolean"
                ))),
            }
        }
        Err(_) => Ok(default),
    }
}
