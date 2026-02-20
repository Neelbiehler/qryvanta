//! Application services and ports.

#![forbid(unsafe_code)]

mod auth_event_service;
mod authorization_service;
mod metadata_service;
mod security_admin_service;

pub use auth_event_service::{AuthEvent, AuthEventRepository, AuthEventService};
pub use authorization_service::{AuthorizationRepository, AuthorizationService};
pub use metadata_service::{
    AuditEvent, AuditRepository, MetadataRepository, MetadataService, TenantRepository,
};
pub use security_admin_service::{
    AuditLogEntry, AuditLogQuery, AuditLogRepository, CreateRoleInput, RoleAssignment,
    RoleDefinition, SecurityAdminRepository, SecurityAdminService,
};
