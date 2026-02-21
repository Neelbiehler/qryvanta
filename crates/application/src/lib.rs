//! Application services and ports.

#![forbid(unsafe_code)]

mod app_service;
mod auth_event_service;
mod auth_token_service;
mod authorization_service;
mod metadata_service;
mod mfa_service;
mod rate_limit_service;
mod security_admin_service;
mod user_service;

pub use app_service::{
    AppRepository, AppService, BindAppEntityInput, CreateAppInput, RuntimeRecordService,
    SaveAppRoleEntityPermissionInput, SubjectEntityPermission,
};
pub use auth_event_service::{AuthEvent, AuthEventRepository, AuthEventService};
pub use auth_token_service::{
    AuthTokenRecord, AuthTokenRepository, AuthTokenService, EmailService,
};
pub use authorization_service::{AuthorizationRepository, AuthorizationService};
pub use metadata_service::{
    AuditEvent, AuditRepository, MetadataRepository, MetadataService, RecordListQuery,
    SaveFieldInput, TenantRepository, UniqueFieldValue,
};
pub use mfa_service::{MfaService, SecretEncryptor, TotpEnrollment, TotpProvider};
pub use rate_limit_service::{AttemptInfo, RateLimitRepository, RateLimitRule, RateLimitService};
pub use security_admin_service::{
    AuditLogEntry, AuditLogQuery, AuditLogRepository, CreateRoleInput, RoleAssignment,
    RoleDefinition, SecurityAdminRepository, SecurityAdminService,
};
pub use user_service::{
    AuthOutcome, PasswordHasher, RegisterParams, UserRecord, UserRepository, UserService,
};
