mod apps;
mod auth;
mod common;
mod entities;
pub(crate) mod runtime;
mod security;

pub use apps::{
    AppEntityBindingResponse, AppEntityCapabilitiesResponse, AppResponse,
    AppRoleEntityPermissionResponse, BindAppEntityRequest, CreateAppRequest,
    SaveAppRoleEntityPermissionRequest,
};
pub use auth::{
    AcceptInviteRequest, AuthLoginRequest, AuthLoginResponse, AuthMfaVerifyRequest,
    AuthRegisterRequest, InviteRequest,
};
pub use common::{GenericMessageResponse, HealthResponse, UserIdentityResponse};
pub use entities::{
    CreateEntityRequest, CreateFieldRequest, EntityResponse, FieldResponse, PublishedSchemaResponse,
};
pub use runtime::{
    CreateRuntimeRecordRequest, QueryRuntimeRecordsRequest, RuntimeRecordQueryFilterRequest,
    RuntimeRecordQueryGroupRequest, RuntimeRecordQueryLinkEntityRequest, RuntimeRecordResponse,
    UpdateRuntimeRecordRequest,
};
pub use security::{
    AssignRoleRequest, AuditLogEntryResponse, AuditPurgeResultResponse,
    AuditRetentionPolicyResponse, CreateRoleRequest, CreateTemporaryAccessGrantRequest,
    RemoveRoleAssignmentRequest, RevokeTemporaryAccessGrantRequest, RoleAssignmentResponse,
    RoleResponse, RuntimeFieldPermissionResponse, SaveRuntimeFieldPermissionsRequest,
    TemporaryAccessGrantResponse, TenantRegistrationModeResponse,
    UpdateAuditRetentionPolicyRequest, UpdateTenantRegistrationModeRequest,
};

#[cfg(test)]
mod tests {
    use super::{
        AcceptInviteRequest, AppEntityBindingResponse, AppEntityCapabilitiesResponse, AppResponse,
        AppRoleEntityPermissionResponse, AssignRoleRequest, AuditLogEntryResponse,
        AuditPurgeResultResponse, AuditRetentionPolicyResponse, AuthLoginRequest,
        AuthLoginResponse, AuthMfaVerifyRequest, AuthRegisterRequest, BindAppEntityRequest,
        CreateAppRequest, CreateEntityRequest, CreateFieldRequest, CreateRoleRequest,
        CreateRuntimeRecordRequest, CreateTemporaryAccessGrantRequest, EntityResponse,
        FieldResponse, GenericMessageResponse, HealthResponse, InviteRequest,
        PublishedSchemaResponse, QueryRuntimeRecordsRequest, RemoveRoleAssignmentRequest,
        RevokeTemporaryAccessGrantRequest, RoleAssignmentResponse, RoleResponse,
        RuntimeFieldPermissionResponse, RuntimeRecordResponse, SaveAppRoleEntityPermissionRequest,
        SaveRuntimeFieldPermissionsRequest, TemporaryAccessGrantResponse,
        TenantRegistrationModeResponse, UpdateAuditRetentionPolicyRequest,
        UpdateRuntimeRecordRequest, UpdateTenantRegistrationModeRequest, UserIdentityResponse,
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
        super::security::RuntimeFieldPermissionInputRequest::export(&config)?;
        SaveRuntimeFieldPermissionsRequest::export(&config)?;
        CreateTemporaryAccessGrantRequest::export(&config)?;
        RevokeTemporaryAccessGrantRequest::export(&config)?;
        UpdateAuditRetentionPolicyRequest::export(&config)?;
        UpdateRuntimeRecordRequest::export(&config)?;
        super::runtime::RuntimeRecordQueryFilterRequest::export(&config)?;
        super::runtime::RuntimeRecordQueryGroupRequest::export(&config)?;
        super::runtime::RuntimeRecordQueryLinkEntityRequest::export(&config)?;
        super::runtime::RuntimeRecordQuerySortRequest::export(&config)?;
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
        RuntimeFieldPermissionResponse::export(&config)?;
        TemporaryAccessGrantResponse::export(&config)?;
        AuditRetentionPolicyResponse::export(&config)?;
        AuditPurgeResultResponse::export(&config)?;
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
