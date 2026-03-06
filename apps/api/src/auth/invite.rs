use axum::Json;
use axum::extract::{ConnectInfo, Extension, State};
use axum::http::HeaderMap;
use qryvanta_application::AuthEvent;
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{
    AuthEventOutcome, AuthEventType, AuthTokenType, EmailAddress, Permission, RegistrationMode,
};
use std::net::SocketAddr;
use tower_sessions::Session;

use crate::dto::{
    AcceptInviteRequest, AuthLoginResponse as LoginResponse, GenericMessageResponse, InviteRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

use super::session_helpers::{
    default_display_name, extract_request_context, mark_step_up_verified,
    persist_authenticated_identity, switch_identity_for_subject, tenant_id_from_invite_metadata,
};
use super::{invite_recipient_rate_rule, invite_sender_rate_rule};

/// POST /auth/invite - Send a tenant invite email (requires auth).
pub async fn send_invite_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
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

    let invite_sender_rule = invite_sender_rate_rule();
    state
        .rate_limit_service
        .check_rate_limit(&invite_sender_rule, user.subject())
        .await?;

    let canonical_email = EmailAddress::new(payload.email.as_str())?;
    let invite_recipient_rule = invite_recipient_rate_rule();
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

    let send_result = state
        .auth_token_service
        .send_invite(
            canonical_email.as_str(),
            user.display_name(),
            tenant_name,
            &metadata,
        )
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
            event_type: AuthEventType::InviteSent,
            outcome: if send_result.is_ok() {
                AuthEventOutcome::Success
            } else {
                AuthEventOutcome::Failed
            },
            ip_address,
            user_agent,
        })
        .await?;

    send_result?;

    // OWASP: generic response to avoid enumeration.
    Ok(Json(GenericMessageResponse {
        message: "if the email can receive invites, an invitation has been sent".to_owned(),
    }))
}

/// POST /auth/invite/accept - Accept an invite token.
pub async fn accept_invite_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    session: Session,
    Json(payload): Json<AcceptInviteRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let accept_result = async {
        let token_record = state
            .auth_token_service
            .consume_valid_token(&payload.token, AuthTokenType::Invite)
            .await?;

        let tenant_id = tenant_id_from_invite_metadata(token_record.metadata.as_ref())?;
        let invited_email = token_record.email.clone();
        let display_name = payload
            .display_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| default_display_name(&invited_email))
            .to_owned();

        let user_id =
            if let Some(existing_user) = state.user_service.find_by_email(&invited_email).await? {
                state
                    .tenant_repository
                    .create_membership(
                        tenant_id,
                        &existing_user.id.to_string(),
                        display_name.as_str(),
                        Some(invited_email.as_str()),
                    )
                    .await?;

                existing_user.id
            } else {
                let password = payload.password.as_deref().ok_or_else(|| {
                    AppError::Validation("password is required for new invited users".to_owned())
                })?;

                state
                    .user_service
                    .register(qryvanta_application::RegisterParams {
                        email: invited_email.clone(),
                        password: password.to_owned(),
                        display_name: display_name.clone(),
                        registration_mode: RegistrationMode::Open,
                        preferred_tenant_id: Some(tenant_id),
                        ip_address: None,
                        user_agent: None,
                    })
                    .await?
            };

        let user_subject = user_id.to_string();

        state
            .contact_bootstrap_service
            .ensure_subject_contact(
                tenant_id,
                user_subject.as_str(),
                display_name.as_str(),
                Some(invited_email.as_str()),
            )
            .await?;

        state
            .user_service
            .user_repository()
            .mark_email_verified(user_id)
            .await?;

        let identity =
            switch_identity_for_subject(&state, user_id.to_string().as_str(), tenant_id).await?;
        persist_authenticated_identity(&session, &identity).await?;
        mark_step_up_verified(&session).await?;

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
            subject: accept_result.as_ref().ok().cloned(),
            event_type: AuthEventType::InviteAccepted,
            outcome: if accept_result.is_ok() {
                AuthEventOutcome::Success
            } else {
                AuthEventOutcome::Failed
            },
            ip_address,
            user_agent,
        })
        .await?;

    accept_result?;

    Ok(Json(LoginResponse {
        status: "authenticated".to_owned(),
        requires_totp: false,
    }))
}
