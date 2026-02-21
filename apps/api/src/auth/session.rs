use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::AuthEvent;
use qryvanta_core::{AppError, UserIdentity};
use tower_sessions::Session;

use crate::dto::UserIdentityResponse;
use crate::error::ApiResult;
use crate::state::AppState;

use super::SESSION_USER_KEY;
use super::session_helpers::extract_request_context;

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    session: Session,
) -> ApiResult<StatusCode> {
    let subject = session
        .get::<UserIdentity>(SESSION_USER_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read session identity: {error}")))?
        .map(|identity| identity.subject().to_owned());

    session
        .delete()
        .await
        .map_err(|error| AppError::Internal(format!("failed to delete session: {error}")))?;

    let (ip_address, user_agent) = extract_request_context(&headers);
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject,
            event_type: "logout".to_owned(),
            outcome: "success".to_owned(),
            ip_address,
            user_agent,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn me_handler(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<UserIdentityResponse>> {
    let identity = session
        .get::<UserIdentity>(SESSION_USER_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read session identity: {error}")))?
        .ok_or_else(|| AppError::Unauthorized("authentication required".to_owned()))?;

    let surfaces = state
        .authorization_service
        .resolve_accessible_surfaces(identity.tenant_id(), identity.subject())
        .await?;

    Ok(Json(UserIdentityResponse::from_identity_with_surfaces(
        identity, surfaces,
    )))
}
