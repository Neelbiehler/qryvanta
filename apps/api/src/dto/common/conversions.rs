use qryvanta_core::UserIdentity;

use super::types::UserIdentityResponse;

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
