use async_trait::async_trait;

use qryvanta_application::{AuthorizationRepository, RuntimeFieldGrant, TemporaryPermissionGrant};
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::Permission;

use sqlx::{FromRow, PgPool};

/// PostgreSQL-backed repository for subject permission lookups.
#[derive(Clone)]
pub struct PostgresAuthorizationRepository {
    pool: PgPool,
}

impl PostgresAuthorizationRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct PermissionRow {
    permission: String,
}

#[derive(Debug, FromRow)]
struct RuntimeFieldGrantRow {
    field_logical_name: String,
    can_read: bool,
    can_write: bool,
}

#[derive(Debug, FromRow)]
struct TemporaryPermissionGrantRow {
    grant_id: uuid::Uuid,
    reason: String,
    expires_at: String,
}

mod permissions;
mod runtime_fields;
mod temporary_grants;

#[async_trait]
impl AuthorizationRepository for PostgresAuthorizationRepository {
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>> {
        self.list_permissions_for_subject_impl(tenant_id, subject)
            .await
    }

    async fn list_runtime_field_grants_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        self.list_runtime_field_grants_for_subject_impl(tenant_id, subject, entity_logical_name)
            .await
    }

    async fn find_active_temporary_permission_grant(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>> {
        self.find_active_temporary_permission_grant_impl(tenant_id, subject, permission)
            .await
    }
}
