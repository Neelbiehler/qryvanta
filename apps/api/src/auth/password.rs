use axum::Json;
use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::{AuthEvent, AuthOutcome};
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{AuthTokenType, EmailAddress, RegistrationMode, UserId};
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::dto::{
    AuthLoginRequest as LoginRequest, AuthLoginResponse as LoginResponse,
    AuthMfaVerifyRequest as MfaVerifyRequest, AuthRegisterRequest as RegisterRequest,
    GenericMessageResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::extract_request_context;
use super::{
    SESSION_CREATED_AT_KEY, SESSION_MFA_PENDING_KEY, SESSION_USER_KEY,
    resend_verification_rate_rule, verify_email_rate_rule,
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
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let (ip_address, user_agent) = extract_request_context(&headers);
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
            ip_address,
            user_agent,
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
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let (ip_address, user_agent) = extract_request_context(&headers);

    let outcome = state
        .user_service
        .login(&payload.email, &payload.password, ip_address, user_agent)
        .await?;

    match outcome {
        AuthOutcome::Authenticated(user) => {
            let user_subject = user.id.to_string();
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
                    user.email.as_str(),
                    Some(user.email.as_str()),
                )
                .await?;

            let identity = UserIdentity::new(
                user_subject,
                user.email.clone(),
                Some(user.email.clone()),
                tenant_id,
            );

            // OWASP Session Management: regenerate session ID on privilege change.
            session.cycle_id().await.map_err(|error| {
                AppError::Internal(format!("failed to cycle session id: {error}"))
            })?;

            session
                .insert(SESSION_USER_KEY, &identity)
                .await
                .map_err(|error| {
                    AppError::Internal(format!("failed to persist session identity: {error}"))
                })?;

            // OWASP Session Management: record absolute creation time.
            session
                .insert(SESSION_CREATED_AT_KEY, chrono::Utc::now().timestamp())
                .await
                .map_err(|error| {
                    AppError::Internal(format!("failed to persist session creation time: {error}"))
                })?;

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
        let (ip_address, user_agent) = extract_request_context(&headers);
        state
            .auth_event_service
            .record_event(AuthEvent {
                subject: Some(user_id.to_string()),
                event_type: "mfa_verify".to_owned(),
                outcome: "failed".to_owned(),
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
            user.email.as_str(),
            Some(user.email.as_str()),
        )
        .await?;

    let identity = UserIdentity::new(
        user_subject,
        user.email.clone(),
        Some(user.email.clone()),
        tenant_id,
    );

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
            subject: Some(user_id.to_string()),
            event_type: "mfa_verify".to_owned(),
            outcome: "success".to_owned(),
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
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<ChangePasswordRequest>,
) -> ApiResult<StatusCode> {
    let user_id_uuid = Uuid::parse_str(user.subject())
        .map_err(|error| AppError::Internal(format!("invalid user subject: {error}")))?;
    let user_id = UserId::from_uuid(user_id_uuid);

    state
        .user_service
        .change_password(user_id, &payload.current_password, &payload.new_password)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/forgot-password - Request password reset email.
pub async fn forgot_password_handler(
    State(state): State<AppState>,
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

    // OWASP: always return generic success message.
    Ok(Json(GenericMessageResponse {
        message: "if that email address is in our database, we will send you an email to reset your password".to_owned(),
    }))
}

/// POST /auth/reset-password - Reset password with token.
pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let token_record = state
        .auth_token_service
        .consume_valid_token(&payload.token, AuthTokenType::PasswordReset)
        .await?;

    let user_id = token_record
        .user_id
        .ok_or_else(|| AppError::Internal("password reset token has no user_id".to_owned()))?;

    // Validate new password.
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

    // Reset failed logins and unlock account.
    state
        .user_service
        .user_repository()
        .reset_failed_logins(user_id)
        .await?;

    Ok(Json(GenericMessageResponse {
        message: "your password has been reset successfully".to_owned(),
    }))
}

/// POST /auth/verify-email - Verify email with token.
pub async fn verify_email_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VerifyEmailRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let (ip_address, _) = extract_request_context(&headers);
    let verify_rule = verify_email_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&verify_rule, ip_address.as_deref().unwrap_or("unknown"))
        .await?;

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

    Ok(Json(GenericMessageResponse {
        message: "email address verified successfully".to_owned(),
    }))
}

/// POST /auth/resend-verification - Resend email verification (requires auth).
pub async fn resend_verification_handler(
    State(state): State<AppState>,
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

    if user_record.email_verified {
        return Ok(Json(GenericMessageResponse {
            message: "email is already verified".to_owned(),
        }));
    }

    state
        .auth_token_service
        .send_email_verification(user_id, &user_record.email)
        .await?;

    Ok(Json(GenericMessageResponse {
        message: "verification email sent".to_owned(),
    }))
}
