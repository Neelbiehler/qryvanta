use qryvanta_core::AppError;
use qryvanta_domain::{AuditAction, Permission};

use crate::AuditEvent;

use super::*;

impl AuthorizationService {
    /// Ensures a subject has the required permission in the tenant scope.
    pub async fn require_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<()> {
        match self
            .resolve_permission_grant(tenant_id, subject, permission)
            .await?
        {
            PermissionGrantResolution::Granted => Ok(()),
            PermissionGrantResolution::Temporary(grant) => {
                self.append_temporary_access_use_event(tenant_id, subject, permission, &grant)
                    .await
            }
            PermissionGrantResolution::Missing => Err(AppError::Forbidden(format!(
                "subject '{subject}' is missing permission '{}' in tenant '{tenant_id}'",
                permission.as_str()
            ))),
        }
    }

    /// Returns whether the subject currently has the permission.
    pub async fn has_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<bool> {
        match self
            .resolve_permission_grant(tenant_id, subject, permission)
            .await?
        {
            PermissionGrantResolution::Granted => Ok(true),
            PermissionGrantResolution::Temporary(grant) => {
                self.append_temporary_access_use_event(tenant_id, subject, permission, &grant)
                    .await?;
                Ok(true)
            }
            PermissionGrantResolution::Missing => Ok(false),
        }
    }

    async fn resolve_permission_grant(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<PermissionGrantResolution> {
        let permissions = self
            .repository
            .list_permissions_for_subject(tenant_id, subject)
            .await?;

        if permissions.iter().any(|value| value == &permission) {
            return Ok(PermissionGrantResolution::Granted);
        }

        let temporary_grant = self
            .repository
            .find_active_temporary_permission_grant(tenant_id, subject, permission)
            .await?;

        Ok(temporary_grant
            .map(PermissionGrantResolution::Temporary)
            .unwrap_or(PermissionGrantResolution::Missing))
    }

    async fn append_temporary_access_use_event(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
        grant: &TemporaryPermissionGrant,
    ) -> AppResult<()> {
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id,
                subject: subject.to_owned(),
                action: AuditAction::SecurityTemporaryAccessUsed,
                resource_type: "security_temporary_access_grant".to_owned(),
                resource_id: grant.grant_id.clone(),
                detail: Some(format!(
                    "used temporary grant '{}' for permission '{}' (expires_at='{}', reason='{}')",
                    grant.grant_id,
                    permission.as_str(),
                    grant.expires_at,
                    grant.reason
                )),
            })
            .await
    }
}
