use qryvanta_application::TenantSelection;
use qryvanta_core::UserIdentity;

use super::types::{TenantOptionResponse, UserIdentityResponse};

impl TenantOptionResponse {
    #[must_use]
    pub fn from_selection(
        selection: TenantSelection,
        current_tenant_id: qryvanta_core::TenantId,
    ) -> Self {
        Self {
            tenant_id: selection.tenant_id.to_string(),
            tenant_name: selection.tenant_name,
            display_name: selection.display_name,
            email: selection.email,
            accessible_surfaces: selection.accessible_surfaces,
            is_current: selection.tenant_id == current_tenant_id,
            is_default: selection.is_default,
        }
    }
}

impl UserIdentityResponse {
    /// Creates a response from the identity and resolved surfaces.
    #[must_use]
    pub fn from_identity_with_surfaces(
        identity: UserIdentity,
        available_tenants: Vec<TenantSelection>,
    ) -> Self {
        Self {
            subject: identity.subject().to_owned(),
            display_name: identity.display_name().to_owned(),
            email: identity.email().map(ToOwned::to_owned),
            tenant_id: identity.tenant_id().to_string(),
            accessible_surfaces: available_tenants
                .iter()
                .find(|selection| selection.tenant_id == identity.tenant_id())
                .map(|selection| selection.accessible_surfaces.clone())
                .unwrap_or_default(),
            available_tenants: available_tenants
                .into_iter()
                .map(|selection| {
                    TenantOptionResponse::from_selection(selection, identity.tenant_id())
                })
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
            available_tenants: Vec::new(),
        }
    }
}
