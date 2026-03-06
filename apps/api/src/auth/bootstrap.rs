use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::AuthEvent;
use qryvanta_core::AppError;
use qryvanta_domain::{AuthEventOutcome, AuthEventType};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_sessions::Session;

use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::{
    active_identity_for_subject, constant_time_eq, extract_request_context, mark_step_up_verified,
    persist_authenticated_identity,
};

#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    pub subject: String,
    pub token: String,
}

pub async fn bootstrap_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    session: Session,
    Json(payload): Json<BootstrapRequest>,
) -> ApiResult<StatusCode> {
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    if !constant_time_eq(payload.token.as_str(), state.bootstrap_token.as_str()) {
        state
            .auth_event_service
            .record_event(AuthEvent {
                subject: Some(payload.subject),
                event_type: AuthEventType::BootstrapLogin,
                outcome: AuthEventOutcome::Failed,
                ip_address,
                user_agent,
            })
            .await?;
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

    state
        .contact_bootstrap_service
        .ensure_subject_contact(
            tenant_id,
            payload.subject.as_str(),
            payload.subject.as_str(),
            None,
        )
        .await?;

    let subject = payload.subject;
    let identity = active_identity_for_subject(&state, subject.as_str()).await?;
    persist_authenticated_identity(&session, &identity).await?;
    mark_step_up_verified(&session).await?;

    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(subject),
            event_type: AuthEventType::BootstrapLogin,
            outcome: AuthEventOutcome::Success,
            ip_address,
            user_agent,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
