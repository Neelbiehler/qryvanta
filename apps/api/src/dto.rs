use qryvanta_core::UserIdentity;
use qryvanta_domain::EntityDefinition;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Health response payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/health-response.ts"
)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// Incoming payload for entity creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-entity-request.ts"
)]
pub struct CreateEntityRequest {
    pub logical_name: String,
    pub display_name: String,
}

/// API representation of an entity.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/entity-response.ts"
)]
pub struct EntityResponse {
    pub logical_name: String,
    pub display_name: String,
}

/// API representation of the authenticated user.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/user-identity-response.ts"
)]
pub struct UserIdentityResponse {
    pub subject: String,
    pub display_name: String,
    pub email: Option<String>,
    pub tenant_id: String,
}

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

impl From<EntityDefinition> for EntityResponse {
    fn from(entity: EntityDefinition) -> Self {
        Self {
            logical_name: entity.logical_name().as_str().to_owned(),
            display_name: entity.display_name().as_str().to_owned(),
        }
    }
}

impl From<UserIdentity> for UserIdentityResponse {
    fn from(identity: UserIdentity) -> Self {
        Self {
            subject: identity.subject().to_owned(),
            display_name: identity.display_name().to_owned(),
            email: identity.email().map(ToOwned::to_owned),
            tenant_id: identity.tenant_id().to_string(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{
        AssignRoleRequest, AuditLogEntryResponse, CreateEntityRequest, CreateRoleRequest,
        EntityResponse, HealthResponse, RemoveRoleAssignmentRequest, RoleAssignmentResponse,
        RoleResponse, UserIdentityResponse,
    };

    use crate::error::ErrorResponse;
    use ts_rs::Config;
    use ts_rs::TS;

    #[test]
    fn export_ts_bindings() -> Result<(), ts_rs::ExportError> {
        let config = Config::default();

        CreateEntityRequest::export(&config)?;
        CreateRoleRequest::export(&config)?;
        AssignRoleRequest::export(&config)?;
        RemoveRoleAssignmentRequest::export(&config)?;
        EntityResponse::export(&config)?;
        RoleResponse::export(&config)?;
        RoleAssignmentResponse::export(&config)?;
        AuditLogEntryResponse::export(&config)?;
        ErrorResponse::export(&config)?;
        HealthResponse::export(&config)?;
        UserIdentityResponse::export(&config)?;

        Ok(())
    }
}
