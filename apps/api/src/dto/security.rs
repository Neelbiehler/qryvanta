use qryvanta_domain::RegistrationMode;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Incoming payload for custom role creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-role-request.ts"
)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

/// Incoming payload for role assignment.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/assign-role-request.ts"
)]
pub struct AssignRoleRequest {
    pub subject: String,
    pub role_name: String,
}

/// Incoming payload for role unassignment.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/remove-role-assignment-request.ts"
)]
pub struct RemoveRoleAssignmentRequest {
    pub subject: String,
    pub role_name: String,
}

/// Incoming payload for tenant registration mode updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/update-tenant-registration-mode-request.ts"
)]
pub struct UpdateTenantRegistrationModeRequest {
    pub registration_mode: String,
}

/// Incoming payload for runtime subject field permission updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/save-runtime-field-permissions-request.ts"
)]
pub struct SaveRuntimeFieldPermissionsRequest {
    pub subject: String,
    pub entity_logical_name: String,
    pub fields: Vec<RuntimeFieldPermissionInputRequest>,
}

/// Incoming runtime field permission item.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-field-permission-input-request.ts"
)]
pub struct RuntimeFieldPermissionInputRequest {
    pub field_logical_name: String,
    pub can_read: bool,
    pub can_write: bool,
}

/// Incoming payload for creating temporary access grants.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-temporary-access-grant-request.ts"
)]
pub struct CreateTemporaryAccessGrantRequest {
    pub subject: String,
    pub permissions: Vec<String>,
    pub reason: String,
    pub duration_minutes: u32,
}

/// Incoming payload for temporary access grant revocation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/revoke-temporary-access-grant-request.ts"
)]
pub struct RevokeTemporaryAccessGrantRequest {
    pub revoke_reason: Option<String>,
}

/// Incoming payload for audit retention updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/update-audit-retention-policy-request.ts"
)]
pub struct UpdateAuditRetentionPolicyRequest {
    pub retention_days: u16,
}

/// API representation of an RBAC role.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/role-response.ts"
)]
pub struct RoleResponse {
    pub role_id: String,
    pub name: String,
    pub is_system: bool,
    pub permissions: Vec<String>,
}

/// API representation of an audit log entry.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/audit-log-entry-response.ts"
)]
pub struct AuditLogEntryResponse {
    pub event_id: String,
    pub subject: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub detail: Option<String>,
    pub created_at: String,
}

/// API representation of a role assignment.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/role-assignment-response.ts"
)]
pub struct RoleAssignmentResponse {
    pub subject: String,
    pub role_id: String,
    pub role_name: String,
    pub assigned_at: String,
}

/// API representation of tenant registration mode.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/tenant-registration-mode-response.ts"
)]
pub struct TenantRegistrationModeResponse {
    pub registration_mode: String,
}

/// API representation of runtime field permission entry.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-field-permission-response.ts"
)]
pub struct RuntimeFieldPermissionResponse {
    pub subject: String,
    pub entity_logical_name: String,
    pub field_logical_name: String,
    pub can_read: bool,
    pub can_write: bool,
    pub updated_at: String,
}

/// API representation of temporary access grant.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/temporary-access-grant-response.ts"
)]
pub struct TemporaryAccessGrantResponse {
    pub grant_id: String,
    pub subject: String,
    pub permissions: Vec<String>,
    pub reason: String,
    pub created_by_subject: String,
    pub expires_at: String,
    pub revoked_at: Option<String>,
}

/// API representation of audit retention policy.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/audit-retention-policy-response.ts"
)]
pub struct AuditRetentionPolicyResponse {
    pub retention_days: u16,
}

/// API representation of audit purge operation result.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/audit-purge-result-response.ts"
)]
pub struct AuditPurgeResultResponse {
    pub deleted_count: u64,
    pub retention_days: u16,
}

impl From<qryvanta_application::RoleDefinition> for RoleResponse {
    fn from(value: qryvanta_application::RoleDefinition) -> Self {
        Self {
            role_id: value.role_id,
            name: value.name,
            is_system: value.is_system,
            permissions: value
                .permissions
                .into_iter()
                .map(|permission| permission.as_str().to_owned())
                .collect(),
        }
    }
}

impl From<qryvanta_application::AuditLogEntry> for AuditLogEntryResponse {
    fn from(value: qryvanta_application::AuditLogEntry) -> Self {
        Self {
            event_id: value.event_id,
            subject: value.subject,
            action: value.action,
            resource_type: value.resource_type,
            resource_id: value.resource_id,
            detail: value.detail,
            created_at: value.created_at,
        }
    }
}

impl From<qryvanta_application::RoleAssignment> for RoleAssignmentResponse {
    fn from(value: qryvanta_application::RoleAssignment) -> Self {
        Self {
            subject: value.subject,
            role_id: value.role_id,
            role_name: value.role_name,
            assigned_at: value.assigned_at,
        }
    }
}

impl From<RegistrationMode> for TenantRegistrationModeResponse {
    fn from(value: RegistrationMode) -> Self {
        Self {
            registration_mode: value.as_str().to_owned(),
        }
    }
}

impl From<qryvanta_application::RuntimeFieldPermissionEntry> for RuntimeFieldPermissionResponse {
    fn from(value: qryvanta_application::RuntimeFieldPermissionEntry) -> Self {
        Self {
            subject: value.subject,
            entity_logical_name: value.entity_logical_name,
            field_logical_name: value.field_logical_name,
            can_read: value.can_read,
            can_write: value.can_write,
            updated_at: value.updated_at,
        }
    }
}

impl From<qryvanta_application::TemporaryAccessGrant> for TemporaryAccessGrantResponse {
    fn from(value: qryvanta_application::TemporaryAccessGrant) -> Self {
        Self {
            grant_id: value.grant_id,
            subject: value.subject,
            permissions: value
                .permissions
                .into_iter()
                .map(|permission| permission.as_str().to_owned())
                .collect(),
            reason: value.reason,
            created_by_subject: value.created_by_subject,
            expires_at: value.expires_at,
            revoked_at: value.revoked_at,
        }
    }
}

impl From<qryvanta_application::AuditRetentionPolicy> for AuditRetentionPolicyResponse {
    fn from(value: qryvanta_application::AuditRetentionPolicy) -> Self {
        Self {
            retention_days: value.retention_days,
        }
    }
}

impl From<qryvanta_application::AuditPurgeResult> for AuditPurgeResultResponse {
    fn from(value: qryvanta_application::AuditPurgeResult) -> Self {
        Self {
            deleted_count: value.deleted_count,
            retention_days: value.retention_days,
        }
    }
}
