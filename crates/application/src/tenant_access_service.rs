use std::sync::Arc;

use qryvanta_core::{AppError, AppResult, TenantId};

use crate::{AuthorizationService, TenantRepository, UserRepository};

/// One tenant option resolved for an authenticated subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantSelection {
    /// Tenant identifier.
    pub tenant_id: TenantId,
    /// Tenant display name.
    pub tenant_name: String,
    /// Membership-specific display name.
    pub display_name: String,
    /// Optional membership email.
    pub email: Option<String>,
    /// Surfaces available to the subject in this tenant.
    pub accessible_surfaces: Vec<String>,
    /// Whether this tenant is the persisted default.
    pub is_default: bool,
}

/// Resolves the active tenant and available tenant memberships for a subject.
#[derive(Clone)]
pub struct TenantAccessService {
    tenant_repository: Arc<dyn TenantRepository>,
    user_repository: Arc<dyn UserRepository>,
    authorization_service: AuthorizationService,
}

impl TenantAccessService {
    /// Creates a new tenant access service.
    #[must_use]
    pub fn new(
        tenant_repository: Arc<dyn TenantRepository>,
        user_repository: Arc<dyn UserRepository>,
        authorization_service: AuthorizationService,
    ) -> Self {
        Self {
            tenant_repository,
            user_repository,
            authorization_service,
        }
    }

    /// Lists all tenant memberships for a subject with accessible surfaces.
    pub async fn list_subject_tenants(&self, subject: &str) -> AppResult<Vec<TenantSelection>> {
        let memberships = self
            .tenant_repository
            .list_memberships_for_subject(subject)
            .await?;
        let default_tenant_id = self.default_tenant_id_for_subject(subject).await?;

        let mut selections = Vec::with_capacity(memberships.len());
        for membership in memberships {
            let surfaces = self
                .authorization_service
                .resolve_accessible_surfaces(membership.tenant_id, subject)
                .await?;
            selections.push(TenantSelection {
                tenant_id: membership.tenant_id,
                tenant_name: membership.tenant_name,
                display_name: membership.display_name,
                email: membership.email,
                accessible_surfaces: surfaces
                    .into_iter()
                    .map(|surface| surface.as_str().to_owned())
                    .collect(),
                is_default: default_tenant_id == Some(membership.tenant_id),
            });
        }

        Ok(selections)
    }

    /// Resolves the active tenant for a subject and persists the deterministic
    /// fallback for user-backed subjects when no default has been set.
    pub async fn resolve_active_tenant(&self, subject: &str) -> AppResult<Option<TenantSelection>> {
        let mut selections = self.list_subject_tenants(subject).await?;
        if selections.is_empty() {
            return Ok(None);
        }

        if let Some(selection) = selections.iter().find(|selection| selection.is_default) {
            return Ok(Some(selection.clone()));
        }

        let fallback = selections.remove(0);
        self.persist_default_tenant_if_user(subject, fallback.tenant_id)
            .await?;

        Ok(Some(TenantSelection {
            is_default: true,
            ..fallback
        }))
    }

    /// Switches the active tenant for a subject after validating membership.
    pub async fn switch_active_tenant(
        &self,
        subject: &str,
        tenant_id: TenantId,
    ) -> AppResult<TenantSelection> {
        let selection = self
            .list_subject_tenants(subject)
            .await?
            .into_iter()
            .find(|selection| selection.tenant_id == tenant_id)
            .ok_or_else(|| {
                AppError::Forbidden(format!(
                    "subject '{subject}' is not a member of tenant '{tenant_id}'"
                ))
            })?;

        self.persist_default_tenant_if_user(subject, tenant_id)
            .await?;

        Ok(TenantSelection {
            is_default: true,
            ..selection
        })
    }

    async fn default_tenant_id_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>> {
        let Some(user_id) = parse_subject_user_id(subject) else {
            return Ok(None);
        };

        self.user_repository.default_tenant_id(user_id).await
    }

    async fn persist_default_tenant_if_user(
        &self,
        subject: &str,
        tenant_id: TenantId,
    ) -> AppResult<()> {
        let Some(user_id) = parse_subject_user_id(subject) else {
            return Ok(());
        };

        self.user_repository
            .set_default_tenant_id(user_id, tenant_id)
            .await
    }
}

fn parse_subject_user_id(subject: &str) -> Option<qryvanta_domain::UserId> {
    uuid::Uuid::parse_str(subject)
        .ok()
        .map(qryvanta_domain::UserId::from_uuid)
}

#[cfg(test)]
mod tests;
