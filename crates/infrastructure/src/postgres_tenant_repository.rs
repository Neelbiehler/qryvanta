use async_trait::async_trait;
use qryvanta_application::TenantRepository;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::RegistrationMode;
use sqlx::PgPool;

use crate::postgres_security_admin_repository::assign_owner_role_grants;

/// PostgreSQL-backed tenant membership repository.
#[derive(Clone)]
pub struct PostgresTenantRepository {
    pool: PgPool,
}

impl PostgresTenantRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

mod contacts;
mod lookup;
mod membership;

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>> {
        self.find_tenant_for_subject_impl(subject).await
    }

    async fn registration_mode_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<RegistrationMode> {
        self.registration_mode_for_tenant_impl(tenant_id).await
    }

    async fn create_membership(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()> {
        self.create_membership_impl(tenant_id, subject, display_name, email)
            .await
    }

    async fn ensure_membership_for_subject(
        &self,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId> {
        self.ensure_membership_for_subject_impl(subject, display_name, email, preferred_tenant_id)
            .await
    }

    async fn contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Option<String>> {
        self.contact_record_for_subject_impl(tenant_id, subject)
            .await
    }

    async fn save_contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        contact_record_id: &str,
    ) -> AppResult<()> {
        self.save_contact_record_for_subject_impl(tenant_id, subject, contact_record_id)
            .await
    }
}
