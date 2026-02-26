use qryvanta_domain::Surface;

use super::*;

impl AuthorizationService {
    /// Returns the set of surfaces a subject may access in a tenant.
    ///
    /// A surface is accessible when the subject holds at least one of the
    /// permissions required by that surface (logical OR).
    pub async fn resolve_accessible_surfaces(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Surface>> {
        let permissions = self
            .repository
            .list_permissions_for_subject(tenant_id, subject)
            .await?;

        let mut surfaces = Vec::new();
        for surface in Surface::all() {
            let has_access = surface
                .required_permissions()
                .iter()
                .any(|required| permissions.contains(required));
            if has_access {
                surfaces.push(*surface);
            }
        }

        Ok(surfaces)
    }
}
