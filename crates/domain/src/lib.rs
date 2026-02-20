//! Domain entities and invariants.

#![forbid(unsafe_code)]

mod metadata;
mod security;
mod user;

pub use metadata::EntityDefinition;
pub use security::{AuditAction, Permission};
pub use user::{
    AuthTokenType, EmailAddress, PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH_WITH_MFA,
    PASSWORD_MIN_LENGTH_WITHOUT_MFA, RegistrationMode, UserId, validate_password,
};
