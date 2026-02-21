use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppResult, TenantId, UserIdentity};
use qryvanta_domain::{AuditAction, Permission, RegistrationMode};

use crate::{AuditEvent, AuditRepository, AuthorizationService};

/// Role definition returned to callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleDefinition {
    /// Stable role identifier.
    pub role_id: String,
    /// Unique role name in tenant scope.
    pub name: String,
    /// Indicates a system-managed role.
    pub is_system: bool,
    /// Effective role grants.
    pub permissions: Vec<Permission>,
}

/// Audit log entry projection for administrative views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLogEntry {
    /// Stable event identifier.
    pub event_id: String,
    /// Actor subject.
    pub subject: String,
    /// Stable action identifier.
    pub action: String,
    /// Event resource type.
    pub resource_type: String,
    /// Event resource identifier.
    pub resource_id: String,
    /// Optional event detail.
    pub detail: Option<String>,
    /// Event timestamp in RFC3339.
    pub created_at: String,
}

/// Assignment projection mapping a subject to a role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleAssignment {
    /// Subject identifier.
    pub subject: String,
    /// Role identifier.
    pub role_id: String,
    /// Role name.
    pub role_name: String,
    /// Assignment timestamp in RFC3339.
    pub assigned_at: String,
}

/// Query parameters for audit log listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLogQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
    /// Optional action filter.
    pub action: Option<String>,
    /// Optional subject filter.
    pub subject: Option<String>,
}

/// Input payload for creating custom roles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRoleInput {
    /// Unique role name in tenant scope.
    pub name: String,
    /// Grants to attach to the role.
    pub permissions: Vec<Permission>,
}

/// Field-level runtime permission update item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldPermissionInput {
    /// Field logical name.
    pub field_logical_name: String,
    /// Read access marker.
    pub can_read: bool,
    /// Write access marker.
    pub can_write: bool,
}

/// Input payload for subject runtime field permission updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveRuntimeFieldPermissionsInput {
    /// Subject principal identifier.
    pub subject: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Field permission entries to upsert.
    pub fields: Vec<RuntimeFieldPermissionInput>,
}

/// Runtime field permission projection returned to callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldPermissionEntry {
    /// Subject principal identifier.
    pub subject: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Field logical name.
    pub field_logical_name: String,
    /// Read access marker.
    pub can_read: bool,
    /// Write access marker.
    pub can_write: bool,
    /// Last update timestamp in RFC3339.
    pub updated_at: String,
}

/// Input payload for temporary access grants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTemporaryAccessGrantInput {
    /// Subject principal identifier.
    pub subject: String,
    /// Granted permissions.
    pub permissions: Vec<Permission>,
    /// Justification for temporary access.
    pub reason: String,
    /// Grant duration in minutes.
    pub duration_minutes: u32,
}

/// Temporary access grant projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryAccessGrant {
    /// Stable grant id.
    pub grant_id: String,
    /// Subject principal identifier.
    pub subject: String,
    /// Granted permissions.
    pub permissions: Vec<Permission>,
    /// Justification for temporary access.
    pub reason: String,
    /// Grant creator subject.
    pub created_by_subject: String,
    /// Expiration timestamp in RFC3339.
    pub expires_at: String,
    /// Revocation timestamp in RFC3339, when present.
    pub revoked_at: Option<String>,
}

/// Query parameters for temporary access grant listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryAccessGrantQuery {
    /// Optional subject filter.
    pub subject: Option<String>,
    /// Whether to return only active (non-revoked, non-expired) grants.
    pub active_only: bool,
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for pagination.
    pub offset: usize,
}

/// Audit retention policy projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditRetentionPolicy {
    /// Retention window in days.
    pub retention_days: u16,
}

/// Audit purge operation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditPurgeResult {
    /// Number of deleted entries.
    pub deleted_count: u64,
    /// Effective retention window in days.
    pub retention_days: u16,
}

/// Repository port for role and assignment administration.
#[async_trait]
pub trait SecurityAdminRepository: Send + Sync {
    /// Lists all tenant roles with effective grants.
    async fn list_roles(&self, tenant_id: TenantId) -> AppResult<Vec<RoleDefinition>>;

    /// Creates a role and attaches grants.
    async fn create_role(
        &self,
        tenant_id: TenantId,
        input: CreateRoleInput,
    ) -> AppResult<RoleDefinition>;

    /// Assigns an existing role to a subject.
    async fn assign_role_to_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()>;

    /// Removes a role assignment from a subject.
    async fn remove_role_from_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()>;

    /// Lists current role assignments in tenant scope.
    async fn list_role_assignments(&self, tenant_id: TenantId) -> AppResult<Vec<RoleAssignment>>;

    /// Saves runtime field permissions for a subject and entity.
    async fn save_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        input: SaveRuntimeFieldPermissionsInput,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>>;

    /// Lists runtime field permissions in tenant scope.
    async fn list_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        subject: Option<&str>,
        entity_logical_name: Option<&str>,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>>;

    /// Creates a temporary privileged access grant.
    async fn create_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        created_by_subject: &str,
        input: CreateTemporaryAccessGrantInput,
    ) -> AppResult<TemporaryAccessGrant>;

    /// Revokes a temporary privileged access grant.
    async fn revoke_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        revoked_by_subject: &str,
        grant_id: &str,
        revoke_reason: Option<&str>,
    ) -> AppResult<()>;

    /// Lists temporary privileged access grants.
    async fn list_temporary_access_grants(
        &self,
        tenant_id: TenantId,
        query: TemporaryAccessGrantQuery,
    ) -> AppResult<Vec<TemporaryAccessGrant>>;

    /// Returns the tenant registration mode.
    async fn registration_mode(&self, tenant_id: TenantId) -> AppResult<RegistrationMode>;

    /// Updates and returns tenant registration mode.
    async fn set_registration_mode(
        &self,
        tenant_id: TenantId,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode>;

    /// Returns tenant audit retention policy.
    async fn audit_retention_policy(&self, tenant_id: TenantId) -> AppResult<AuditRetentionPolicy>;

    /// Updates and returns tenant audit retention policy.
    async fn set_audit_retention_policy(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<AuditRetentionPolicy>;
}

/// Repository port for reading tenant audit logs.
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Lists most recent tenant audit entries.
    async fn list_recent_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>>;

    /// Exports tenant audit entries for operational workflows.
    async fn export_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>>;

    /// Purges tenant audit entries older than the retention window.
    async fn purge_entries_older_than(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<u64>;
}

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

    /// Returns tenant roles for administrative users.
    pub async fn list_roles(&self, actor: &UserIdentity) -> AppResult<Vec<RoleDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

        self.repository.list_roles(actor.tenant_id()).await
    }

    /// Creates a custom role and emits an audit event.
    pub async fn create_role(
        &self,
        actor: &UserIdentity,
        input: CreateRoleInput,
    ) -> AppResult<RoleDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

        self.repository
            .list_role_assignments(actor.tenant_id())
            .await
    }

    /// Saves runtime field-level permissions for a subject and entity.
    pub async fn save_runtime_field_permissions(
        &self,
        actor: &UserIdentity,
        input: SaveRuntimeFieldPermissionsInput,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

        self.repository
            .list_runtime_field_permissions(actor.tenant_id(), subject, entity_logical_name)
            .await
    }

    /// Creates a temporary privileged access grant.
    pub async fn create_temporary_access_grant(
        &self,
        actor: &UserIdentity,
        input: CreateTemporaryAccessGrantInput,
    ) -> AppResult<TemporaryAccessGrant> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

        self.repository
            .list_temporary_access_grants(actor.tenant_id(), query)
            .await
    }

    /// Returns recent audit entries.
    pub async fn list_audit_log(
        &self,
        actor: &UserIdentity,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityAuditRead,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityAuditRead,
            )
            .await?;

        self.audit_log_repository
            .export_entries(actor.tenant_id(), query)
            .await
    }

    /// Returns tenant registration mode for administrative users.
    pub async fn registration_mode(&self, actor: &UserIdentity) -> AppResult<RegistrationMode> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

        self.repository.registration_mode(actor.tenant_id()).await
    }

    /// Updates tenant registration mode and emits an audit event.
    pub async fn update_registration_mode(
        &self,
        actor: &UserIdentity,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await?;

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::sync::Mutex;

    use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
    use qryvanta_domain::{Permission, RegistrationMode};

    use crate::{
        AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService,
        RuntimeFieldGrant, TemporaryPermissionGrant,
    };

    use super::{
        AuditLogEntry, AuditLogQuery, AuditLogRepository, AuditRetentionPolicy, CreateRoleInput,
        CreateTemporaryAccessGrantInput, RoleAssignment, RoleDefinition,
        RuntimeFieldPermissionEntry, SaveRuntimeFieldPermissionsInput, SecurityAdminRepository,
        SecurityAdminService, TemporaryAccessGrant, TemporaryAccessGrantQuery,
    };

    struct FakeAuthorizationRepository {
        grants: HashMap<(TenantId, String), Vec<Permission>>,
    }

    #[async_trait]
    impl AuthorizationRepository for FakeAuthorizationRepository {
        async fn list_permissions_for_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
        ) -> AppResult<Vec<Permission>> {
            Ok(self
                .grants
                .get(&(tenant_id, subject.to_owned()))
                .cloned()
                .unwrap_or_default())
        }

        async fn list_runtime_field_grants_for_subject(
            &self,
            _tenant_id: TenantId,
            _subject: &str,
            _entity_logical_name: &str,
        ) -> AppResult<Vec<RuntimeFieldGrant>> {
            Ok(Vec::new())
        }

        async fn find_active_temporary_permission_grant(
            &self,
            _tenant_id: TenantId,
            _subject: &str,
            _permission: Permission,
        ) -> AppResult<Option<TemporaryPermissionGrant>> {
            Ok(None)
        }
    }

    struct FakeSecurityAdminRepository {
        roles: Mutex<Vec<RoleDefinition>>,
        assignments: Mutex<Vec<(TenantId, String, String)>>,
        registration_mode: Mutex<RegistrationMode>,
        audit_retention_days: Mutex<u16>,
    }

    impl Default for FakeSecurityAdminRepository {
        fn default() -> Self {
            Self {
                roles: Mutex::new(Vec::new()),
                assignments: Mutex::new(Vec::new()),
                registration_mode: Mutex::new(RegistrationMode::InviteOnly),
                audit_retention_days: Mutex::new(365),
            }
        }
    }

    #[async_trait]
    impl SecurityAdminRepository for FakeSecurityAdminRepository {
        async fn list_roles(&self, _tenant_id: TenantId) -> AppResult<Vec<RoleDefinition>> {
            Ok(self.roles.lock().await.clone())
        }

        async fn create_role(
            &self,
            _tenant_id: TenantId,
            input: CreateRoleInput,
        ) -> AppResult<RoleDefinition> {
            let role = RoleDefinition {
                role_id: "1".to_owned(),
                name: input.name,
                is_system: false,
                permissions: input.permissions,
            };
            self.roles.lock().await.push(role.clone());
            Ok(role)
        }

        async fn assign_role_to_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
            role_name: &str,
        ) -> AppResult<()> {
            self.assignments.lock().await.push((
                tenant_id,
                subject.to_owned(),
                role_name.to_owned(),
            ));
            Ok(())
        }

        async fn remove_role_from_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
            role_name: &str,
        ) -> AppResult<()> {
            let mut assignments = self.assignments.lock().await;
            assignments.retain(|(stored_tenant_id, stored_subject, stored_role_name)| {
                !(stored_tenant_id == &tenant_id
                    && stored_subject == subject
                    && stored_role_name == role_name)
            });
            Ok(())
        }

        async fn list_role_assignments(
            &self,
            _tenant_id: TenantId,
        ) -> AppResult<Vec<RoleAssignment>> {
            Ok(Vec::new())
        }

        async fn save_runtime_field_permissions(
            &self,
            _tenant_id: TenantId,
            _input: SaveRuntimeFieldPermissionsInput,
        ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
            Ok(Vec::new())
        }

        async fn list_runtime_field_permissions(
            &self,
            _tenant_id: TenantId,
            _subject: Option<&str>,
            _entity_logical_name: Option<&str>,
        ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
            Ok(Vec::new())
        }

        async fn create_temporary_access_grant(
            &self,
            _tenant_id: TenantId,
            created_by_subject: &str,
            input: CreateTemporaryAccessGrantInput,
        ) -> AppResult<TemporaryAccessGrant> {
            Ok(TemporaryAccessGrant {
                grant_id: "grant-1".to_owned(),
                subject: input.subject,
                permissions: input.permissions,
                reason: input.reason,
                created_by_subject: created_by_subject.to_owned(),
                expires_at: "2026-01-01T00:00:00Z".to_owned(),
                revoked_at: None,
            })
        }

        async fn revoke_temporary_access_grant(
            &self,
            _tenant_id: TenantId,
            _revoked_by_subject: &str,
            _grant_id: &str,
            _revoke_reason: Option<&str>,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn list_temporary_access_grants(
            &self,
            _tenant_id: TenantId,
            _query: TemporaryAccessGrantQuery,
        ) -> AppResult<Vec<TemporaryAccessGrant>> {
            Ok(Vec::new())
        }

        async fn registration_mode(&self, _tenant_id: TenantId) -> AppResult<RegistrationMode> {
            Ok(*self.registration_mode.lock().await)
        }

        async fn set_registration_mode(
            &self,
            _tenant_id: TenantId,
            registration_mode: RegistrationMode,
        ) -> AppResult<RegistrationMode> {
            let mut mode = self.registration_mode.lock().await;
            *mode = registration_mode;
            Ok(*mode)
        }

        async fn audit_retention_policy(
            &self,
            _tenant_id: TenantId,
        ) -> AppResult<AuditRetentionPolicy> {
            Ok(AuditRetentionPolicy {
                retention_days: *self.audit_retention_days.lock().await,
            })
        }

        async fn set_audit_retention_policy(
            &self,
            _tenant_id: TenantId,
            retention_days: u16,
        ) -> AppResult<AuditRetentionPolicy> {
            let mut stored_days = self.audit_retention_days.lock().await;
            *stored_days = retention_days;
            Ok(AuditRetentionPolicy {
                retention_days: *stored_days,
            })
        }
    }

    struct FakeAuditLogRepository {
        entries: Vec<AuditLogEntry>,
    }

    #[async_trait]
    impl AuditLogRepository for FakeAuditLogRepository {
        async fn list_recent_entries(
            &self,
            _tenant_id: TenantId,
            _query: AuditLogQuery,
        ) -> AppResult<Vec<AuditLogEntry>> {
            Ok(self.entries.clone())
        }

        async fn export_entries(
            &self,
            _tenant_id: TenantId,
            _query: AuditLogQuery,
        ) -> AppResult<Vec<AuditLogEntry>> {
            Ok(self.entries.clone())
        }

        async fn purge_entries_older_than(
            &self,
            _tenant_id: TenantId,
            _retention_days: u16,
        ) -> AppResult<u64> {
            Ok(0)
        }
    }

    #[derive(Default)]
    struct FakeAuditRepository {
        events: Mutex<Vec<AuditEvent>>,
    }

    #[async_trait]
    impl AuditRepository for FakeAuditRepository {
        async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
            self.events.lock().await.push(event);
            Ok(())
        }
    }

    fn actor(tenant_id: TenantId, subject: &str) -> UserIdentity {
        UserIdentity::new(subject, subject, None, tenant_id)
    }

    fn service_with_permissions(
        tenant_id: TenantId,
        subject: &str,
        permissions: Vec<Permission>,
    ) -> (SecurityAdminService, Arc<FakeAuditRepository>) {
        let audit_repository = Arc::new(FakeAuditRepository::default());
        let authorization_service = AuthorizationService::new(
            Arc::new(FakeAuthorizationRepository {
                grants: HashMap::from([((tenant_id, subject.to_owned()), permissions)]),
            }),
            audit_repository.clone(),
        );
        let service = SecurityAdminService::new(
            authorization_service,
            Arc::new(FakeSecurityAdminRepository::default()),
            Arc::new(FakeAuditLogRepository {
                entries: Vec::new(),
            }),
            audit_repository.clone(),
        );
        (service, audit_repository)
    }

    #[tokio::test]
    async fn create_role_requires_manage_permission() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "alice");
        let (service, _) = service_with_permissions(tenant_id, "alice", Vec::new());

        let result = service
            .create_role(
                &actor,
                CreateRoleInput {
                    name: "ops".to_owned(),
                    permissions: vec![Permission::RuntimeRecordRead],
                },
            )
            .await;

        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn create_role_writes_audit_event() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "alice");
        let (service, audit_repository) =
            service_with_permissions(tenant_id, "alice", vec![Permission::SecurityRoleManage]);

        let result = service
            .create_role(
                &actor,
                CreateRoleInput {
                    name: "ops".to_owned(),
                    permissions: vec![Permission::RuntimeRecordRead],
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(audit_repository.events.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn list_audit_log_requires_audit_permission() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "alice");
        let (service, _) =
            service_with_permissions(tenant_id, "alice", vec![Permission::SecurityRoleManage]);

        let result = service
            .list_audit_log(
                &actor,
                AuditLogQuery {
                    limit: 20,
                    offset: 0,
                    action: None,
                    subject: None,
                },
            )
            .await;

        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn unassign_role_requires_manage_permission() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "alice");
        let (service, _) = service_with_permissions(tenant_id, "alice", Vec::new());

        let result = service.unassign_role(&actor, "bob", "ops").await;

        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn update_registration_mode_requires_manage_permission() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "alice");
        let (service, _) = service_with_permissions(tenant_id, "alice", Vec::new());

        let result = service
            .update_registration_mode(&actor, RegistrationMode::Open)
            .await;

        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn update_registration_mode_writes_audit_event() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "alice");
        let (service, audit_repository) =
            service_with_permissions(tenant_id, "alice", vec![Permission::SecurityRoleManage]);

        let updated_mode = service
            .update_registration_mode(&actor, RegistrationMode::Open)
            .await;

        assert!(updated_mode.is_ok());
        assert_eq!(
            updated_mode.unwrap_or(RegistrationMode::InviteOnly),
            RegistrationMode::Open
        );

        let events = audit_repository.events.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].action,
            qryvanta_domain::AuditAction::SecurityTenantRegistrationModeUpdated
        );
    }
}
