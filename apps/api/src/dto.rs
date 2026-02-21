use std::collections::BTreeMap;

use qryvanta_core::UserIdentity;
use qryvanta_domain::{
    AppDefinition, AppEntityBinding, AppEntityRolePermission, EntityDefinition,
    EntityFieldDefinition, PublishedEntitySchema, RegistrationMode, RuntimeRecord,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

/// Incoming payload for app creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-app-request.ts"
)]
pub struct CreateAppRequest {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
}

/// API representation of an app definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-response.ts"
)]
pub struct AppResponse {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
}

/// Incoming payload for binding an entity into app navigation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/bind-app-entity-request.ts"
)]
pub struct BindAppEntityRequest {
    pub entity_logical_name: String,
    pub navigation_label: Option<String>,
    pub navigation_order: i32,
}

/// API representation of an app entity navigation binding.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-entity-binding-response.ts"
)]
pub struct AppEntityBindingResponse {
    pub app_logical_name: String,
    pub entity_logical_name: String,
    pub navigation_label: Option<String>,
    pub navigation_order: i32,
}

/// Incoming payload for app role entity permission updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/save-app-role-entity-permission-request.ts"
)]
pub struct SaveAppRoleEntityPermissionRequest {
    pub role_name: String,
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// API representation of app-scoped role entity permissions.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-role-entity-permission-response.ts"
)]
pub struct AppRoleEntityPermissionResponse {
    pub app_logical_name: String,
    pub role_name: String,
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// API representation of effective app entity capabilities for the current subject.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-entity-capabilities-response.ts"
)]
pub struct AppEntityCapabilitiesResponse {
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// Incoming payload for metadata field create/update.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-field-request.ts"
)]
pub struct CreateFieldRequest {
    pub logical_name: String,
    pub display_name: String,
    pub field_type: String,
    pub is_required: bool,
    pub is_unique: bool,
    #[ts(type = "unknown | null")]
    pub default_value: Option<Value>,
    pub relation_target_entity: Option<String>,
}

/// API representation of a metadata field definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/field-response.ts"
)]
pub struct FieldResponse {
    pub entity_logical_name: String,
    pub logical_name: String,
    pub display_name: String,
    pub field_type: String,
    pub is_required: bool,
    pub is_unique: bool,
    #[ts(type = "unknown | null")]
    pub default_value: Option<Value>,
    pub relation_target_entity: Option<String>,
}

/// API representation of a published schema snapshot.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/published-schema-response.ts"
)]
pub struct PublishedSchemaResponse {
    pub entity_logical_name: String,
    pub entity_display_name: String,
    pub version: i32,
    pub fields: Vec<FieldResponse>,
}

/// Incoming runtime record create payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-runtime-record-request.ts"
)]
pub struct CreateRuntimeRecordRequest {
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}

/// Incoming runtime record update payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/update-runtime-record-request.ts"
)]
pub struct UpdateRuntimeRecordRequest {
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}

/// Incoming runtime record query payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/query-runtime-records-request.ts"
)]
pub struct QueryRuntimeRecordsRequest {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[ts(type = "Record<string, unknown> | null")]
    pub filters: Option<BTreeMap<String, Value>>,
}

/// API representation of a runtime record.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-record-response.ts"
)]
pub struct RuntimeRecordResponse {
    pub record_id: String,
    pub entity_logical_name: String,
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
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

/// Incoming payload for email/password registration.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/auth-register-request.ts"
)]
pub struct AuthRegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

/// Incoming payload for email/password login.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/auth-login-request.ts"
)]
pub struct AuthLoginRequest {
    pub email: String,
    pub password: String,
}

/// Auth status response for login and challenge flows.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/auth-login-response.ts"
)]
pub struct AuthLoginResponse {
    pub status: String,
    pub requires_totp: bool,
}

/// Incoming payload for TOTP or recovery code verification.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/auth-mfa-verify-request.ts"
)]
pub struct AuthMfaVerifyRequest {
    pub code: String,
    pub method: Option<String>,
}

/// Generic message response for auth flows.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/generic-message-response.ts"
)]
pub struct GenericMessageResponse {
    pub message: String,
}

/// Incoming payload for invite creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/invite-request.ts"
)]
pub struct InviteRequest {
    pub email: String,
    pub tenant_name: Option<String>,
}

/// Incoming payload for invite acceptance.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/accept-invite-request.ts"
)]
pub struct AcceptInviteRequest {
    pub token: String,
    pub password: Option<String>,
    pub display_name: Option<String>,
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

/// Incoming payload for tenant registration mode updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/update-tenant-registration-mode-request.ts"
)]
pub struct UpdateTenantRegistrationModeRequest {
    pub registration_mode: String,
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

impl From<EntityDefinition> for EntityResponse {
    fn from(entity: EntityDefinition) -> Self {
        Self {
            logical_name: entity.logical_name().as_str().to_owned(),
            display_name: entity.display_name().as_str().to_owned(),
        }
    }
}

impl From<AppDefinition> for AppResponse {
    fn from(value: AppDefinition) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            description: value.description().map(ToOwned::to_owned),
        }
    }
}

impl From<AppEntityBinding> for AppEntityBindingResponse {
    fn from(value: AppEntityBinding) -> Self {
        Self {
            app_logical_name: value.app_logical_name().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            navigation_label: value.navigation_label().map(ToOwned::to_owned),
            navigation_order: value.navigation_order(),
        }
    }
}

impl From<AppEntityRolePermission> for AppRoleEntityPermissionResponse {
    fn from(value: AppEntityRolePermission) -> Self {
        Self {
            app_logical_name: value.app_logical_name().as_str().to_owned(),
            role_name: value.role_name().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            can_read: value.can_read(),
            can_create: value.can_create(),
            can_update: value.can_update(),
            can_delete: value.can_delete(),
        }
    }
}

impl From<qryvanta_application::SubjectEntityPermission> for AppEntityCapabilitiesResponse {
    fn from(value: qryvanta_application::SubjectEntityPermission) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name,
            can_read: value.can_read,
            can_create: value.can_create,
            can_update: value.can_update,
            can_delete: value.can_delete,
        }
    }
}

impl From<EntityFieldDefinition> for FieldResponse {
    fn from(value: EntityFieldDefinition) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            field_type: value.field_type().as_str().to_owned(),
            is_required: value.is_required(),
            is_unique: value.is_unique(),
            default_value: value.default_value().cloned(),
            relation_target_entity: value
                .relation_target_entity()
                .map(|target| target.as_str().to_owned()),
        }
    }
}

impl From<PublishedEntitySchema> for PublishedSchemaResponse {
    fn from(value: PublishedEntitySchema) -> Self {
        Self {
            entity_logical_name: value.entity().logical_name().as_str().to_owned(),
            entity_display_name: value.entity().display_name().as_str().to_owned(),
            version: value.version(),
            fields: value
                .fields()
                .iter()
                .cloned()
                .map(FieldResponse::from)
                .collect(),
        }
    }
}

impl From<RuntimeRecord> for RuntimeRecordResponse {
    fn from(value: RuntimeRecord) -> Self {
        Self {
            record_id: value.record_id().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            data: value.data().clone(),
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

impl From<RegistrationMode> for TenantRegistrationModeResponse {
    fn from(value: RegistrationMode) -> Self {
        Self {
            registration_mode: value.as_str().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AcceptInviteRequest, AppEntityBindingResponse, AppEntityCapabilitiesResponse, AppResponse,
        AppRoleEntityPermissionResponse, AssignRoleRequest, AuditLogEntryResponse,
        AuthLoginRequest, AuthLoginResponse, AuthMfaVerifyRequest, AuthRegisterRequest,
        BindAppEntityRequest, CreateAppRequest, CreateEntityRequest, CreateFieldRequest,
        CreateRoleRequest, CreateRuntimeRecordRequest, EntityResponse, FieldResponse,
        GenericMessageResponse, HealthResponse, InviteRequest, PublishedSchemaResponse,
        QueryRuntimeRecordsRequest, RemoveRoleAssignmentRequest, RoleAssignmentResponse,
        RoleResponse, RuntimeRecordResponse, SaveAppRoleEntityPermissionRequest,
        TenantRegistrationModeResponse, UpdateRuntimeRecordRequest,
        UpdateTenantRegistrationModeRequest, UserIdentityResponse,
    };

    use crate::error::ErrorResponse;
    use ts_rs::Config;
    use ts_rs::TS;

    #[test]
    fn export_ts_bindings() -> Result<(), ts_rs::ExportError> {
        let config = Config::default();

        CreateEntityRequest::export(&config)?;
        CreateAppRequest::export(&config)?;
        BindAppEntityRequest::export(&config)?;
        SaveAppRoleEntityPermissionRequest::export(&config)?;
        CreateFieldRequest::export(&config)?;
        CreateRoleRequest::export(&config)?;
        CreateRuntimeRecordRequest::export(&config)?;
        AssignRoleRequest::export(&config)?;
        RemoveRoleAssignmentRequest::export(&config)?;
        UpdateTenantRegistrationModeRequest::export(&config)?;
        UpdateRuntimeRecordRequest::export(&config)?;
        QueryRuntimeRecordsRequest::export(&config)?;
        EntityResponse::export(&config)?;
        AppResponse::export(&config)?;
        AppEntityBindingResponse::export(&config)?;
        AppEntityCapabilitiesResponse::export(&config)?;
        AppRoleEntityPermissionResponse::export(&config)?;
        FieldResponse::export(&config)?;
        PublishedSchemaResponse::export(&config)?;
        RuntimeRecordResponse::export(&config)?;
        RoleResponse::export(&config)?;
        RoleAssignmentResponse::export(&config)?;
        TenantRegistrationModeResponse::export(&config)?;
        AuditLogEntryResponse::export(&config)?;
        ErrorResponse::export(&config)?;
        HealthResponse::export(&config)?;
        UserIdentityResponse::export(&config)?;
        AuthRegisterRequest::export(&config)?;
        AuthLoginRequest::export(&config)?;
        AuthLoginResponse::export(&config)?;
        AuthMfaVerifyRequest::export(&config)?;
        GenericMessageResponse::export(&config)?;
        InviteRequest::export(&config)?;
        AcceptInviteRequest::export(&config)?;

        Ok(())
    }
}
