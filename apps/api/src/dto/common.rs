use qryvanta_core::UserIdentity;
use serde::Serialize;
use ts_rs::TS;

/// Health response payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/health-response.ts"
)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// Generic message response for auth flows.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/generic-message-response.ts"
)]
pub struct GenericMessageResponse {
    pub message: String,
}

/// API representation of the authenticated user.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/user-identity-response.ts"
)]
pub struct UserIdentityResponse {
    pub subject: String,
    pub display_name: String,
    pub email: Option<String>,
    pub tenant_id: String,
}

impl From<UserIdentity> for UserIdentityResponse {
    fn from(identity: UserIdentity) -> Self {
        Self {
            subject: identity.subject().to_owned(),
            display_name: identity.display_name().to_owned(),
            email: identity.email().map(ToOwned::to_owned),
            tenant_id: identity.tenant_id().to_string(),
        }
    }
}
