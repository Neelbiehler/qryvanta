use std::sync::Arc;

use qryvanta_application::TenantRepository;
use qryvanta_infrastructure::{
    PostgresAppRepository, PostgresAuditLogRepository, PostgresAuditRepository,
    PostgresAuthEventRepository, PostgresAuthorizationRepository, PostgresMetadataRepository,
    PostgresPasskeyRepository, PostgresSecurityAdminRepository, PostgresTenantRepository,
    PostgresUserRepository, PostgresWorkflowRepository,
};
use sqlx::PgPool;

pub(super) struct RepositorySet {
    pub(super) metadata_repository: Arc<PostgresMetadataRepository>,
    pub(super) app_repository: Arc<PostgresAppRepository>,
    pub(super) workflow_repository: Arc<PostgresWorkflowRepository>,
    pub(super) audit_repository: Arc<PostgresAuditRepository>,
    pub(super) authorization_repository: Arc<PostgresAuthorizationRepository>,
    pub(super) security_admin_repository: Arc<PostgresSecurityAdminRepository>,
    pub(super) audit_log_repository: Arc<PostgresAuditLogRepository>,
    pub(super) auth_event_repository: Arc<PostgresAuthEventRepository>,
    pub(super) tenant_repository: Arc<dyn TenantRepository>,
    pub(super) passkey_repository: PostgresPasskeyRepository,
    pub(super) user_repository: Arc<PostgresUserRepository>,
}

pub(super) fn build_repository_set(pool: &PgPool) -> RepositorySet {
    RepositorySet {
        metadata_repository: Arc::new(PostgresMetadataRepository::new(pool.clone())),
        app_repository: Arc::new(PostgresAppRepository::new(pool.clone())),
        workflow_repository: Arc::new(PostgresWorkflowRepository::new(pool.clone())),
        audit_repository: Arc::new(PostgresAuditRepository::new(pool.clone())),
        authorization_repository: Arc::new(PostgresAuthorizationRepository::new(pool.clone())),
        security_admin_repository: Arc::new(PostgresSecurityAdminRepository::new(pool.clone())),
        audit_log_repository: Arc::new(PostgresAuditLogRepository::new(pool.clone())),
        auth_event_repository: Arc::new(PostgresAuthEventRepository::new(pool.clone())),
        tenant_repository: Arc::new(PostgresTenantRepository::new(pool.clone())),
        passkey_repository: PostgresPasskeyRepository::new(pool.clone()),
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
    }
}
