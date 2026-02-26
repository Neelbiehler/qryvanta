use super::*;

use qryvanta_domain::AuditAction;

use crate::AuditEvent;
use crate::security_admin_ports::{CreateRoleInput, RoleAssignment, RoleDefinition};

impl SecurityAdminService {
    /// Returns tenant roles for administrative users.
    pub async fn list_roles(&self, actor: &UserIdentity) -> AppResult<Vec<RoleDefinition>> {
        self.require_role_manage_permission(actor).await?;
        self.repository.list_roles(actor.tenant_id()).await
    }

    /// Creates a custom role and emits an audit event.
    pub async fn create_role(
        &self,
        actor: &UserIdentity,
        input: CreateRoleInput,
    ) -> AppResult<RoleDefinition> {
        self.require_role_manage_permission(actor).await?;

        let role = self
            .repository
            .create_role(actor.tenant_id(), input)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityRoleCreated,
                resource_type: "rbac_role".to_owned(),
                resource_id: role.name.clone(),
                detail: Some(format!("created role '{}'", role.name)),
            })
            .await?;

        Ok(role)
    }

    /// Assigns a role to a subject and emits an audit event.
    pub async fn assign_role(
        &self,
        actor: &UserIdentity,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()> {
        self.require_role_manage_permission(actor).await?;

        self.repository
            .assign_role_to_subject(actor.tenant_id(), subject, role_name)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityRoleAssigned,
                resource_type: "rbac_subject_role".to_owned(),
                resource_id: format!("{subject}:{role_name}"),
                detail: Some(format!("assigned role '{role_name}' to '{subject}'")),
            })
            .await
    }

    /// Removes a role assignment from a subject and emits an audit event.
    pub async fn unassign_role(
        &self,
        actor: &UserIdentity,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()> {
        self.require_role_manage_permission(actor).await?;

        self.repository
            .remove_role_from_subject(actor.tenant_id(), subject, role_name)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityRoleUnassigned,
                resource_type: "rbac_subject_role".to_owned(),
                resource_id: format!("{subject}:{role_name}"),
                detail: Some(format!("removed role '{role_name}' from '{subject}'")),
            })
            .await
    }

    /// Returns role assignments for administrative users.
    pub async fn list_role_assignments(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<Vec<RoleAssignment>> {
        self.require_role_manage_permission(actor).await?;

        self.repository
            .list_role_assignments(actor.tenant_id())
            .await
    }
}
