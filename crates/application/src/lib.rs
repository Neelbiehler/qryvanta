//! Application services and ports.

#![forbid(unsafe_code)]

mod app_ports;
mod app_service;
mod auth_event_service;
mod auth_token_service;
mod authorization_service;
mod contact_bootstrap_service;
mod metadata_ports;
mod metadata_service;
mod mfa_service;
mod rate_limit_service;
mod security_admin_service;
mod user_service;

pub use app_ports::{
    AppRepository, BindAppEntityInput, CreateAppInput, RuntimeRecordService,
    SaveAppRoleEntityPermissionInput, SubjectEntityPermission,
};
pub use app_service::AppService;
pub use auth_event_service::{AuthEvent, AuthEventRepository, AuthEventService};
pub use auth_token_service::{
    AuthTokenRecord, AuthTokenRepository, AuthTokenService, EmailService,
};
pub use authorization_service::{
    AuthorizationRepository, AuthorizationService, RuntimeFieldAccess, RuntimeFieldGrant,
    TemporaryPermissionGrant,
};
pub use contact_bootstrap_service::ContactBootstrapService;
pub use metadata_ports::{
    AuditEvent, AuditRepository, MetadataRepository, RecordListQuery, RuntimeRecordConditionGroup,
    RuntimeRecordConditionNode, RuntimeRecordFilter, RuntimeRecordJoinType, RuntimeRecordLink,
    RuntimeRecordLogicalMode, RuntimeRecordOperator, RuntimeRecordQuery, RuntimeRecordSort,
    RuntimeRecordSortDirection, SaveFieldInput, TenantRepository, UniqueFieldValue,
};
pub use metadata_service::MetadataService;
pub use mfa_service::{MfaService, SecretEncryptor, TotpEnrollment, TotpProvider};
pub use rate_limit_service::{AttemptInfo, RateLimitRepository, RateLimitRule, RateLimitService};
pub use security_admin_service::{
    AuditLogEntry, AuditLogQuery, AuditLogRepository, AuditPurgeResult, AuditRetentionPolicy,
    CreateRoleInput, CreateTemporaryAccessGrantInput, RoleAssignment, RoleDefinition,
    RuntimeFieldPermissionEntry, RuntimeFieldPermissionInput, SaveRuntimeFieldPermissionsInput,
    SecurityAdminRepository, SecurityAdminService, TemporaryAccessGrant, TemporaryAccessGrantQuery,
};
pub use user_service::{
    AuthOutcome, PasswordHasher, RegisterParams, UserRecord, UserRepository, UserService,
};
