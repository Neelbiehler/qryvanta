use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;

use qryvanta_core::UserIdentity;
use qryvanta_domain::{Permission, RegistrationMode};

use crate::dto::{
    AssignRoleRequest, AuditLogEntryResponse, CreateRoleRequest, RemoveRoleAssignmentRequest,
    RoleAssignmentResponse, RoleResponse, TenantRegistrationModeResponse,
    UpdateTenantRegistrationModeRequest,
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
