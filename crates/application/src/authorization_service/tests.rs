use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{Permission, Surface};
use tokio::sync::Mutex;

use crate::{AuditEvent, AuditRepository};

use super::{
    AuthorizationRepository, AuthorizationService, RuntimeFieldGrant, TemporaryPermissionGrant,
};

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

struct FakeAuthorizationRepository {
    map: HashMap<(TenantId, String), Vec<Permission>>,
    runtime_field_grants: HashMap<(TenantId, String, String), Vec<RuntimeFieldGrant>>,
    temporary_permission_grants: HashMap<(TenantId, String, Permission), TemporaryPermissionGrant>,
}

#[async_trait]
impl AuthorizationRepository for FakeAuthorizationRepository {
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>> {
        Ok(self
            .map
            .get(&(tenant_id, subject.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn list_runtime_field_grants_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        Ok(self
            .runtime_field_grants
            .get(&(
                tenant_id,
                subject.to_owned(),
                entity_logical_name.to_owned(),
            ))
            .cloned()
            .unwrap_or_default())
    }

    async fn find_active_temporary_permission_grant(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>> {
        Ok(self
            .temporary_permission_grants
            .get(&(tenant_id, subject.to_owned(), permission))
            .cloned())
    }
}

#[tokio::test]
async fn require_permission_allows_granted_subject() {
    let tenant_id = TenantId::new();
    let repository = FakeAuthorizationRepository {
        map: HashMap::from([(
            (tenant_id, "alice".to_owned()),
            vec![Permission::MetadataEntityRead],
        )]),
        runtime_field_grants: HashMap::new(),
        temporary_permission_grants: HashMap::new(),
    };
    let service = AuthorizationService::new(
        Arc::new(repository),
        Arc::new(FakeAuditRepository::default()),
    );

    let result = service
        .require_permission(tenant_id, "alice", Permission::MetadataEntityRead)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn require_permission_denies_missing_grant() {
    let tenant_id = TenantId::new();
    let repository = FakeAuthorizationRepository {
        map: HashMap::new(),
        runtime_field_grants: HashMap::new(),
        temporary_permission_grants: HashMap::new(),
    };
    let service = AuthorizationService::new(
        Arc::new(repository),
        Arc::new(FakeAuditRepository::default()),
    );

    let result = service
        .require_permission(tenant_id, "alice", Permission::MetadataEntityCreate)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn resolve_accessible_surfaces_returns_matching_surfaces() {
    let tenant_id = TenantId::new();
    let repository = FakeAuthorizationRepository {
        map: HashMap::from([(
            (tenant_id, "alice".to_owned()),
            vec![
                Permission::SecurityRoleManage,
                Permission::MetadataEntityRead,
            ],
        )]),
        runtime_field_grants: HashMap::new(),
        temporary_permission_grants: HashMap::new(),
    };
    let service = AuthorizationService::new(
        Arc::new(repository),
        Arc::new(FakeAuditRepository::default()),
    );

    let surfaces = service
        .resolve_accessible_surfaces(tenant_id, "alice")
        .await;
    assert!(surfaces.is_ok());

    let surfaces = surfaces.unwrap_or_default();
    assert!(surfaces.contains(&Surface::Admin));
    assert!(surfaces.contains(&Surface::Maker));
    assert!(!surfaces.contains(&Surface::Worker));
}

#[tokio::test]
async fn resolve_accessible_surfaces_empty_for_no_permissions() {
    let tenant_id = TenantId::new();
    let repository = FakeAuthorizationRepository {
        map: HashMap::new(),
        runtime_field_grants: HashMap::new(),
        temporary_permission_grants: HashMap::new(),
    };
    let service = AuthorizationService::new(
        Arc::new(repository),
        Arc::new(FakeAuditRepository::default()),
    );

    let surfaces = service.resolve_accessible_surfaces(tenant_id, "bob").await;
    assert!(surfaces.is_ok());
    assert!(surfaces.unwrap_or_default().is_empty());
}

#[tokio::test]
async fn require_permission_allows_active_temporary_grant() {
    let tenant_id = TenantId::new();
    let repository = FakeAuthorizationRepository {
        map: HashMap::new(),
        runtime_field_grants: HashMap::new(),
        temporary_permission_grants: HashMap::from([(
            (
                tenant_id,
                "alice".to_owned(),
                Permission::RuntimeRecordWrite,
            ),
            TemporaryPermissionGrant {
                grant_id: "grant-1".to_owned(),
                reason: "incident response".to_owned(),
                expires_at: "2099-01-01T00:00:00Z".to_owned(),
            },
        )]),
    };
    let audit_repository = Arc::new(FakeAuditRepository::default());
    let service = AuthorizationService::new(Arc::new(repository), audit_repository.clone());

    let result = service
        .require_permission(tenant_id, "alice", Permission::RuntimeRecordWrite)
        .await;
    assert!(result.is_ok());

    let events = audit_repository.events.lock().await;
    assert_eq!(events.len(), 1);
}
