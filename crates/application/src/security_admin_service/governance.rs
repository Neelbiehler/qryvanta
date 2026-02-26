use super::*;

use qryvanta_domain::AuditAction;

use crate::AuditEvent;
use crate::security_admin_ports::{
    AuditLogEntry, AuditLogQuery, AuditPurgeResult, AuditRetentionPolicy,
    WorkspacePublishRunAuditInput,
};

impl SecurityAdminService {
    /// Returns recent audit entries.
    pub async fn list_audit_log(
        &self,
        actor: &UserIdentity,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        self.require_audit_read_permission(actor).await?;
        self.audit_log_repository
            .list_recent_entries(actor.tenant_id(), query)
            .await
    }

    /// Exports tenant audit entries for operational workflows.
    pub async fn export_audit_log(
        &self,
        actor: &UserIdentity,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        self.require_audit_read_permission(actor).await?;
        self.audit_log_repository
            .export_entries(actor.tenant_id(), query)
            .await
    }

    pub(super) async fn record_workspace_publish_run_impl(
        &self,
        actor: &UserIdentity,
        input: WorkspacePublishRunAuditInput,
    ) -> AppResult<()> {
        self.require_role_manage_permission(actor).await?;

        let detail = serde_json::json!({
            "requested_entities": input.requested_entities,
            "requested_apps": input.requested_apps,
            "requested_entity_logical_names": input.requested_entity_logical_names,
            "requested_app_logical_names": input.requested_app_logical_names,
            "published_entities": input.published_entities,
            "validated_apps": input.validated_apps,
            "issue_count": input.issue_count,
            "is_publishable": input.is_publishable,
        })
        .to_string();

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataWorkspacePublished,
                resource_type: "workspace_publish_run".to_owned(),
                resource_id: format!("{}-{}", actor.subject(), chrono::Utc::now().timestamp()),
                detail: Some(detail),
            })
            .await
    }

    pub(super) async fn registration_mode_impl(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<RegistrationMode> {
        self.require_role_manage_permission(actor).await?;
        self.repository.registration_mode(actor.tenant_id()).await
    }

    pub(super) async fn update_registration_mode_impl(
        &self,
        actor: &UserIdentity,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode> {
        self.require_role_manage_permission(actor).await?;

        let updated_mode = self
            .repository
            .set_registration_mode(actor.tenant_id(), registration_mode)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityTenantRegistrationModeUpdated,
                resource_type: "tenant".to_owned(),
                resource_id: actor.tenant_id().to_string(),
                detail: Some(format!(
                    "set tenant registration mode to '{}'",
                    updated_mode.as_str()
                )),
            })
            .await?;

        Ok(updated_mode)
    }

    /// Returns tenant audit retention policy for administrative users.
    pub async fn audit_retention_policy(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<AuditRetentionPolicy> {
        self.require_role_manage_permission(actor).await?;
        self.repository
            .audit_retention_policy(actor.tenant_id())
            .await
    }

    /// Updates tenant audit retention policy and emits an audit event.
    pub async fn update_audit_retention_policy(
        &self,
        actor: &UserIdentity,
        retention_days: u16,
    ) -> AppResult<AuditRetentionPolicy> {
        self.require_role_manage_permission(actor).await?;

        if retention_days == 0 {
            return Err(qryvanta_core::AppError::Validation(
                "audit retention_days must be greater than zero".to_owned(),
            ));
        }

        let policy = self
            .repository
            .set_audit_retention_policy(actor.tenant_id(), retention_days)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityAuditRetentionUpdated,
                resource_type: "tenant".to_owned(),
                resource_id: actor.tenant_id().to_string(),
                detail: Some(format!(
                    "set audit retention policy to {} day(s)",
                    policy.retention_days
                )),
            })
            .await?;

        Ok(policy)
    }

    /// Purges audit entries older than the configured retention policy.
    pub async fn purge_audit_log_entries(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<AuditPurgeResult> {
        self.require_role_manage_permission(actor).await?;

        let policy = self
            .repository
            .audit_retention_policy(actor.tenant_id())
            .await?;
        let deleted_count = self
            .audit_log_repository
            .purge_entries_older_than(actor.tenant_id(), policy.retention_days)
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::SecurityAuditEntriesPurged,
                resource_type: "audit_log_entries".to_owned(),
                resource_id: actor.tenant_id().to_string(),
                detail: Some(format!(
                    "purged {} audit entries older than {} day(s)",
                    deleted_count, policy.retention_days
                )),
            })
            .await?;

        Ok(AuditPurgeResult {
            deleted_count,
            retention_days: policy.retention_days,
        })
    }
}
