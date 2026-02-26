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
mod security_admin_ports;
mod security_admin_service;
mod user_service;
mod workflow_ports;
mod workflow_service;

pub use app_ports::{
    AppEntityFormInput, AppEntityViewInput, AppRepository, BindAppEntityInput, CreateAppInput,
    RuntimeRecordService, SaveAppRoleEntityPermissionInput, SaveAppSitemapInput,
    SubjectEntityPermission,
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
    AuditEvent, AuditRepository, MetadataComponentsRepository, MetadataDefinitionsRepository,
    MetadataPublishRepository, MetadataRepository, MetadataRepositoryByConcern,
    MetadataRuntimeRepository, RecordListQuery, RuntimeRecordConditionGroup,
    RuntimeRecordConditionNode, RuntimeRecordFilter, RuntimeRecordJoinType, RuntimeRecordLink,
    RuntimeRecordLogicalMode, RuntimeRecordOperator, RuntimeRecordQuery, RuntimeRecordSort,
    RuntimeRecordSortDirection, SaveBusinessRuleInput, SaveFieldInput, SaveFormInput,
    SaveOptionSetInput, SaveViewInput, TenantRepository, UniqueFieldValue, UpdateEntityInput,
    UpdateFieldInput,
};
pub use metadata_service::MetadataService;
pub use mfa_service::{MfaService, SecretEncryptor, TotpEnrollment, TotpProvider};
pub use rate_limit_service::{AttemptInfo, RateLimitRepository, RateLimitRule, RateLimitService};
pub use security_admin_ports::{
    AuditLogEntry, AuditLogQuery, AuditLogRepository, AuditPurgeResult, AuditRetentionPolicy,
    CreateRoleInput, CreateTemporaryAccessGrantInput, RoleAssignment, RoleDefinition,
    RuntimeFieldPermissionEntry, RuntimeFieldPermissionInput, SaveRuntimeFieldPermissionsInput,
    SecurityAdminRepository, TemporaryAccessGrant, TemporaryAccessGrantQuery,
    WorkspacePublishRunAuditInput,
};
pub use security_admin_service::SecurityAdminService;
pub use user_service::{
    AuthOutcome, PasswordHasher, RegisterParams, UserRecord, UserRepository, UserService,
};
pub use workflow_ports::{
    ClaimedWorkflowJob, CompleteWorkflowRunInput, CreateWorkflowRunInput, SaveWorkflowInput,
    WorkflowClaimPartition, WorkflowExecutionMode, WorkflowQueueStats, WorkflowQueueStatsCache,
    WorkflowQueueStatsQuery, WorkflowRepository, WorkflowRun, WorkflowRunAttempt,
    WorkflowRunAttemptStatus, WorkflowRunListQuery, WorkflowRunStatus,
    WorkflowRuntimeRecordService, WorkflowWorkerHeartbeatInput, WorkflowWorkerLease,
    WorkflowWorkerLeaseCoordinator,
};
pub use workflow_service::WorkflowService;
