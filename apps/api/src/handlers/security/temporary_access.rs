use super::*;

#[derive(Debug, serde::Deserialize)]
pub struct TemporaryAccessGrantListQuery {
    pub subject: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
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
