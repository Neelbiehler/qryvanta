use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::AuthEvent;
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{AuthEventOutcome, AuthEventType};
use std::net::SocketAddr;
use tower_sessions::Session;
use uuid::Uuid;

use crate::dto::{AuthSwitchTenantRequest, UserIdentityResponse};
use crate::error::ApiResult;
use crate::state::AppState;

use super::SESSION_USER_KEY;
use super::session_helpers::{
    extract_request_context, persist_authenticated_identity, switch_identity_for_subject,
};

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
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

    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject,
            event_type: AuthEventType::SessionLogout,
            outcome: AuthEventOutcome::Success,
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
        .tenant_access_service
        .list_subject_tenants(identity.subject())
        .await?;

    Ok(Json(UserIdentityResponse::from_identity_with_surfaces(
        identity, surfaces,
    )))
}

pub async fn switch_tenant_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    session: Session,
    Json(payload): Json<AuthSwitchTenantRequest>,
) -> ApiResult<Json<UserIdentityResponse>> {
    let current_identity = session
        .get::<UserIdentity>(SESSION_USER_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read session identity: {error}")))?
        .ok_or_else(|| AppError::Unauthorized("authentication required".to_owned()))?;

    let tenant_uuid = Uuid::parse_str(payload.tenant_id.as_str()).map_err(|error| {
        AppError::Validation(format!(
            "invalid tenant id '{}': {error}",
            payload.tenant_id
        ))
    })?;
    let next_identity = switch_identity_for_subject(
        &state,
        current_identity.subject(),
        qryvanta_core::TenantId::from_uuid(tenant_uuid),
    )
    .await?;
    persist_authenticated_identity(&session, &next_identity).await?;

    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(current_identity.subject().to_owned()),
            event_type: AuthEventType::SessionTenantSwitched,
            outcome: AuthEventOutcome::Success,
            ip_address,
            user_agent,
        })
        .await?;

    let available_tenants = state
        .tenant_access_service
        .list_subject_tenants(next_identity.subject())
        .await?;

    Ok(Json(UserIdentityResponse::from_identity_with_surfaces(
        next_identity,
        available_tenants,
    )))
}
