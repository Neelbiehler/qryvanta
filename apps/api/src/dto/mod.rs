mod apps;
mod auth;
mod common;
mod entities;
mod publish;
pub(crate) mod runtime;
mod security;
mod workflows;

pub use apps::{
    AppEntityBindingResponse, AppEntityCapabilitiesResponse, AppPublishChecksResponse, AppResponse,
    AppRoleEntityPermissionResponse, AppSitemapAreaDto, AppSitemapGroupDto, AppSitemapResponse,
    AppSitemapSubAreaDto, AppSitemapTargetDto, BindAppEntityRequest, CreateAppRequest,
    SaveAppRoleEntityPermissionRequest, SaveAppSitemapRequest, WorkspaceDashboardResponse,
};
pub use auth::{
    AcceptInviteRequest, AuthLoginRequest, AuthLoginResponse, AuthMfaVerifyRequest,
    AuthRegisterRequest, InviteRequest,
};
pub use common::{
    GenericMessageResponse, HealthDependencyStatus, HealthResponse, UserIdentityResponse,
};
pub use entities::{
    BusinessRuleResponse, CreateBusinessRuleRequest, CreateEntityRequest, CreateFieldRequest,
    CreateFormRequest, CreateOptionSetRequest, CreateViewRequest, EntityResponse, FieldResponse,
    FormResponse, OptionSetResponse, PublishChecksResponse, PublishedSchemaResponse,
    UpdateEntityRequest, UpdateFieldRequest, ViewResponse,
};
pub use publish::{
    AppBindingDiffResponse, AppPublishDiffResponse, EntityPublishDiffResponse,
    PublishCheckCategoryDto, PublishCheckIssueResponse, PublishCheckScopeDto,
    PublishCheckSeverityDto, PublishFieldDiffItemResponse, PublishSurfaceDeltaItemResponse,
    PublishSurfaceDiffItemResponse, RunWorkspacePublishRequest, RunWorkspacePublishResponse,
    WorkspacePublishChecksResponse, WorkspacePublishDiffRequest, WorkspacePublishDiffResponse,
    WorkspacePublishHistoryEntryResponse,
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
pub use workflows::{
    ExecuteWorkflowRequest, SaveWorkflowRequest, WorkflowResponse, WorkflowRunAttemptResponse,
    WorkflowRunResponse,
};

#[cfg(test)]
mod tests {
    use super::apps::{
        AppEntityFormDto, AppEntityViewDto, ChartAggregationDto, ChartResponse, ChartTypeDto,
        DashboardWidgetResponse,
    };
    use super::common::HealthDependencyStatus;
    use super::{
        AcceptInviteRequest, AppEntityBindingResponse, AppEntityCapabilitiesResponse,
        AppPublishChecksResponse, AppResponse, AppRoleEntityPermissionResponse, AppSitemapAreaDto,
        AppSitemapGroupDto, AppSitemapResponse, AppSitemapSubAreaDto, AppSitemapTargetDto,
        AssignRoleRequest, AuditLogEntryResponse, AuditPurgeResultResponse,
        AuditRetentionPolicyResponse, AuthLoginRequest, AuthLoginResponse, AuthMfaVerifyRequest,
        AuthRegisterRequest, BindAppEntityRequest, BusinessRuleResponse, CreateAppRequest,
        CreateBusinessRuleRequest, CreateEntityRequest, CreateFieldRequest, CreateFormRequest,
        CreateOptionSetRequest, CreateRoleRequest, CreateRuntimeRecordRequest,
        CreateTemporaryAccessGrantRequest, CreateViewRequest, EntityResponse,
        ExecuteWorkflowRequest, FieldResponse, FormResponse, GenericMessageResponse,
        HealthResponse, InviteRequest, OptionSetResponse, PublishCheckCategoryDto,
        PublishCheckIssueResponse, PublishCheckScopeDto, PublishCheckSeverityDto,
        PublishChecksResponse, PublishSurfaceDeltaItemResponse, PublishedSchemaResponse,
        QueryRuntimeRecordsRequest, RemoveRoleAssignmentRequest, RevokeTemporaryAccessGrantRequest,
        RoleAssignmentResponse, RoleResponse, RunWorkspacePublishRequest,
        RunWorkspacePublishResponse, RuntimeFieldPermissionResponse, RuntimeRecordResponse,
        SaveAppRoleEntityPermissionRequest, SaveAppSitemapRequest,
        SaveRuntimeFieldPermissionsRequest, SaveWorkflowRequest, TemporaryAccessGrantResponse,
        TenantRegistrationModeResponse, UpdateAuditRetentionPolicyRequest, UpdateEntityRequest,
        UpdateFieldRequest, UpdateRuntimeRecordRequest, UpdateTenantRegistrationModeRequest,
        UserIdentityResponse, ViewResponse, WorkflowResponse, WorkflowRunAttemptResponse,
        WorkflowRunResponse, WorkspaceDashboardResponse, WorkspacePublishChecksResponse,
        WorkspacePublishDiffRequest, WorkspacePublishDiffResponse,
        WorkspacePublishHistoryEntryResponse,
    };

    use crate::error::ErrorResponse;
    use ts_rs::Config;
    use ts_rs::TS;

    #[test]
    fn export_ts_bindings() -> Result<(), ts_rs::ExportError> {
        let config = Config::default();

        CreateEntityRequest::export(&config)?;
        CreateAppRequest::export(&config)?;
        SaveAppSitemapRequest::export(&config)?;
        BindAppEntityRequest::export(&config)?;
        SaveAppRoleEntityPermissionRequest::export(&config)?;
        SaveWorkflowRequest::export(&config)?;
        super::workflows::WorkflowConditionOperatorDto::export(&config)?;
        super::workflows::WorkflowStepDto::export(&config)?;
        ExecuteWorkflowRequest::export(&config)?;
        CreateFieldRequest::export(&config)?;
        CreateBusinessRuleRequest::export(&config)?;
        CreateFormRequest::export(&config)?;
        CreateOptionSetRequest::export(&config)?;
        CreateViewRequest::export(&config)?;
        super::entities::OptionSetItemDto::export(&config)?;
        OptionSetResponse::export(&config)?;
        PublishChecksResponse::export(&config)?;
        UpdateEntityRequest::export(&config)?;
        UpdateFieldRequest::export(&config)?;
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
        AppSitemapResponse::export(&config)?;
        AppSitemapAreaDto::export(&config)?;
        AppSitemapGroupDto::export(&config)?;
        AppSitemapSubAreaDto::export(&config)?;
        AppSitemapTargetDto::export(&config)?;
        WorkspaceDashboardResponse::export(&config)?;
        DashboardWidgetResponse::export(&config)?;
        ChartResponse::export(&config)?;
        ChartTypeDto::export(&config)?;
        ChartAggregationDto::export(&config)?;
        AppPublishChecksResponse::export(&config)?;
        PublishCheckSeverityDto::export(&config)?;
        PublishCheckScopeDto::export(&config)?;
        PublishCheckCategoryDto::export(&config)?;
        PublishCheckIssueResponse::export(&config)?;
        PublishSurfaceDeltaItemResponse::export(&config)?;
        WorkspacePublishChecksResponse::export(&config)?;
        WorkspacePublishDiffRequest::export(&config)?;
        WorkspacePublishDiffResponse::export(&config)?;
        WorkspacePublishHistoryEntryResponse::export(&config)?;
        RunWorkspacePublishRequest::export(&config)?;
        RunWorkspacePublishResponse::export(&config)?;
        AppEntityFormDto::export(&config)?;
        AppEntityViewDto::export(&config)?;
        AppEntityCapabilitiesResponse::export(&config)?;
        super::apps::AppEntityViewModeDto::export(&config)?;
        AppRoleEntityPermissionResponse::export(&config)?;
        FieldResponse::export(&config)?;
        BusinessRuleResponse::export(&config)?;
        FormResponse::export(&config)?;
        PublishedSchemaResponse::export(&config)?;
        ViewResponse::export(&config)?;
        RuntimeRecordResponse::export(&config)?;
        WorkflowResponse::export(&config)?;
        WorkflowRunResponse::export(&config)?;
        WorkflowRunAttemptResponse::export(&config)?;
        RoleResponse::export(&config)?;
        RoleAssignmentResponse::export(&config)?;
        TenantRegistrationModeResponse::export(&config)?;
        AuditLogEntryResponse::export(&config)?;
        RuntimeFieldPermissionResponse::export(&config)?;
        TemporaryAccessGrantResponse::export(&config)?;
        AuditRetentionPolicyResponse::export(&config)?;
        AuditPurgeResultResponse::export(&config)?;
        ErrorResponse::export(&config)?;
        HealthDependencyStatus::export(&config)?;
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
