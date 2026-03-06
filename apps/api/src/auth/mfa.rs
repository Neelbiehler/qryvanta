use axum::Json;
use axum::extract::{ConnectInfo, Extension, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::AuthEvent;
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{AuthEventOutcome, AuthEventType, UserId};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::extract_request_context;
use super::{mfa_enroll_confirm_rate_rule, mfa_management_rate_rule};

#[derive(Debug, Deserialize)]
pub struct MfaDisableRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct MfaRegenerateRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TotpEnrollmentResponse {
    pub secret_base32: String,
    pub otpauth_uri: String,
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RecoveryCodesResponse {
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MfaConfirmRequest {
    pub code: String,
}

/// POST /auth/mfa/totp/enroll - Start TOTP enrollment.
pub async fn mfa_enroll_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<TotpEnrollmentResponse>> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    let enrollment_result = state.mfa_service.start_enrollment(user_id).await;
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );

    match &enrollment_result {
        Ok(_) => {
            state
                .auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.subject().to_owned()),
                    event_type: AuthEventType::MfaEnrollmentStarted,
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
                    event_type: AuthEventType::MfaEnrollmentStarted,
                    outcome: AuthEventOutcome::Failed,
                    ip_address,
                    user_agent,
                })
                .await?;
        }
    }

    let enrollment = enrollment_result?;

    Ok(Json(TotpEnrollmentResponse {
        secret_base32: enrollment.secret_base32,
        otpauth_uri: enrollment.otpauth_uri,
        recovery_codes: enrollment.recovery_codes,
    }))
}

/// POST /auth/mfa/totp/confirm - Confirm TOTP enrollment.
pub async fn mfa_confirm_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<MfaConfirmRequest>,
) -> ApiResult<StatusCode> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);
    let rate_limit_rule = mfa_enroll_confirm_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&rate_limit_rule, user.subject())
        .await?;

    let confirm_result = state
        .mfa_service
        .confirm_enrollment(user_id, &payload.code)
        .await;
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );

    match &confirm_result {
        Ok(_) => {
            state
                .auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.subject().to_owned()),
                    event_type: AuthEventType::MfaEnrollmentConfirmed,
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
                    event_type: AuthEventType::MfaEnrollmentConfirmed,
                    outcome: AuthEventOutcome::Failed,
                    ip_address,
                    user_agent,
                })
                .await?;
        }
    }

    confirm_result?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /auth/mfa/totp - Disable TOTP (requires password).
pub async fn mfa_disable_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(user): Extension<UserIdentity>,
    session: Session,
    Json(payload): Json<MfaDisableRequest>,
) -> ApiResult<StatusCode> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);
    let rate_limit_rule = mfa_management_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&rate_limit_rule, user.subject())
        .await?;

    let disable_result = async {
        state
            .mfa_service
            .disable_totp(user_id, &payload.password)
            .await?;
        state
            .user_service
            .user_repository()
            .revoke_sessions(user_id)
            .await?;
        session.delete().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to delete session after MFA disable: {error}"
            ))
        })?;
        Ok::<(), AppError>(())
    }
    .await;
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );

    match &disable_result {
        Ok(_) => {
            state
                .auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.subject().to_owned()),
                    event_type: AuthEventType::MfaDisabled,
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
                    event_type: AuthEventType::MfaDisabled,
                    outcome: AuthEventOutcome::Failed,
                    ip_address,
                    user_agent,
                })
                .await?;
        }
    }

    disable_result?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/mfa/recovery-codes/regenerate - Regenerate recovery codes.
pub async fn mfa_regenerate_recovery_codes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(user): Extension<UserIdentity>,
    session: Session,
    Json(payload): Json<MfaRegenerateRequest>,
) -> ApiResult<Json<RecoveryCodesResponse>> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);
    let rate_limit_rule = mfa_management_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&rate_limit_rule, user.subject())
        .await?;

    let codes_result = async {
        let codes = state
            .mfa_service
            .regenerate_recovery_codes(user_id, &payload.password)
            .await?;
        state
            .user_service
            .user_repository()
            .revoke_sessions(user_id)
            .await?;
        session.delete().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to delete session after recovery-code regeneration: {error}"
            ))
        })?;

        Ok::<Vec<String>, AppError>(codes)
    }
    .await;
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );

    match &codes_result {
        Ok(_) => {
            state
                .auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.subject().to_owned()),
                    event_type: AuthEventType::MfaRecoveryCodesRegenerated,
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
                    event_type: AuthEventType::MfaRecoveryCodesRegenerated,
                    outcome: AuthEventOutcome::Failed,
                    ip_address,
                    user_agent,
                })
                .await?;
        }
    }

    let codes = codes_result?;

    Ok(Json(RecoveryCodesResponse {
        recovery_codes: codes,
    }))
}
