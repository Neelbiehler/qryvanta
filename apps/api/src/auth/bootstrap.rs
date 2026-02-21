use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::AuthEvent;
use qryvanta_core::{AppError, UserIdentity};
use serde::Deserialize;
use tower_sessions::Session;

use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::extract_request_context;
use super::{SESSION_CREATED_AT_KEY, SESSION_USER_KEY};

#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    pub subject: String,
    pub token: String,
}

pub async fn bootstrap_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    session: Session,
    Json(payload): Json<BootstrapRequest>,
) -> ApiResult<StatusCode> {
    if payload.token != state.bootstrap_token {
        return Err(AppError::Unauthorized("invalid bootstrap token".to_owned()).into());
    }

    let tenant_id = state
        .tenant_repository
        .ensure_membership_for_subject(
            &payload.subject,
            &payload.subject,
            None,
            state.bootstrap_tenant_id,
        )
        .await?;

    let subject = payload.subject;
    let identity = UserIdentity::new(subject.clone(), subject.clone(), None, tenant_id);

    session
        .cycle_id()
        .await
        .map_err(|error| AppError::Internal(format!("failed to cycle session id: {error}")))?;

    session
        .insert(SESSION_USER_KEY, &identity)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to persist session identity: {error}"))
        })?;

    session
        .insert(SESSION_CREATED_AT_KEY, chrono::Utc::now().timestamp())
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to persist session creation time: {error}"))
        })?;

    let (ip_address, user_agent) = extract_request_context(&headers);
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(subject),
            event_type: "bootstrap_login".to_owned(),
            outcome: "success".to_owned(),
            ip_address,
            user_agent,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
