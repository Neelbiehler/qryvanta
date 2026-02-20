//! Shared primitives for all Rust crates in Qryvanta.

#![forbid(unsafe_code)]

/// Authentication primitives shared across services.
pub mod auth;

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub use auth::UserIdentity;

/// Result type used across Qryvanta crates.
pub type AppResult<T> = Result<T, AppError>;

/// A validated non-empty UTF-8 string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Creates a validated non-empty string.
    pub fn new(value: impl Into<String>) -> AppResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AppError::Validation(
                "value must not be empty or whitespace".to_owned(),
            ));
        }

        Ok(Self(value))
    }

    /// Returns the underlying string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
    }
}

/// Tenant identifier used as the partition key for every persisted resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(Uuid);

impl TenantId {
    /// Creates a random tenant identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a tenant identifier from an existing UUID value.
    #[must_use]
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TenantId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Common application error categories.
#[derive(Debug, Error)]
pub enum AppError {
    /// Invalid input or violated invariant.
    #[error("validation error: {0}")]
    Validation(String),

    /// Requested resource does not exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// Write operation conflicts with existing state.
    #[error("conflict: {0}")]
    Conflict(String),

    /// User is not authenticated or not allowed to access a resource.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// User is authenticated but blocked by authorization policy.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Internal unexpected error.
    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::{NonEmptyString, TenantId};

    #[test]
    fn non_empty_string_rejects_whitespace() {
        let result = NonEmptyString::new("   ");
        assert!(result.is_err());
    }

    #[test]
    fn tenant_id_formats_as_uuid() {
        let tenant_id = TenantId::new();
        assert_eq!(tenant_id.to_string().len(), 36);
    }
}
