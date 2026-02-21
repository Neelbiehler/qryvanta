//! Domain entities and invariants.

#![forbid(unsafe_code)]

mod app;
mod metadata;
mod security;
mod user;

pub use app::{AppDefinition, AppEntityAction, AppEntityBinding, AppEntityRolePermission};
pub use metadata::{
    EntityDefinition, EntityFieldDefinition, FieldType, PublishedEntitySchema, RuntimeRecord,
};
pub use security::{AuditAction, Permission};
pub use user::{
    AuthTokenType, EmailAddress, PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH_WITH_MFA,
    PASSWORD_MIN_LENGTH_WITHOUT_MFA, RegistrationMode, UserId, validate_password,
};
