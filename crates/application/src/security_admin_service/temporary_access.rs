use super::*;

use qryvanta_domain::AuditAction;

use crate::AuditEvent;
use crate::security_admin_ports::{
    CreateTemporaryAccessGrantInput, TemporaryAccessGrant, TemporaryAccessGrantQuery,
};

impl SecurityAdminService {
    /// Creates a temporary privileged access grant.
    pub async fn create_temporary_access_grant(
        &self,
        actor: &UserIdentity,
        input: CreateTemporaryAccessGrantInput,
    ) -> AppResult<TemporaryAccessGrant> {
        self.require_role_manage_permission(actor).await?;

        if input.duration_minutes == 0 {
            return Err(qryvanta_core::AppError::Validation(
                "temporary access duration_minutes must be greater than zero".to_owned(),
            ));
        }

        let grant = self
            .repository
            .create_temporary_access_grant(actor.tenant_id(), actor.subject(), input)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityTemporaryAccessGranted,
                resource_type: "security_temporary_access_grant".to_owned(),
                resource_id: grant.grant_id.clone(),
                detail: Some(format!(
                    "granted temporary access to '{}' until '{}'",
                    grant.subject, grant.expires_at
                )),
            })
            .await?;

        Ok(grant)
    }

    /// Revokes a temporary privileged access grant.
    pub async fn revoke_temporary_access_grant(
        &self,
        actor: &UserIdentity,
        grant_id: &str,
        revoke_reason: Option<&str>,
    ) -> AppResult<()> {
        self.require_role_manage_permission(actor).await?;

        self.repository
            .revoke_temporary_access_grant(
                actor.tenant_id(),
                actor.subject(),
                grant_id,
                revoke_reason,
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityTemporaryAccessRevoked,
                resource_type: "security_temporary_access_grant".to_owned(),
                resource_id: grant_id.to_owned(),
                detail: revoke_reason
                    .map(|reason| format!("revoked temporary access grant: {reason}"))
                    .or(Some("revoked temporary access grant".to_owned())),
            })
            .await?;

        Ok(())
    }

    /// Lists temporary privileged access grants.
    pub async fn list_temporary_access_grants(
        &self,
        actor: &UserIdentity,
        query: TemporaryAccessGrantQuery,
    ) -> AppResult<Vec<TemporaryAccessGrant>> {
        self.require_role_manage_permission(actor).await?;

        self.repository
            .list_temporary_access_grants(actor.tenant_id(), query)
            .await
    }
}
