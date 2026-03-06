use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;

use qryvanta_core::UserIdentity;
use qryvanta_domain::{Permission, RegistrationMode};
use tower_sessions::Session;

use crate::auth::session_helpers::require_recent_step_up;
use crate::dto::{
    AssignRoleRequest, AuditIntegrityStatusResponse, AuditLogEntryResponse,
    AuditPurgeResultResponse, AuditRetentionPolicyResponse, CreateRoleRequest,
    CreateTemporaryAccessGrantRequest, RemoveRoleAssignmentRequest,
    RevokeTemporaryAccessGrantRequest, RoleAssignmentResponse, RoleResponse,
    RuntimeFieldPermissionResponse, SaveRuntimeFieldPermissionsRequest,
    TemporaryAccessGrantResponse, TenantRegistrationModeResponse,
    UpdateAuditRetentionPolicyRequest, UpdateTenantRegistrationModeRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

mod audit;
mod governance;
mod roles;
mod runtime_permissions;
mod temporary_access;

pub use audit::{
    export_audit_log_handler, list_audit_log_handler, purge_audit_log_handler,
    verify_audit_log_integrity_handler,
};
pub use governance::{
    audit_retention_policy_handler, registration_mode_handler,
    update_audit_retention_policy_handler, update_registration_mode_handler,
};
pub use roles::{
    assign_role_handler, create_role_handler, list_role_assignments_handler, list_roles_handler,
    unassign_role_handler,
};
pub use runtime_permissions::{
    list_runtime_field_permissions_handler, save_runtime_field_permissions_handler,
};
pub use temporary_access::{
    create_temporary_access_grant_handler, list_temporary_access_grants_handler,
    revoke_temporary_access_grant_handler,
};
