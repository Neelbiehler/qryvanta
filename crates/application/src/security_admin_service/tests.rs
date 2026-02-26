use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{Permission, RegistrationMode};

use crate::security_admin_ports::{
    AuditLogEntry, AuditLogQuery, AuditLogRepository, AuditRetentionPolicy, CreateRoleInput,
    CreateTemporaryAccessGrantInput, RoleAssignment, RoleDefinition, RuntimeFieldPermissionEntry,
    SaveRuntimeFieldPermissionsInput, SecurityAdminRepository, TemporaryAccessGrant,
    TemporaryAccessGrantQuery, WorkspacePublishRunAuditInput,
};
use crate::{
    AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService, RuntimeFieldGrant,
    TemporaryPermissionGrant,
};

use super::SecurityAdminService;

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
        self.assignments
            .lock()
            .await
            .push((tenant_id, subject.to_owned(), role_name.to_owned()));
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

    async fn list_role_assignments(&self, _tenant_id: TenantId) -> AppResult<Vec<RoleAssignment>> {
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
async fn record_workspace_publish_run_writes_audit_event() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "alice");
    let (service, audit_repository) =
        service_with_permissions(tenant_id, "alice", vec![Permission::SecurityRoleManage]);

    let result = service
        .record_workspace_publish_run(
            &actor,
            WorkspacePublishRunAuditInput {
                requested_entities: 2,
                requested_apps: 1,
                requested_entity_logical_names: vec!["contact".to_owned(), "account".to_owned()],
                requested_app_logical_names: vec!["sales".to_owned()],
                published_entities: vec!["contact".to_owned(), "account".to_owned()],
                validated_apps: vec!["sales".to_owned()],
                issue_count: 0,
                is_publishable: true,
            },
        )
        .await;

    assert!(result.is_ok());

    let events = audit_repository.events.lock().await;
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].action,
        qryvanta_domain::AuditAction::MetadataWorkspacePublished
    );
    assert_eq!(events[0].resource_type, "workspace_publish_run");
    assert!(events[0].detail.is_some());
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
