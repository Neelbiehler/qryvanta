use std::net::SocketAddr;

use axum::http::HeaderMap;
use ipnet::IpNet;
use qryvanta_core::{AppError, TenantId, UserIdentity};
use tower_sessions::Session;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

use crate::middleware::extract_client_ip_from_parts;
use crate::state::AppState;

use super::{SESSION_CREATED_AT_KEY, SESSION_STEP_UP_VERIFIED_AT_KEY, SESSION_USER_KEY};

const STEP_UP_MAX_AGE_SECONDS: i64 = 10 * 60;

pub(super) async fn load_passkeys(
    state: &AppState,
    subject: &str,
) -> Result<Vec<Passkey>, AppError> {
    let passkey_json_values = state.passkey_repository.list_by_subject(subject).await?;

    passkey_json_values
        .into_iter()
        .map(|passkey_json| {
            serde_json::from_str::<Passkey>(&passkey_json)
                .map_err(|error| AppError::Internal(format!("failed to decode passkey: {error}")))
        })
        .collect()
}

pub(super) fn tenant_id_from_invite_metadata(
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

pub(super) fn default_display_name(email: &str) -> &str {
    email.split('@').next().unwrap_or("new user")
}

pub(super) async fn active_identity_for_subject(
    state: &AppState,
    subject: &str,
) -> Result<UserIdentity, AppError> {
    let selection = state
        .tenant_access_service
        .resolve_active_tenant(subject)
        .await?
        .ok_or_else(|| {
            AppError::Unauthorized(format!(
                "no tenant membership is configured for subject '{subject}'"
            ))
        })?;

    Ok(UserIdentity::new(
        subject.to_owned(),
        selection.display_name,
        selection.email,
        selection.tenant_id,
    ))
}

pub(super) async fn switch_identity_for_subject(
    state: &AppState,
    subject: &str,
    tenant_id: TenantId,
) -> Result<UserIdentity, AppError> {
    let selection = state
        .tenant_access_service
        .switch_active_tenant(subject, tenant_id)
        .await?;

    Ok(UserIdentity::new(
        subject.to_owned(),
        selection.display_name,
        selection.email,
        selection.tenant_id,
    ))
}

pub(super) async fn persist_authenticated_identity(
    session: &Session,
    identity: &UserIdentity,
) -> Result<(), AppError> {
    session
        .cycle_id()
        .await
        .map_err(|error| AppError::Internal(format!("failed to cycle session id: {error}")))?;

    session
        .insert(SESSION_USER_KEY, identity)
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

    Ok(())
}

pub(super) async fn mark_step_up_verified(session: &Session) -> Result<(), AppError> {
    session
        .insert(
            SESSION_STEP_UP_VERIFIED_AT_KEY,
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to persist step-up verification timestamp: {error}"
            ))
        })?;

    Ok(())
}

pub(crate) async fn require_recent_step_up(session: &Session) -> Result<(), AppError> {
    let verified_at = session
        .get::<i64>(SESSION_STEP_UP_VERIFIED_AT_KEY)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to read step-up verification timestamp: {error}"
            ))
        })?;

    if step_up_timestamp_is_fresh(verified_at, chrono::Utc::now().timestamp()) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "step-up authentication required for this action".to_owned(),
        ))
    }
}

fn step_up_timestamp_is_fresh(verified_at: Option<i64>, now_timestamp: i64) -> bool {
    verified_at
        .map(|verified_at| now_timestamp.saturating_sub(verified_at) <= STEP_UP_MAX_AGE_SECONDS)
        .unwrap_or(false)
}

pub(crate) fn constant_time_eq(left: &str, right: &str) -> bool {
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();

    let mut diff = left_bytes.len() ^ right_bytes.len();
    let max_len = left_bytes.len().max(right_bytes.len());

    for index in 0..max_len {
        let left_byte = left_bytes.get(index).copied().unwrap_or_default();
        let right_byte = right_bytes.get(index).copied().unwrap_or_default();
        diff |= usize::from(left_byte ^ right_byte);
    }

    diff == 0
}

pub(super) fn extract_request_context(
    headers: &HeaderMap,
    socket_addr: Option<SocketAddr>,
    trust_proxy_headers: bool,
    trusted_proxy_cidrs: &[IpNet],
) -> (Option<String>, Option<String>) {
    let ip_address = Some(extract_client_ip_from_parts(
        headers,
        socket_addr,
        trust_proxy_headers,
        trusted_proxy_cidrs,
    ));

    let user_agent = headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    (ip_address, user_agent)
}

#[cfg(test)]
mod tests {
    use qryvanta_core::TenantId;

    use super::{default_display_name, step_up_timestamp_is_fresh, tenant_id_from_invite_metadata};

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

    #[test]
    fn step_up_timestamp_requires_recent_verification() {
        let now = 1_000_i64;

        assert!(step_up_timestamp_is_fresh(Some(now), now));
        assert!(step_up_timestamp_is_fresh(Some(now - 600), now));
        assert!(!step_up_timestamp_is_fresh(Some(now - 601), now));
        assert!(!step_up_timestamp_is_fresh(None, now));
    }
}
