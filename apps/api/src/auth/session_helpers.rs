use axum::http::HeaderMap;
use qryvanta_core::{AppError, TenantId};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

use crate::state::AppState;

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

pub(super) fn extract_request_context(headers: &HeaderMap) -> (Option<String>, Option<String>) {
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
    use qryvanta_core::TenantId;

    use super::{default_display_name, tenant_id_from_invite_metadata};

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
