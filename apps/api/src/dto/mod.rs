mod apps;
mod auth;
mod common;
mod entities;
mod runtime;
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
    CreateRuntimeRecordRequest, QueryRuntimeRecordsRequest, RuntimeRecordResponse,
    UpdateRuntimeRecordRequest,
};
pub use security::{
    AssignRoleRequest, AuditLogEntryResponse, CreateRoleRequest, RemoveRoleAssignmentRequest,
    RoleAssignmentResponse, RoleResponse, TenantRegistrationModeResponse,
    UpdateTenantRegistrationModeRequest,
};

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
