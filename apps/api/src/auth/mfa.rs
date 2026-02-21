use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

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
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<TotpEnrollmentResponse>> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    let enrollment = state.mfa_service.start_enrollment(user_id).await?;

    Ok(Json(TotpEnrollmentResponse {
        secret_base32: enrollment.secret_base32,
        otpauth_uri: enrollment.otpauth_uri,
        recovery_codes: enrollment.recovery_codes,
    }))
}

/// POST /auth/mfa/totp/confirm - Confirm TOTP enrollment.
pub async fn mfa_confirm_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<MfaConfirmRequest>,
) -> ApiResult<StatusCode> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    state
        .mfa_service
        .confirm_enrollment(user_id, &payload.code)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /auth/mfa/totp - Disable TOTP (requires password).
pub async fn mfa_disable_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<MfaDisableRequest>,
) -> ApiResult<StatusCode> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    state
        .mfa_service
        .disable_totp(user_id, &payload.password)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/mfa/recovery-codes/regenerate - Regenerate recovery codes.
pub async fn mfa_regenerate_recovery_codes_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<MfaRegenerateRequest>,
) -> ApiResult<Json<RecoveryCodesResponse>> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    let codes = state
        .mfa_service
        .regenerate_recovery_codes(user_id, &payload.password)
        .await?;

    Ok(Json(RecoveryCodesResponse {
        recovery_codes: codes,
    }))
}
