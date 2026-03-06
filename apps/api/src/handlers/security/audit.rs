use super::*;

#[derive(Debug, serde::Deserialize)]
pub struct AuditLogQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub action: Option<String>,
    pub subject: Option<String>,
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

pub async fn verify_audit_log_integrity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<AuditIntegrityStatusResponse>> {
    let status = state
        .security_admin_service
        .verify_audit_integrity(&user)
        .await?;

    Ok(Json(AuditIntegrityStatusResponse::from(status)))
}

pub async fn purge_audit_log_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    session: Session,
) -> ApiResult<Json<AuditPurgeResultResponse>> {
    require_recent_step_up(&session).await?;

    let result = state
        .security_admin_service
        .purge_audit_log_entries(&user)
        .await?;

    Ok(Json(AuditPurgeResultResponse::from(result)))
}
