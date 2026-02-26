use super::*;

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
