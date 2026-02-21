use qryvanta_domain::RegistrationMode;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Incoming payload for custom role creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/create-role-request.ts"
)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

/// Incoming payload for role assignment.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/assign-role-request.ts"
)]
pub struct AssignRoleRequest {
    pub subject: String,
    pub role_name: String,
}

/// Incoming payload for role unassignment.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/remove-role-assignment-request.ts"
)]
pub struct RemoveRoleAssignmentRequest {
    pub subject: String,
    pub role_name: String,
}

/// Incoming payload for tenant registration mode updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/update-tenant-registration-mode-request.ts"
)]
pub struct UpdateTenantRegistrationModeRequest {
    pub registration_mode: String,
}

/// API representation of an RBAC role.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/role-response.ts"
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
    export_to = "../../../../packages/api-types/src/generated/audit-log-entry-response.ts"
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
    export_to = "../../../../packages/api-types/src/generated/role-assignment-response.ts"
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
    export_to = "../../../../packages/api-types/src/generated/tenant-registration-mode-response.ts"
)]
pub struct TenantRegistrationModeResponse {
    pub registration_mode: String,
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
