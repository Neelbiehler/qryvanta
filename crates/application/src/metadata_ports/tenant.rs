use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::RegistrationMode;

/// One subject membership in a tenant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantMembership {
    /// Tenant identifier.
    pub tenant_id: TenantId,
    /// Tenant display name.
    pub tenant_name: String,
    /// Membership-specific display name for the subject.
    pub display_name: String,
    /// Optional membership email.
    pub email: Option<String>,
}

/// Port for tenant membership and subject-contact mapping operations.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Finds tenant membership for a subject.
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>>;

    /// Returns tenant registration mode.
    async fn registration_mode_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<RegistrationMode>;

    /// Creates membership for an existing tenant.
    async fn create_membership(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()>;

    /// Ensures subject membership exists, creating tenant and membership as needed.
    async fn ensure_membership_for_subject(
        &self,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId>;

    /// Lists every tenant membership for a subject.
    async fn list_memberships_for_subject(&self, subject: &str)
    -> AppResult<Vec<TenantMembership>>;

    /// Returns contact record mapping for a subject.
    async fn contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Option<String>>;

    /// Persists contact record mapping for a subject.
    async fn save_contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        contact_record_id: &str,
    ) -> AppResult<()>;
}
