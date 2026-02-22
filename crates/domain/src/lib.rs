//! Domain entities and invariants.

#![forbid(unsafe_code)]

mod app;
mod metadata;
mod security;
mod user;
mod workflow;

pub use app::{
    AppDefinition, AppEntityAction, AppEntityBinding, AppEntityRolePermission, AppEntityViewMode,
};
pub use metadata::{
    EntityDefinition, EntityFieldDefinition, FieldType, PublishedEntitySchema, RuntimeRecord,
};
pub use security::{AuditAction, Permission, Surface};
pub use user::{
    AuthTokenType, EmailAddress, PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH_WITH_MFA,
    PASSWORD_MIN_LENGTH_WITHOUT_MFA, RegistrationMode, UserId, validate_password,
};
pub use workflow::{
    WorkflowAction, WorkflowConditionOperator, WorkflowDefinition, WorkflowDefinitionInput,
    WorkflowStep, WorkflowTrigger,
};
