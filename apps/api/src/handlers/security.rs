use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;

use qryvanta_core::UserIdentity;
use qryvanta_domain::{Permission, RegistrationMode};

use crate::dto::{
    AssignRoleRequest, AuditLogEntryResponse, AuditPurgeResultResponse,
    AuditRetentionPolicyResponse, CreateRoleRequest, CreateTemporaryAccessGrantRequest,
    RemoveRoleAssignmentRequest, RevokeTemporaryAccessGrantRequest, RoleAssignmentResponse,
    RoleResponse, RuntimeFieldPermissionResponse, SaveRuntimeFieldPermissionsRequest,
    TemporaryAccessGrantResponse, TenantRegistrationModeResponse,
    UpdateAuditRetentionPolicyRequest, UpdateTenantRegistrationModeRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct AuditLogQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub action: Option<String>,
    pub subject: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RuntimeFieldPermissionQuery {
    pub subject: Option<String>,
    pub entity_logical_name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TemporaryAccessGrantListQuery {
    pub subject: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_roles_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<RoleResponse>>> {
    let roles = state
        .security_admin_service
        .list_roles(&user)
        .await?
        .into_iter()
        .map(RoleResponse::from)
        .collect();

    Ok(Json(roles))
}

pub async fn create_role_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateRoleRequest>,
) -> ApiResult<(StatusCode, Json<RoleResponse>)> {
    let permissions = payload
        .permissions
        .iter()
        .map(|value| Permission::from_transport(value.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let role = state
        .security_admin_service
        .create_role(
            &user,
            qryvanta_application::CreateRoleInput {
                name: payload.name,
                permissions,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(RoleResponse::from(role))))
}

pub async fn assign_role_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<AssignRoleRequest>,
) -> ApiResult<StatusCode> {
    state
        .security_admin_service
        .assign_role(&user, payload.subject.as_str(), payload.role_name.as_str())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unassign_role_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<RemoveRoleAssignmentRequest>,
) -> ApiResult<StatusCode> {
    state
        .security_admin_service
        .unassign_role(&user, payload.subject.as_str(), payload.role_name.as_str())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_role_assignments_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<RoleAssignmentResponse>>> {
    let assignments = state
        .security_admin_service
        .list_role_assignments(&user)
        .await?
        .into_iter()
        .map(RoleAssignmentResponse::from)
        .collect();

    Ok(Json(assignments))
}

pub async fn list_audit_log_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<AuditLogQuery>,
) -> ApiResult<Json<Vec<AuditLogEntryResponse>>> {
    let entries = state
        .security_admin_service
        .list_audit_log(
            &user,
            qryvanta_application::AuditLogQuery {
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
                action: query.action,
                subject: query.subject,
            },
        )
        .await?
        .into_iter()
        .map(AuditLogEntryResponse::from)
        .collect();

    Ok(Json(entries))
}

pub async fn export_audit_log_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<AuditLogQuery>,
) -> ApiResult<Json<Vec<AuditLogEntryResponse>>> {
    let entries = state
        .security_admin_service
        .export_audit_log(
            &user,
            qryvanta_application::AuditLogQuery {
                limit: query.limit.unwrap_or(1_000),
                offset: query.offset.unwrap_or(0),
                action: query.action,
                subject: query.subject,
            },
        )
        .await?
        .into_iter()
        .map(AuditLogEntryResponse::from)
        .collect();

    Ok(Json(entries))
}

pub async fn purge_audit_log_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<AuditPurgeResultResponse>> {
    let result = state
        .security_admin_service
        .purge_audit_log_entries(&user)
        .await?;

    Ok(Json(AuditPurgeResultResponse::from(result)))
}

pub async fn audit_retention_policy_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<AuditRetentionPolicyResponse>> {
    let policy = state
        .security_admin_service
        .audit_retention_policy(&user)
        .await?;

    Ok(Json(AuditRetentionPolicyResponse::from(policy)))
}

pub async fn update_audit_retention_policy_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<UpdateAuditRetentionPolicyRequest>,
) -> ApiResult<Json<AuditRetentionPolicyResponse>> {
    let policy = state
        .security_admin_service
        .update_audit_retention_policy(&user, payload.retention_days)
        .await?;

    Ok(Json(AuditRetentionPolicyResponse::from(policy)))
}

pub async fn save_runtime_field_permissions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<SaveRuntimeFieldPermissionsRequest>,
) -> ApiResult<Json<Vec<RuntimeFieldPermissionResponse>>> {
    let entries = state
        .security_admin_service
        .save_runtime_field_permissions(
            &user,
            qryvanta_application::SaveRuntimeFieldPermissionsInput {
                subject: payload.subject,
                entity_logical_name: payload.entity_logical_name,
                fields: payload
                    .fields
                    .into_iter()
                    .map(|field| qryvanta_application::RuntimeFieldPermissionInput {
                        field_logical_name: field.field_logical_name,
                        can_read: field.can_read,
                        can_write: field.can_write,
                    })
                    .collect(),
            },
        )
        .await?
        .into_iter()
        .map(RuntimeFieldPermissionResponse::from)
        .collect();

    Ok(Json(entries))
}

pub async fn list_runtime_field_permissions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<RuntimeFieldPermissionQuery>,
) -> ApiResult<Json<Vec<RuntimeFieldPermissionResponse>>> {
    let entries = state
        .security_admin_service
        .list_runtime_field_permissions(
            &user,
            query.subject.as_deref(),
            query.entity_logical_name.as_deref(),
        )
        .await?
        .into_iter()
        .map(RuntimeFieldPermissionResponse::from)
        .collect();

    Ok(Json(entries))
}

pub async fn create_temporary_access_grant_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateTemporaryAccessGrantRequest>,
) -> ApiResult<(StatusCode, Json<TemporaryAccessGrantResponse>)> {
    let permissions = payload
        .permissions
        .iter()
        .map(|value| Permission::from_transport(value.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let grant = state
        .security_admin_service
        .create_temporary_access_grant(
            &user,
            qryvanta_application::CreateTemporaryAccessGrantInput {
                subject: payload.subject,
                permissions,
                reason: payload.reason,
                duration_minutes: payload.duration_minutes,
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(TemporaryAccessGrantResponse::from(grant)),
    ))
}

pub async fn list_temporary_access_grants_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<TemporaryAccessGrantListQuery>,
) -> ApiResult<Json<Vec<TemporaryAccessGrantResponse>>> {
    let grants = state
        .security_admin_service
        .list_temporary_access_grants(
            &user,
            qryvanta_application::TemporaryAccessGrantQuery {
                subject: query.subject,
                active_only: query.active_only.unwrap_or(false),
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
            },
        )
        .await?
        .into_iter()
        .map(TemporaryAccessGrantResponse::from)
        .collect();

    Ok(Json(grants))
}

pub async fn revoke_temporary_access_grant_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(grant_id): Path<String>,
    Json(payload): Json<RevokeTemporaryAccessGrantRequest>,
) -> ApiResult<StatusCode> {
    state
        .security_admin_service
        .revoke_temporary_access_grant(&user, grant_id.as_str(), payload.revoke_reason.as_deref())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn registration_mode_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<TenantRegistrationModeResponse>> {
    let registration_mode = state
        .security_admin_service
        .registration_mode(&user)
        .await?;

    Ok(Json(TenantRegistrationModeResponse::from(
        registration_mode,
    )))
}

pub async fn update_registration_mode_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<UpdateTenantRegistrationModeRequest>,
) -> ApiResult<Json<TenantRegistrationModeResponse>> {
    let registration_mode = RegistrationMode::parse(payload.registration_mode.as_str())?;

    let updated_mode = state
        .security_admin_service
        .update_registration_mode(&user, registration_mode)
        .await?;

    Ok(Json(TenantRegistrationModeResponse::from(updated_mode)))
}
