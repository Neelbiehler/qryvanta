use axum::Json;
use axum::extract::{ConnectInfo, Extension, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::AuthEvent;
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{AuthEventOutcome, AuthEventType, UserId};
use std::net::SocketAddr;
use tower_sessions::Session;
use uuid::Uuid;

use crate::dto::AuthStepUpRequest;
use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::{extract_request_context, mark_step_up_verified};
use super::step_up_verify_rate_rule;

pub async fn step_up_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(user): Extension<UserIdentity>,
    session: Session,
    Json(payload): Json<AuthStepUpRequest>,
) -> ApiResult<StatusCode> {
    let rate_limit_rule = step_up_verify_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&rate_limit_rule, user.subject())
        .await?;

    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    let verification_result = verify_step_up(&state, user_id, &payload).await;
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );

    match &verification_result {
        Ok(()) => {
            state
                .auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.subject().to_owned()),
                    event_type: AuthEventType::SessionStepUpVerification,
                    outcome: AuthEventOutcome::Success,
                    ip_address,
                    user_agent,
                })
                .await?;
        }
        Err(_) => {
            state
                .auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.subject().to_owned()),
                    event_type: AuthEventType::SessionStepUpVerification,
                    outcome: AuthEventOutcome::Failed,
                    ip_address,
                    user_agent,
                })
                .await?;
        }
    }

    verification_result?;
    mark_step_up_verified(&session).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn verify_step_up(
    state: &AppState,
    user_id: UserId,
    payload: &AuthStepUpRequest,
) -> Result<(), AppError> {
    let password = payload
        .password
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    if !password.is_empty() {
        let user = state
            .user_service
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        let Some(stored_hash) = user.password_hash.as_deref() else {
            return Err(AppError::Validation(
                "password step-up is not available for this account; use MFA instead".to_owned(),
            ));
        };

        let valid = state
            .user_service
            .password_hasher()
            .verify_password(password, stored_hash)?;

        if valid {
            return Ok(());
        }

        return Err(AppError::Unauthorized("incorrect password".to_owned()));
    }

    let code = payload.code.as_deref().map(str::trim).unwrap_or_default();
    if code.is_empty() {
        return Err(AppError::Validation(
            "password or MFA code is required for step-up authentication".to_owned(),
        ));
    }

    let method = payload.method.as_deref().unwrap_or("totp");
    let valid = match method {
        "totp" => state.mfa_service.verify_totp(user_id, code).await?,
        "recovery" => {
            state
                .mfa_service
                .verify_recovery_code(user_id, code)
                .await?
        }
        _ => {
            return Err(AppError::Validation(format!(
                "unsupported step-up method '{method}'"
            )));
        }
    };

    if valid {
        Ok(())
    } else {
        Err(AppError::Unauthorized("invalid MFA code".to_owned()))
    }
}
