use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppResult, TenantId, UserIdentity};
use qryvanta_domain::{AuditAction, Permission};

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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::sync::Mutex;

    use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
    use qryvanta_domain::Permission;

    use crate::{AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService};

    use super::{
        AuditLogEntry, AuditLogQuery, AuditLogRepository, CreateRoleInput, RoleAssignment,
        RoleDefinition, SecurityAdminRepository, SecurityAdminService,
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
    }

    #[derive(Default)]
    struct FakeSecurityAdminRepository {
        roles: Mutex<Vec<RoleDefinition>>,
        assignments: Mutex<Vec<(TenantId, String, String)>>,
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
        let authorization_service =
            AuthorizationService::new(Arc::new(FakeAuthorizationRepository {
                grants: HashMap::from([((tenant_id, subject.to_owned()), permissions)]),
            }));
        let audit_repository = Arc::new(FakeAuditRepository::default());
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
}
