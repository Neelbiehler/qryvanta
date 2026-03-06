use axum::Json;
use axum::extract::{ConnectInfo, Extension, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::{AuthEvent, AuthOutcome};
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{
    AuthEventOutcome, AuthEventType, AuthTokenType, EmailAddress, RegistrationMode, UserId,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_sessions::Session;
use uuid::Uuid;

use crate::dto::{
    AuthLoginRequest as LoginRequest, AuthLoginResponse as LoginResponse,
    AuthMfaVerifyRequest as MfaVerifyRequest, AuthRegisterRequest as RegisterRequest,
    GenericMessageResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::{
    active_identity_for_subject, extract_request_context, mark_step_up_verified,
    persist_authenticated_identity,
};
use super::{
    SESSION_MFA_PENDING_KEY, mfa_login_verify_rate_rule, resend_verification_rate_rule,
    verify_email_rate_rule,
};

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

/// POST /auth/register - Create a new account with email+password.
pub async fn register_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    let register_email = payload.email.clone();
    let register_display_name = payload.display_name.clone();

    let registration_mode = if let Some(tenant_id) = state.bootstrap_tenant_id {
        state
            .tenant_repository
            .registration_mode_for_tenant(tenant_id)
            .await?
    } else {
        RegistrationMode::Open
    };

    let user_id = state
        .user_service
        .register(qryvanta_application::RegisterParams {
            email: payload.email,
            password: payload.password,
            display_name: payload.display_name,
            registration_mode,
            preferred_tenant_id: state.bootstrap_tenant_id,
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
        })
        .await?;

    let user_subject = user_id.to_string();
    let tenant_id = state
        .tenant_repository
        .find_tenant_for_subject(user_subject.as_str())
        .await?
        .ok_or_else(|| AppError::Internal("user has no tenant membership".to_owned()))?;

    state
        .contact_bootstrap_service
        .ensure_subject_contact(
            tenant_id,
            user_subject.as_str(),
            register_display_name.as_str(),
            Some(register_email.as_str()),
        )
        .await?;

    state
        .auth_token_service
        .send_email_verification(user_id, &register_email)
        .await?;
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(user_id.to_string()),
            event_type: AuthEventType::EmailVerificationSent,
            outcome: AuthEventOutcome::Success,
            ip_address,
            user_agent,
        })
        .await?;

    // OWASP: generic response to prevent account enumeration.
    Ok(Json(GenericMessageResponse {
        message: "a link to activate your account has been emailed to the address provided"
            .to_owned(),
    }))
}

/// POST /auth/login - Authenticate with email+password.
pub async fn login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );

    let outcome = state
        .user_service
        .login(&payload.email, &payload.password, ip_address, user_agent)
        .await?;

    match outcome {
        AuthOutcome::Authenticated(user) => {
            let user_subject = user.id.to_string();
            let identity = active_identity_for_subject(&state, user_subject.as_str()).await?;

            state
                .contact_bootstrap_service
                .ensure_subject_contact(
                    identity.tenant_id(),
                    user_subject.as_str(),
                    identity.display_name(),
                    identity.email(),
                )
                .await?;
            persist_authenticated_identity(&session, &identity).await?;
            mark_step_up_verified(&session).await?;

            Ok(Json(LoginResponse {
                status: "authenticated".to_owned(),
                requires_totp: false,
            }))
        }
        AuthOutcome::MfaRequired { user_id } => {
            // Store the pending user_id in session for MFA verification.
            session
                .insert(SESSION_MFA_PENDING_KEY, user_id.to_string())
                .await
                .map_err(|error| {
                    AppError::Internal(format!("failed to persist MFA pending state: {error}"))
                })?;

            Ok(Json(LoginResponse {
                status: "mfa_required".to_owned(),
                requires_totp: true,
            }))
        }
        AuthOutcome::Failed => {
            // OWASP: generic error message for all failure cases.
            Err(AppError::Unauthorized("invalid email or password".to_owned()).into())
        }
    }
}

/// POST /auth/login/mfa - Complete MFA challenge with TOTP or recovery code.
pub async fn mfa_verify_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    session: Session,
    Json(payload): Json<MfaVerifyRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let pending_user_id_str: String = session
        .get(SESSION_MFA_PENDING_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read MFA pending state: {error}")))?
        .ok_or_else(|| AppError::Unauthorized("no MFA challenge in progress".to_owned()))?;

    let user_id_uuid = Uuid::parse_str(&pending_user_id_str)
        .map_err(|error| AppError::Internal(format!("invalid pending user id: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);
    let method = payload.method.as_deref().unwrap_or("totp");
    let rate_limit_rule = mfa_login_verify_rate_rule();
    let rate_limit_key = format!("{}:{method}", user_id);
    state
        .rate_limit_service
        .check_rate_limit(&rate_limit_rule, rate_limit_key.as_str())
        .await?;

    let valid = match method {
        "recovery" => {
            state
                .mfa_service
                .verify_recovery_code(user_id, &payload.code)
                .await?
        }
        _ => {
            state
                .mfa_service
                .verify_totp(user_id, &payload.code)
                .await?
        }
    };

    if !valid {
        let (ip_address, user_agent) = extract_request_context(
            &headers,
            Some(connect_info),
            state.trust_proxy_headers,
            &state.trusted_proxy_cidrs,
        );
        state
            .auth_event_service
            .record_event(AuthEvent {
                subject: Some(user_id.to_string()),
                event_type: AuthEventType::MfaVerification,
                outcome: AuthEventOutcome::Failed,
                ip_address,
                user_agent,
            })
            .await?;

        return Err(AppError::Unauthorized("invalid MFA code".to_owned()).into());
    }

    // MFA verified -- establish session.
    session
        .remove_value(SESSION_MFA_PENDING_KEY)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to clear MFA pending state: {error}"))
        })?;

    let user = state
        .user_service
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::Internal("user not found after MFA".to_owned()))?;

    let user_subject = user.id.to_string();

    let identity = active_identity_for_subject(&state, user_subject.as_str()).await?;

    state
        .contact_bootstrap_service
        .ensure_subject_contact(
            identity.tenant_id(),
            user_subject.as_str(),
            identity.display_name(),
            identity.email(),
        )
        .await?;
    persist_authenticated_identity(&session, &identity).await?;
    mark_step_up_verified(&session).await?;

    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(user_id.to_string()),
            event_type: AuthEventType::MfaVerification,
            outcome: AuthEventOutcome::Success,
            ip_address,
            user_agent,
        })
        .await?;

    Ok(Json(LoginResponse {
        status: "authenticated".to_owned(),
        requires_totp: false,
    }))
}

/// PUT /api/profile/password - Change password (requires auth).
pub async fn change_password_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(user): Extension<UserIdentity>,
    session: Session,
    Json(payload): Json<ChangePasswordRequest>,
) -> ApiResult<StatusCode> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    let change_result = async {
        state
            .user_service
            .change_password(user_id, &payload.current_password, &payload.new_password)
            .await?;

        state
            .user_service
            .user_repository()
            .revoke_sessions(user_id)
            .await?;

        session
            .delete()
            .await
            .map_err(|error| AppError::Internal(format!("failed to delete session: {error}")))?;

        Ok::<(), AppError>(())
    }
    .await;

    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(user.subject().to_owned()),
            event_type: AuthEventType::PasswordChanged,
            outcome: if change_result.is_ok() {
                AuthEventOutcome::Success
            } else {
                AuthEventOutcome::Failed
            },
            ip_address,
            user_agent,
        })
        .await?;

    change_result?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/forgot-password - Request password reset email.
pub async fn forgot_password_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let canonical_email = EmailAddress::new(payload.email.as_str()).ok();

    let user = if let Some(email) = canonical_email.as_ref() {
        state.user_service.find_by_email(email.as_str()).await?
    } else {
        None
    };

    let user_id = user.map(|u| u.id);
    let request_email = canonical_email
        .as_ref()
        .map(|email| email.as_str())
        .unwrap_or(payload.email.as_str());

    state
        .auth_token_service
        .request_password_reset(request_email, user_id)
        .await?;

    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: user_id.map(|value| value.to_string()),
            event_type: AuthEventType::PasswordResetRequested,
            outcome: AuthEventOutcome::Success,
            ip_address,
            user_agent,
        })
        .await?;

    // OWASP: always return generic success message.
    Ok(Json(GenericMessageResponse {
        message: "if that email address is in our database, we will send you an email to reset your password".to_owned(),
    }))
}

/// POST /auth/reset-password - Reset password with token.
pub async fn reset_password_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Json(payload): Json<ResetPasswordRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let reset_result = async {
        let token_record = state
            .auth_token_service
            .consume_valid_token(&payload.token, AuthTokenType::PasswordReset)
            .await?;

        let user_id = token_record
            .user_id
            .ok_or_else(|| AppError::Internal("password reset token has no user_id".to_owned()))?;

        let user = state
            .user_service
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;

        qryvanta_domain::validate_password(&payload.new_password, user.totp_enabled)?;

        let password_hash = state
            .user_service
            .password_hasher()
            .hash_password(&payload.new_password)?;

        state
            .user_service
            .user_repository()
            .update_password(user_id, &password_hash)
            .await?;
        state
            .user_service
            .user_repository()
            .reset_failed_logins(user_id)
            .await?;
        state
            .user_service
            .user_repository()
            .revoke_sessions(user_id)
            .await?;

        Ok::<String, AppError>(user_id.to_string())
    }
    .await;

    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: reset_result.as_ref().ok().cloned(),
            event_type: AuthEventType::PasswordResetCompleted,
            outcome: if reset_result.is_ok() {
                AuthEventOutcome::Success
            } else {
                AuthEventOutcome::Failed
            },
            ip_address,
            user_agent,
        })
        .await?;

    reset_result?;

    Ok(Json(GenericMessageResponse {
        message: "your password has been reset successfully".to_owned(),
    }))
}

/// POST /auth/verify-email - Verify email with token.
pub async fn verify_email_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Json(payload): Json<VerifyEmailRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
    let verify_rule = verify_email_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&verify_rule, ip_address.as_deref().unwrap_or("unknown"))
        .await?;

    let verify_result = async {
        let token_record = state
            .auth_token_service
            .consume_valid_token(&payload.token, AuthTokenType::EmailVerification)
            .await?;

        let user_id = token_record
            .user_id
            .ok_or_else(|| AppError::Internal("verification token has no user_id".to_owned()))?;

        state
            .user_service
            .user_repository()
            .mark_email_verified(user_id)
            .await?;

        Ok::<String, AppError>(user_id.to_string())
    }
    .await;
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: verify_result.as_ref().ok().cloned(),
            event_type: AuthEventType::EmailVerificationCompleted,
            outcome: if verify_result.is_ok() {
                AuthEventOutcome::Success
            } else {
                AuthEventOutcome::Failed
            },
            ip_address,
            user_agent,
        })
        .await?;

    verify_result?;

    Ok(Json(GenericMessageResponse {
        message: "email address verified successfully".to_owned(),
    }))
}

/// POST /auth/resend-verification - Resend email verification (requires auth).
pub async fn resend_verification_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let resend_rule = resend_verification_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&resend_rule, user.subject())
        .await?;

    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    let user_record = state
        .user_service
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_owned()))?;
    let (ip_address, user_agent) = extract_request_context(
        &headers,
        Some(connect_info),
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );

    if user_record.email_verified {
        state
            .auth_event_service
            .record_event(AuthEvent {
                subject: Some(user.subject().to_owned()),
                event_type: AuthEventType::EmailVerificationSent,
                outcome: AuthEventOutcome::AlreadyVerified,
                ip_address: ip_address.clone(),
                user_agent: user_agent.clone(),
            })
            .await?;
        return Ok(Json(GenericMessageResponse {
            message: "email is already verified".to_owned(),
        }));
    }

    state
        .auth_token_service
        .send_email_verification(user_id, &user_record.email)
        .await?;
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(user.subject().to_owned()),
            event_type: AuthEventType::EmailVerificationSent,
            outcome: AuthEventOutcome::Success,
            ip_address,
            user_agent,
        })
        .await?;

    Ok(Json(GenericMessageResponse {
        message: "verification email sent".to_owned(),
    }))
}
