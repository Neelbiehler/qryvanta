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
    /// Surfaces the authenticated user may access (e.g. `["admin", "maker", "worker"]`).
    pub accessible_surfaces: Vec<String>,
}

impl UserIdentityResponse {
    /// Creates a response from the identity and resolved surfaces.
    #[must_use]
    pub fn from_identity_with_surfaces(
        identity: UserIdentity,
        surfaces: Vec<qryvanta_domain::Surface>,
    ) -> Self {
        Self {
            subject: identity.subject().to_owned(),
            display_name: identity.display_name().to_owned(),
            email: identity.email().map(ToOwned::to_owned),
            tenant_id: identity.tenant_id().to_string(),
            accessible_surfaces: surfaces
                .into_iter()
                .map(|surface| surface.as_str().to_owned())
                .collect(),
        }
    }
}

impl From<UserIdentity> for UserIdentityResponse {
    fn from(identity: UserIdentity) -> Self {
        Self {
            subject: identity.subject().to_owned(),
            display_name: identity.display_name().to_owned(),
            email: identity.email().map(ToOwned::to_owned),
            tenant_id: identity.tenant_id().to_string(),
            accessible_surfaces: Vec::new(),
        }
    }
}
