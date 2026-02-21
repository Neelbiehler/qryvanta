use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use qryvanta_application::AuthEvent;
use qryvanta_core::{AppError, UserIdentity};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;
use webauthn_rs::prelude::{
    Passkey, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::{extract_request_context, load_passkeys};
use super::{
    SESSION_CREATED_AT_KEY, SESSION_USER_KEY, SESSION_WEBAUTHN_AUTH_STATE_KEY,
    SESSION_WEBAUTHN_REG_STATE_KEY,
};

#[derive(Debug, Deserialize)]
pub struct LoginStartQuery {
    pub subject: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub requires_totp: bool,
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
