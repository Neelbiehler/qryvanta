use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityBinding, AppEntityRolePermission, Permission, RuntimeRecord,
};

use crate::{
    AppRepository, AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService,
    CreateAppInput, RecordListQuery, RuntimeFieldGrant, RuntimeRecordLogicalMode,
    RuntimeRecordQuery, RuntimeRecordService, SubjectEntityPermission, TemporaryPermissionGrant,
};

use super::AppService;

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

#[derive(Default)]
struct FakeAppRepository {
    bindings: Mutex<HashMap<(TenantId, String), Vec<AppEntityBinding>>>,
    subject_permissions: Mutex<HashMap<(TenantId, String, String), Vec<SubjectEntityPermission>>>,
    subject_access: Mutex<HashMap<(TenantId, String, String), bool>>,
}

#[async_trait]
impl AppRepository for FakeAppRepository {
    async fn create_app(&self, _tenant_id: TenantId, _app: AppDefinition) -> AppResult<()> {
        Ok(())
    }

    async fn list_apps(&self, _tenant_id: TenantId) -> AppResult<Vec<AppDefinition>> {
        Ok(Vec::new())
    }

    async fn find_app(
        &self,
        _tenant_id: TenantId,
        _app_logical_name: &str,
    ) -> AppResult<Option<AppDefinition>> {
        Ok(Some(AppDefinition::new("sales", "Sales", None)?))
    }

    async fn save_app_entity_binding(
        &self,
        _tenant_id: TenantId,
        _binding: AppEntityBinding,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn list_app_entity_bindings(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>> {
        Ok(self
            .bindings
            .lock()
            .await
            .get(&(tenant_id, app_logical_name.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn save_app_role_entity_permission(
        &self,
        _tenant_id: TenantId,
        _permission: AppEntityRolePermission,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn list_app_role_entity_permissions(
        &self,
        _tenant_id: TenantId,
        _app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityRolePermission>> {
        Ok(Vec::new())
    }

    async fn list_accessible_apps(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
    ) -> AppResult<Vec<AppDefinition>> {
        Ok(Vec::new())
    }

    async fn subject_can_access_app(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<bool> {
        Ok(*self
            .subject_access
            .lock()
            .await
            .get(&(tenant_id, subject.to_owned(), app_logical_name.to_owned()))
            .unwrap_or(&false))
    }

    async fn subject_entity_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<Option<SubjectEntityPermission>> {
        Ok(self
            .subject_permissions
            .lock()
            .await
            .get(&(tenant_id, subject.to_owned(), app_logical_name.to_owned()))
            .and_then(|permissions| {
                permissions
                    .iter()
                    .find(|permission| permission.entity_logical_name == entity_logical_name)
                    .cloned()
            }))
    }

    async fn list_subject_entity_permissions(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<Vec<SubjectEntityPermission>> {
        Ok(self
            .subject_permissions
            .lock()
            .await
            .get(&(tenant_id, subject.to_owned(), app_logical_name.to_owned()))
            .cloned()
            .unwrap_or_default())
    }
}

#[derive(Default)]
struct FakeRuntimeRecordService {
    create_calls: Mutex<usize>,
    query_calls: Mutex<usize>,
}

#[async_trait]
impl RuntimeRecordService for FakeRuntimeRecordService {
    async fn latest_published_schema_unchecked(
        &self,
        _actor: &UserIdentity,
        _entity_logical_name: &str,
    ) -> AppResult<Option<qryvanta_domain::PublishedEntitySchema>> {
        Ok(None)
    }

    async fn list_runtime_records_unchecked(
        &self,
        _actor: &UserIdentity,
        _entity_logical_name: &str,
        _query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        Ok(Vec::new())
    }

    async fn query_runtime_records_unchecked(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
        _query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let mut calls = self.query_calls.lock().await;
        *calls += 1;

        Ok(vec![RuntimeRecord::new(
            "record-1",
            entity_logical_name,
            json!({"id": "record-1"}),
        )?])
    }

    async fn get_runtime_record_unchecked(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        RuntimeRecord::new(record_id, entity_logical_name, json!({"id": record_id}))
    }

    async fn create_runtime_record_unchecked(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        let mut calls = self.create_calls.lock().await;
        *calls += 1;
        RuntimeRecord::new("record-1", entity_logical_name, data)
    }

    async fn update_runtime_record_unchecked(
        &self,
        _actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        RuntimeRecord::new(record_id, entity_logical_name, data)
    }

    async fn delete_runtime_record_unchecked(
        &self,
        _actor: &UserIdentity,
        _entity_logical_name: &str,
        _record_id: &str,
    ) -> AppResult<()> {
        Ok(())
    }
}

fn actor(tenant_id: TenantId, subject: &str) -> UserIdentity {
    UserIdentity::new(subject, subject, None, tenant_id)
}

fn build_service(
    grants: HashMap<(TenantId, String), Vec<Permission>>,
    app_repository: Arc<FakeAppRepository>,
    runtime_record_service: Arc<FakeRuntimeRecordService>,
) -> AppService {
    let audit_repository = Arc::new(FakeAuditRepository::default());
    let authorization_service = AuthorizationService::new(
        Arc::new(FakeAuthorizationRepository { grants }),
        audit_repository.clone(),
    );
    AppService::new(
        authorization_service,
        app_repository,
        runtime_record_service,
        audit_repository,
    )
}

#[tokio::test]
async fn create_app_requires_manage_permission() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "alice");
    let service = build_service(
        HashMap::new(),
        Arc::new(FakeAppRepository::default()),
        Arc::new(FakeRuntimeRecordService::default()),
    );

    let result = service
        .create_app(
            &actor,
            CreateAppInput {
                logical_name: "sales".to_owned(),
                display_name: "Sales".to_owned(),
                description: None,
            },
        )
        .await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[tokio::test]
async fn app_navigation_only_includes_readable_entities() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "worker");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::new(),
        app_repository.clone(),
        runtime_record_service,
    );

    app_repository
        .subject_access
        .lock()
        .await
        .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new("sales", "account", None, 0).unwrap_or_else(|_| unreachable!()),
            AppEntityBinding::new("sales", "invoice", None, 1).unwrap_or_else(|_| unreachable!()),
        ],
    );

    app_repository.subject_permissions.lock().await.insert(
        (tenant_id, "worker".to_owned(), "sales".to_owned()),
        vec![
            SubjectEntityPermission {
                entity_logical_name: "account".to_owned(),
                can_read: true,
                can_create: false,
                can_update: false,
                can_delete: false,
            },
            SubjectEntityPermission {
                entity_logical_name: "invoice".to_owned(),
                can_read: false,
                can_create: true,
                can_update: false,
                can_delete: false,
            },
        ],
    );

    let navigation = service.app_navigation_for_subject(&actor, "sales").await;

    assert!(navigation.is_ok());
    let navigation = navigation.unwrap_or_default();
    assert_eq!(navigation.len(), 1);
    assert_eq!(navigation[0].entity_logical_name().as_str(), "account");
}

#[tokio::test]
async fn create_record_is_forbidden_without_create_capability() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "worker");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::new(),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository
        .subject_access
        .lock()
        .await
        .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);
    app_repository.subject_permissions.lock().await.insert(
        (tenant_id, "worker".to_owned(), "sales".to_owned()),
        vec![SubjectEntityPermission {
            entity_logical_name: "account".to_owned(),
            can_read: true,
            can_create: false,
            can_update: false,
            can_delete: false,
        }],
    );

    let result = service
        .create_record(&actor, "sales", "account", json!({"name": "A"}))
        .await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
    assert_eq!(*runtime_record_service.create_calls.lock().await, 0);
}

#[tokio::test]
async fn create_record_calls_runtime_when_create_capability_exists() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "worker");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::new(),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository
        .subject_access
        .lock()
        .await
        .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);
    app_repository.subject_permissions.lock().await.insert(
        (tenant_id, "worker".to_owned(), "sales".to_owned()),
        vec![SubjectEntityPermission {
            entity_logical_name: "account".to_owned(),
            can_read: true,
            can_create: true,
            can_update: false,
            can_delete: false,
        }],
    );

    let created = service
        .create_record(&actor, "sales", "account", json!({"name": "A"}))
        .await;

    assert!(created.is_ok());
    let created = created.unwrap_or_else(|_| unreachable!());
    assert_eq!(created.entity_logical_name().as_str(), "account");
    assert_eq!(*runtime_record_service.create_calls.lock().await, 1);
}

#[tokio::test]
async fn query_records_is_forbidden_without_read_capability() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "worker");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::new(),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository
        .subject_access
        .lock()
        .await
        .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);
    app_repository.subject_permissions.lock().await.insert(
        (tenant_id, "worker".to_owned(), "sales".to_owned()),
        vec![SubjectEntityPermission {
            entity_logical_name: "account".to_owned(),
            can_read: false,
            can_create: true,
            can_update: false,
            can_delete: false,
        }],
    );

    let result = service
        .query_records(
            &actor,
            "sales",
            "account",
            RuntimeRecordQuery {
                limit: 25,
                offset: 0,
                logical_mode: RuntimeRecordLogicalMode::And,
                where_clause: None,
                filters: Vec::new(),
                links: Vec::new(),
                sort: Vec::new(),
                owner_subject: None,
            },
        )
        .await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
    assert_eq!(*runtime_record_service.query_calls.lock().await, 0);
}

#[tokio::test]
async fn query_records_calls_runtime_when_read_capability_exists() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "worker");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::new(),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository
        .subject_access
        .lock()
        .await
        .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);
    app_repository.subject_permissions.lock().await.insert(
        (tenant_id, "worker".to_owned(), "sales".to_owned()),
        vec![SubjectEntityPermission {
            entity_logical_name: "account".to_owned(),
            can_read: true,
            can_create: false,
            can_update: false,
            can_delete: false,
        }],
    );

    let result = service
        .query_records(
            &actor,
            "sales",
            "account",
            RuntimeRecordQuery {
                limit: 25,
                offset: 0,
                logical_mode: RuntimeRecordLogicalMode::And,
                where_clause: None,
                filters: Vec::new(),
                links: Vec::new(),
                sort: Vec::new(),
                owner_subject: None,
            },
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(*runtime_record_service.query_calls.lock().await, 1);
}
