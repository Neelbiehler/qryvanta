use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::{AuthEvent, AuthOutcome, RateLimitRule};
use qryvanta_core::{AppError, TenantId, UserIdentity};
use qryvanta_domain::{AuthTokenType, EmailAddress, Permission, RegistrationMode, UserId};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;
use webauthn_rs::prelude::{
    Passkey, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use crate::dto::{
    AcceptInviteRequest, AuthLoginRequest as LoginRequest, AuthLoginResponse as LoginResponse,
    AuthMfaVerifyRequest as MfaVerifyRequest, AuthRegisterRequest as RegisterRequest,
    GenericMessageResponse, InviteRequest, UserIdentityResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub const SESSION_USER_KEY: &str = "user_identity";
/// Absolute session creation timestamp for OWASP absolute timeout enforcement.
pub const SESSION_CREATED_AT_KEY: &str = "session_created_at";
const SESSION_MFA_PENDING_KEY: &str = "mfa_pending_user_id";

const SESSION_WEBAUTHN_REG_STATE_KEY: &str = "webauthn_reg_state";
const SESSION_WEBAUTHN_AUTH_STATE_KEY: &str = "webauthn_auth_state";

const RESEND_VERIFICATION_RATE_RULE: (i32, i64) = (5, 60 * 60);
const INVITE_SENDER_RATE_RULE: (i32, i64) = (20, 60 * 60);
const INVITE_RECIPIENT_RATE_RULE: (i32, i64) = (3, 60 * 60);
const VERIFY_EMAIL_RATE_RULE: (i32, i64) = (30, 60 * 60);

#[derive(Debug, Deserialize)]
pub struct LoginStartQuery {
    pub subject: String,
}

#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    pub subject: String,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub requires_totp: bool,
}

// ---- New DTOs for password auth ----

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

pub async fn webauthn_registration_start_handler(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<serde_json::Value>> {
    let identity = session
        .get::<UserIdentity>(SESSION_USER_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read session identity: {error}")))?
        .ok_or_else(|| AppError::Unauthorized("authentication required".to_owned()))?;

    let subject = identity.subject().to_owned();

    let stored_passkeys = load_passkeys(&state, subject.as_str()).await?;
    let exclude_credentials = (!stored_passkeys.is_empty()).then(|| {
        stored_passkeys
            .iter()
            .map(|passkey| passkey.cred_id().clone())
            .collect()
    });

    let (creation_challenge_response, reg_state) = state
        .webauthn
        .start_passkey_registration(Uuid::new_v4(), &subject, &subject, exclude_credentials)
        .map_err(|error| {
            AppError::Internal(format!("failed to start passkey registration: {error}"))
        })?;

    session
        .insert(SESSION_WEBAUTHN_REG_STATE_KEY, (subject, reg_state))
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to persist registration state: {error}"))
        })?;

    Ok(Json(
        serde_json::to_value(creation_challenge_response).map_err(|error| {
            AppError::Internal(format!("failed to encode registration challenge: {error}"))
        })?,
    ))
}

pub async fn webauthn_registration_finish_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    session: Session,
    Json(payload): Json<RegisterPublicKeyCredential>,
) -> ApiResult<StatusCode> {
    let (subject, reg_state): (String, PasskeyRegistration) = session
        .get(SESSION_WEBAUTHN_REG_STATE_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read registration state: {error}")))?
        .ok_or_else(|| AppError::Unauthorized("missing registration state".to_owned()))?;

    session
        .remove_value(SESSION_WEBAUTHN_REG_STATE_KEY)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to clear registration state: {error}"))
        })?;

    let passkey = state
        .webauthn
        .finish_passkey_registration(&payload, &reg_state)
        .map_err(|error| {
            AppError::Unauthorized(format!("passkey registration verification failed: {error}"))
        })?;

    let passkey_json = serde_json::to_string(&passkey)
        .map_err(|error| AppError::Internal(format!("failed to serialize passkey: {error}")))?;

    state
        .passkey_repository
        .insert_for_subject(subject.as_str(), passkey_json.as_str())
        .await?;

    let (ip_address, user_agent) = extract_request_context(&headers);
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(subject),
            event_type: "passkey_registration".to_owned(),
            outcome: "success".to_owned(),
            ip_address,
            user_agent,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn webauthn_login_start_handler(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<LoginStartQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let passkeys = load_passkeys(&state, query.subject.as_str()).await?;
    if passkeys.is_empty() {
        return Err(AppError::Unauthorized("no passkeys enrolled for subject".to_owned()).into());
    }

    let (request_challenge_response, auth_state) = state
        .webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|error| AppError::Internal(format!("failed to start passkey login: {error}")))?;

    session
        .insert(
            SESSION_WEBAUTHN_AUTH_STATE_KEY,
            (query.subject, passkeys, auth_state),
        )
        .await
        .map_err(|error| AppError::Internal(format!("failed to persist auth state: {error}")))?;

    Ok(Json(
        serde_json::to_value(request_challenge_response).map_err(|error| {
            AppError::Internal(format!(
                "failed to encode authentication challenge: {error}"
            ))
        })?,
    ))
}

pub async fn webauthn_login_finish_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    session: Session,
    Json(payload): Json<PublicKeyCredential>,
) -> ApiResult<Json<AuthStatusResponse>> {
    let (subject, mut passkeys, auth_state): (String, Vec<Passkey>, PasskeyAuthentication) =
        session
            .get(SESSION_WEBAUTHN_AUTH_STATE_KEY)
            .await
            .map_err(|error| AppError::Internal(format!("failed to read auth state: {error}")))?
            .ok_or_else(|| AppError::Unauthorized("missing authentication state".to_owned()))?;

    session
        .remove_value(SESSION_WEBAUTHN_AUTH_STATE_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to clear auth state: {error}")))?;

    let auth_result = state
        .webauthn
        .finish_passkey_authentication(&payload, &auth_state)
        .map_err(|error| {
            AppError::Unauthorized(format!(
                "passkey authentication verification failed: {error}"
            ))
        })?;

    passkeys.iter_mut().for_each(|passkey| {
        passkey.update_credential(&auth_result);
    });

    let serialized_passkeys = passkeys
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<String>, _>>()
        .map_err(|error| AppError::Internal(format!("failed to serialize passkeys: {error}")))?;

    state
        .passkey_repository
        .replace_for_subject(subject.as_str(), serialized_passkeys.as_slice())
        .await?;

    let tenant_id = state
        .tenant_repository
        .find_tenant_for_subject(&subject)
        .await?
        .ok_or_else(|| {
            AppError::Unauthorized(format!(
                "no tenant membership is configured for subject '{subject}'"
            ))
        })?;

    state
        .tenant_repository
        .create_membership(tenant_id, &subject, &subject, None)
        .await?;

    let identity = UserIdentity::new(subject.clone(), subject.clone(), None, tenant_id);

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

    let (ip_address, user_agent) = extract_request_context(&headers);
    state
        .auth_event_service
        .record_event(AuthEvent {
            subject: Some(subject),
            event_type: "passkey_login".to_owned(),
            outcome: "success".to_owned(),
            ip_address,
            user_agent,
        })
        .await?;

    Ok(Json(AuthStatusResponse {
        requires_totp: false,
    }))
}

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

pub async fn me_handler(session: Session) -> ApiResult<Json<UserIdentityResponse>> {
    let identity = session
        .get::<UserIdentity>(SESSION_USER_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read session identity: {error}")))?
        .ok_or_else(|| AppError::Unauthorized("authentication required".to_owned()))?;

    Ok(Json(UserIdentityResponse::from(identity)))
}

// ---- New handlers for email+password auth ----

/// POST /auth/register - Create a new account with email+password.
pub async fn register_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    let (ip_address, user_agent) = extract_request_context(&headers);
    let register_email = payload.email.clone();

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
            let tenant_id = state
                .tenant_repository
                .find_tenant_for_subject(&user.id.to_string())
                .await?
                .ok_or_else(|| AppError::Internal("user has no tenant membership".to_owned()))?;

            let identity = UserIdentity::new(
                user.id.to_string(),
                user.email.clone(),
                Some(user.email),
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

    let tenant_id = state
        .tenant_repository
        .find_tenant_for_subject(&user.id.to_string())
        .await?
        .ok_or_else(|| AppError::Internal("user has no tenant membership".to_owned()))?;

    let identity = UserIdentity::new(
        user.id.to_string(),
        user.email.clone(),
        Some(user.email),
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
    let verify_rule = RateLimitRule::new(
        "verify_email",
        VERIFY_EMAIL_RATE_RULE.0,
        VERIFY_EMAIL_RATE_RULE.1,
    );
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
    let resend_rule = RateLimitRule::new(
        "resend_verification",
        RESEND_VERIFICATION_RATE_RULE.0,
        RESEND_VERIFICATION_RATE_RULE.1,
    );
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

/// POST /auth/invite - Send a tenant invite email (requires auth).
pub async fn send_invite_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<InviteRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    state
        .authorization_service
        .require_permission(
            user.tenant_id(),
            user.subject(),
            Permission::SecurityInviteSend,
        )
        .await?;

    let invite_sender_rule = RateLimitRule::new(
        "invite_sender",
        INVITE_SENDER_RATE_RULE.0,
        INVITE_SENDER_RATE_RULE.1,
    );
    state
        .rate_limit_service
        .check_rate_limit(&invite_sender_rule, user.subject())
        .await?;

    let canonical_email = EmailAddress::new(payload.email.as_str())?;
    let invite_recipient_rule = RateLimitRule::new(
        "invite_recipient",
        INVITE_RECIPIENT_RATE_RULE.0,
        INVITE_RECIPIENT_RATE_RULE.1,
    );
    let invite_recipient_key = format!("{}:{}", user.tenant_id(), canonical_email.as_str());
    state
        .rate_limit_service
        .check_rate_limit(&invite_recipient_rule, invite_recipient_key.as_str())
        .await?;

    let tenant_name = payload
        .tenant_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("your workspace");

    let metadata = serde_json::json!({
        "tenant_id": user.tenant_id().to_string(),
        "invited_by": user.subject(),
    });

    state
        .auth_token_service
        .send_invite(
            canonical_email.as_str(),
            user.display_name(),
            tenant_name,
            &metadata,
        )
        .await?;

    // OWASP: generic response to avoid enumeration.
    Ok(Json(GenericMessageResponse {
        message: "if the email can receive invites, an invitation has been sent".to_owned(),
    }))
}

/// POST /auth/invite/accept - Accept an invite token.
pub async fn accept_invite_handler(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<AcceptInviteRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let token_record = state
        .auth_token_service
        .consume_valid_token(&payload.token, AuthTokenType::Invite)
        .await?;

    let tenant_id = tenant_id_from_invite_metadata(token_record.metadata.as_ref())?;
    let invited_email = token_record.email.clone();

    let user_id =
        if let Some(existing_user) = state.user_service.find_by_email(&invited_email).await? {
            let display_name = payload
                .display_name
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| default_display_name(&invited_email));

            state
                .tenant_repository
                .create_membership(
                    tenant_id,
                    &existing_user.id.to_string(),
                    display_name,
                    Some(invited_email.as_str()),
                )
                .await?;

            existing_user.id
        } else {
            let password = payload.password.as_deref().ok_or_else(|| {
                AppError::Validation("password is required for new invited users".to_owned())
            })?;

            let display_name = payload
                .display_name
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| default_display_name(&invited_email));

            state
                .user_service
                .register(qryvanta_application::RegisterParams {
                    email: invited_email.clone(),
                    password: password.to_owned(),
                    display_name: display_name.to_owned(),
                    registration_mode: RegistrationMode::Open,
                    preferred_tenant_id: Some(tenant_id),
                    ip_address: None,
                    user_agent: None,
                })
                .await?
        };

    state
        .user_service
        .user_repository()
        .mark_email_verified(user_id)
        .await?;

    // Establish authenticated session for the invited user.
    session
        .cycle_id()
        .await
        .map_err(|error| AppError::Internal(format!("failed to cycle session id: {error}")))?;

    let identity = UserIdentity::new(
        user_id.to_string(),
        invited_email.clone(),
        Some(invited_email),
        tenant_id,
    );

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

    Ok(Json(LoginResponse {
        status: "authenticated".to_owned(),
        requires_totp: false,
    }))
}

// ---- MFA management handlers (requires auth) ----

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

async fn load_passkeys(state: &AppState, subject: &str) -> Result<Vec<Passkey>, AppError> {
    let passkey_json_values = state.passkey_repository.list_by_subject(subject).await?;

    passkey_json_values
        .into_iter()
        .map(|passkey_json| {
            serde_json::from_str::<Passkey>(&passkey_json)
                .map_err(|error| AppError::Internal(format!("failed to decode passkey: {error}")))
        })
        .collect()
}

fn tenant_id_from_invite_metadata(
    metadata: Option<&serde_json::Value>,
) -> Result<TenantId, AppError> {
    let metadata = metadata.ok_or_else(|| {
        AppError::Unauthorized("invite token is missing tenant metadata".to_owned())
    })?;

    let tenant_id_str = metadata
        .get("tenant_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            AppError::Unauthorized("invite token has invalid tenant metadata".to_owned())
        })?;

    let tenant_uuid = Uuid::parse_str(tenant_id_str)
        .map_err(|error| AppError::Unauthorized(format!("invalid invite tenant id: {error}")))?;

    Ok(TenantId::from_uuid(tenant_uuid))
}

fn default_display_name(email: &str) -> &str {
    email.split('@').next().unwrap_or("new user")
}

fn extract_request_context(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let user_agent = headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    (ip_address, user_agent)
}

#[cfg(test)]
mod tests {
    use super::{default_display_name, tenant_id_from_invite_metadata};
    use qryvanta_core::TenantId;

    #[test]
    fn invite_metadata_parses_tenant_id() {
        let tenant_id = TenantId::new();
        let metadata = serde_json::json!({
            "tenant_id": tenant_id.to_string(),
            "invited_by": "tester",
        });

        let parsed = tenant_id_from_invite_metadata(Some(&metadata));
        assert!(parsed.is_ok());
        assert_eq!(
            parsed.unwrap_or_default().to_string(),
            tenant_id.to_string()
        );
    }

    #[test]
    fn invite_metadata_rejects_missing_tenant_id() {
        let metadata = serde_json::json!({"invited_by": "tester"});
        let parsed = tenant_id_from_invite_metadata(Some(&metadata));
        assert!(parsed.is_err());
    }

    #[test]
    fn display_name_defaults_to_email_local_part() {
        assert_eq!(default_display_name("alex@company.com"), "alex");
    }
}
