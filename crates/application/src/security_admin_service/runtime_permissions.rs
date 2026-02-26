use super::*;

use qryvanta_domain::AuditAction;

use crate::AuditEvent;
use crate::security_admin_ports::{RuntimeFieldPermissionEntry, SaveRuntimeFieldPermissionsInput};

impl SecurityAdminService {
    /// Saves runtime field-level permissions for a subject and entity.
    pub async fn save_runtime_field_permissions(
        &self,
        actor: &UserIdentity,
        input: SaveRuntimeFieldPermissionsInput,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        self.require_role_manage_permission(actor).await?;

        let entries = self
            .repository
            .save_runtime_field_permissions(actor.tenant_id(), input.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityRuntimeFieldPermissionsSaved,
                resource_type: "runtime_subject_field_permissions".to_owned(),
                resource_id: format!("{}:{}", input.subject, input.entity_logical_name),
                detail: Some(format!(
                    "saved {} runtime field permission entries for subject '{}' and entity '{}'",
                    entries.len(),
                    input.subject,
                    input.entity_logical_name
                )),
            })
            .await?;

        Ok(entries)
    }

    /// Lists runtime field permission entries in tenant scope.
    pub async fn list_runtime_field_permissions(
        &self,
        actor: &UserIdentity,
        subject: Option<&str>,
        entity_logical_name: Option<&str>,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        self.require_role_manage_permission(actor).await?;

        self.repository
            .list_runtime_field_permissions(actor.tenant_id(), subject, entity_logical_name)
            .await
    }
}
