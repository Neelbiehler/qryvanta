use std::sync::Arc;

use qryvanta_core::{AppResult, UserIdentity};
use qryvanta_domain::{Permission, RegistrationMode};

use crate::security_admin_ports::{
    AuditLogRepository, SecurityAdminRepository, WorkspacePublishRunAuditInput,
};
use crate::{AuditRepository, AuthorizationService};

mod governance;
mod roles;
mod runtime_permissions;
mod temporary_access;

/// Application service for security administration workflows.
#[derive(Clone)]
pub struct SecurityAdminService {
    authorization_service: AuthorizationService,
    repository: Arc<dyn SecurityAdminRepository>,
    audit_log_repository: Arc<dyn AuditLogRepository>,
    audit_repository: Arc<dyn AuditRepository>,
}

impl SecurityAdminService {
    /// Creates a new service from required dependencies.
    #[must_use]
    pub fn new(
        authorization_service: AuthorizationService,
        repository: Arc<dyn SecurityAdminRepository>,
        audit_log_repository: Arc<dyn AuditLogRepository>,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            authorization_service,
            repository,
            audit_log_repository,
            audit_repository,
        }
    }

    pub(super) async fn require_role_manage_permission(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await
    }

    pub(super) async fn require_audit_read_permission(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityAuditRead,
            )
            .await
    }

    /// Appends a workspace publish run summary to the tenant audit log.
    pub async fn record_workspace_publish_run(
        &self,
        actor: &UserIdentity,
        input: WorkspacePublishRunAuditInput,
    ) -> AppResult<()> {
        self.record_workspace_publish_run_impl(actor, input).await
    }

    /// Returns tenant registration mode for administrative users.
    pub async fn registration_mode(&self, actor: &UserIdentity) -> AppResult<RegistrationMode> {
        self.registration_mode_impl(actor).await
    }

    /// Updates tenant registration mode and emits an audit event.
    pub async fn update_registration_mode(
        &self,
        actor: &UserIdentity,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode> {
        self.update_registration_mode_impl(actor, registration_mode)
            .await
    }
}

#[cfg(test)]
mod tests;
