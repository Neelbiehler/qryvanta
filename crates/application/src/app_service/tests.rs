use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityBinding, AppEntityForm, AppEntityRolePermission, AppEntityView,
    AppEntityViewMode, AppSitemap, FormDefinition, FormFieldPlacement, FormSection, FormTab,
    FormType, Permission, RuntimeRecord, SitemapArea, SitemapGroup, SitemapSubArea, SitemapTarget,
    ViewColumn, ViewDefinition, ViewType,
};

use crate::{
    AppEntityFormInput, AppEntityViewInput, AppRepository, AuditEvent, AuditRepository,
    AuthorizationRepository, AuthorizationService, BindAppEntityInput, CreateAppInput,
    RecordListQuery, RuntimeFieldGrant, RuntimeRecordLogicalMode, RuntimeRecordQuery,
    RuntimeRecordService, SaveAppSitemapInput, SubjectEntityPermission, TemporaryPermissionGrant,
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
    sitemaps: Mutex<HashMap<(TenantId, String), AppSitemap>>,
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

    async fn save_sitemap(&self, tenant_id: TenantId, sitemap: AppSitemap) -> AppResult<()> {
        self.sitemaps.lock().await.insert(
            (tenant_id, sitemap.app_logical_name().as_str().to_owned()),
            sitemap,
        );
        Ok(())
    }

    async fn get_sitemap(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppSitemap>> {
        Ok(self
            .sitemaps
            .lock()
            .await
            .get(&(tenant_id, app_logical_name.to_owned()))
            .cloned())
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
    forms: Mutex<HashMap<(TenantId, String), Vec<FormDefinition>>>,
    views: Mutex<HashMap<(TenantId, String), Vec<ViewDefinition>>>,
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

    async fn list_forms_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<qryvanta_domain::FormDefinition>> {
        Ok(self
            .forms
            .lock()
            .await
            .get(&(actor.tenant_id(), entity_logical_name.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn find_form_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<qryvanta_domain::FormDefinition>> {
        Ok(self
            .forms
            .lock()
            .await
            .get(&(actor.tenant_id(), entity_logical_name.to_owned()))
            .and_then(|forms| {
                forms
                    .iter()
                    .find(|form| form.logical_name().as_str() == form_logical_name)
                    .cloned()
            }))
    }

    async fn list_views_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<qryvanta_domain::ViewDefinition>> {
        Ok(self
            .views
            .lock()
            .await
            .get(&(actor.tenant_id(), entity_logical_name.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn find_view_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<qryvanta_domain::ViewDefinition>> {
        Ok(self
            .views
            .lock()
            .await
            .get(&(actor.tenant_id(), entity_logical_name.to_owned()))
            .and_then(|views| {
                views
                    .iter()
                    .find(|view| view.logical_name().as_str() == view_logical_name)
                    .cloned()
            }))
    }
}

fn minimal_form(entity_logical_name: &str, form_logical_name: &str) -> FormDefinition {
    let field = FormFieldPlacement::new("name", 0, 0, true, false, None, None)
        .unwrap_or_else(|_| unreachable!());
    let section = FormSection::new(
        "main_section",
        "Main Section",
        0,
        true,
        1,
        vec![field],
        vec![],
    )
    .unwrap_or_else(|_| unreachable!());
    let tab = FormTab::new("main_tab", "Main Tab", 0, true, vec![section])
        .unwrap_or_else(|_| unreachable!());
    FormDefinition::new(
        entity_logical_name,
        form_logical_name,
        "Main Form",
        FormType::Main,
        vec![tab],
        vec![],
    )
    .unwrap_or_else(|_| unreachable!())
}

fn minimal_view(entity_logical_name: &str, view_logical_name: &str) -> ViewDefinition {
    let column = ViewColumn::new("name", 0, None, None).unwrap_or_else(|_| unreachable!());
    ViewDefinition::new(
        entity_logical_name,
        view_logical_name,
        "Main View",
        ViewType::Grid,
        vec![column],
        None,
        None,
        true,
    )
    .unwrap_or_else(|_| unreachable!())
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
            AppEntityBinding::new(
                "sales",
                "account",
                None,
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
            AppEntityBinding::new(
                "sales",
                "invoice",
                None,
                1,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
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
    let navigation = navigation.unwrap_or_else(|_| unreachable!());
    assert_eq!(navigation.areas().len(), 1);
    assert_eq!(navigation.areas()[0].groups().len(), 1);
    assert_eq!(navigation.areas()[0].groups()[0].sub_areas().len(), 1);
}

#[tokio::test]
async fn app_navigation_orders_bindings_by_navigation_order_then_entity_name() {
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
            AppEntityBinding::new(
                "sales",
                "invoice",
                Some("Invoices".to_owned()),
                2,
                vec![
                    AppEntityForm::new("invoice_form", "Invoice Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("invoice_view", "Invoice View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "invoice_form",
                "invoice_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
            AppEntityBinding::new(
                "sales",
                "contact",
                Some("Contacts".to_owned()),
                1,
                vec![
                    AppEntityForm::new("contact_form", "Contact Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("contact_view", "Contact View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "contact_form",
                "contact_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
            AppEntityBinding::new(
                "sales",
                "account",
                Some("Accounts".to_owned()),
                1,
                vec![
                    AppEntityForm::new("account_form", "Account Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("account_view", "Account View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "account_form",
                "account_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    app_repository.subject_permissions.lock().await.insert(
        (tenant_id, "worker".to_owned(), "sales".to_owned()),
        vec![
            SubjectEntityPermission {
                entity_logical_name: "contact".to_owned(),
                can_read: true,
                can_create: false,
                can_update: false,
                can_delete: false,
            },
            SubjectEntityPermission {
                entity_logical_name: "account".to_owned(),
                can_read: true,
                can_create: false,
                can_update: false,
                can_delete: false,
            },
            SubjectEntityPermission {
                entity_logical_name: "invoice".to_owned(),
                can_read: true,
                can_create: false,
                can_update: false,
                can_delete: false,
            },
        ],
    );

    let navigation = service.app_navigation_for_subject(&actor, "sales").await;

    assert!(navigation.is_ok());
    let navigation = navigation.unwrap_or_else(|_| unreachable!());
    let sub_areas = navigation.areas()[0].groups()[0].sub_areas();
    assert_eq!(sub_areas.len(), 3);

    let entity_names = sub_areas
        .iter()
        .map(|sub_area| match sub_area.target() {
            SitemapTarget::Entity {
                entity_logical_name,
                ..
            } => entity_logical_name.to_owned(),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        entity_names,
        vec![
            "account".to_owned(),
            "contact".to_owned(),
            "invoice".to_owned()
        ]
    );
}

#[tokio::test]
async fn bind_entity_defaults_to_first_surface_when_defaults_omitted() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository,
        runtime_record_service,
    );

    let binding = service
        .bind_entity(
            &actor,
            BindAppEntityInput {
                app_logical_name: "sales".to_owned(),
                entity_logical_name: "contact".to_owned(),
                navigation_label: Some("Contacts".to_owned()),
                navigation_order: 0,
                forms: Some(vec![
                    AppEntityFormInput {
                        logical_name: "quick_form".to_owned(),
                        display_name: "Quick Form".to_owned(),
                        field_logical_names: Vec::new(),
                    },
                    AppEntityFormInput {
                        logical_name: "main_custom".to_owned(),
                        display_name: "Main Custom".to_owned(),
                        field_logical_names: Vec::new(),
                    },
                ]),
                list_views: Some(vec![
                    AppEntityViewInput {
                        logical_name: "compact_view".to_owned(),
                        display_name: "Compact View".to_owned(),
                        field_logical_names: Vec::new(),
                    },
                    AppEntityViewInput {
                        logical_name: "grid_view".to_owned(),
                        display_name: "Grid View".to_owned(),
                        field_logical_names: Vec::new(),
                    },
                ]),
                default_form_logical_name: None,
                default_list_view_logical_name: None,
                form_field_logical_names: None,
                list_field_logical_names: None,
                default_view_mode: None,
            },
        )
        .await;

    assert!(binding.is_ok());
    let binding = binding.unwrap_or_else(|_| unreachable!());
    assert_eq!(binding.default_form_logical_name().as_str(), "quick_form");
    assert_eq!(
        binding.default_list_view_logical_name().as_str(),
        "compact_view"
    );
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

#[tokio::test]
async fn app_publish_checks_report_unpublished_entity_bindings() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service,
    );

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new(
                "sales",
                "contact",
                None,
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    let result = service.publish_checks(&actor, "sales").await;

    assert!(result.is_ok());
    let errors = result.unwrap_or_default();
    assert!(!errors.is_empty());
    assert!(
        errors
            .iter()
            .any(|error| error.contains("requires a published schema"))
    );
}

#[tokio::test]
async fn app_publish_checks_allow_selected_unpublished_dependencies() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new(
                "sales",
                "contact",
                None,
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    runtime_record_service.forms.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_form("contact", "main_form")],
    );
    runtime_record_service.views.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_view("contact", "main_view")],
    );

    let result = service
        .publish_checks_with_allowed_unpublished_entities(&actor, "sales", &["contact".to_owned()])
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap_or_default().is_empty());
}

#[tokio::test]
async fn save_sitemap_rejects_unbound_entity_target() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository,
        runtime_record_service,
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "entities",
                        "Entities",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "contacts",
                                "Contacts",
                                0,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let result = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap,
            },
        )
        .await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn save_sitemap_rejects_missing_default_form_or_view_reference() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new(
                "sales",
                "contact",
                None,
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    runtime_record_service.forms.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_form("contact", "main_form")],
    );
    runtime_record_service.views.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_view("contact", "main_view")],
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "entities",
                        "Entities",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "contacts",
                                "Contacts",
                                0,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: Some("missing_form".to_owned()),
                                    default_view: Some("main_view".to_owned()),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let result = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap,
            },
        )
        .await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn save_sitemap_rejects_duplicate_sub_area_positions() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service,
    );

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new(
                "sales",
                "contact",
                None,
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "entities",
                        "Entities",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "contacts",
                                "Contacts",
                                0,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "contacts_secondary",
                                "Contacts Secondary",
                                0,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let result = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap,
            },
        )
        .await;

    match result {
        Err(AppError::Validation(message)) => {
            assert!(message.contains("duplicate sitemap sub area position"));
        }
        _ => panic!("expected sitemap structure validation failure"),
    }
}

#[tokio::test]
async fn app_publish_checks_report_sitemap_structure_issues() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new(
                "sales",
                "contact",
                None,
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    runtime_record_service.forms.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_form("contact", "main_form")],
    );
    runtime_record_service.views.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_view("contact", "main_view")],
    );

    let invalid_sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "entities",
                        "Entities",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "contacts",
                                "Contacts",
                                0,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                    SitemapGroup::new(
                        "entities",
                        "Entities Duplicate",
                        1,
                        vec![
                            SitemapSubArea::new(
                                "contacts_second",
                                "Contacts Second",
                                0,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    app_repository
        .sitemaps
        .lock()
        .await
        .insert((tenant_id, "sales".to_owned()), invalid_sitemap);

    let result = service.publish_checks(&actor, "sales").await;

    assert!(result.is_ok());
    let errors = result.unwrap_or_default();
    assert!(
        errors
            .iter()
            .any(|error| error.contains("duplicate sitemap group logical name"))
    );
}

#[tokio::test]
async fn save_sitemap_rejects_negative_area_position() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository,
        runtime_record_service,
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                -1,
                None,
                vec![
                    SitemapGroup::new(
                        "general",
                        "General",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "help",
                                "Help",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/help".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let result = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap,
            },
        )
        .await;

    match result {
        Err(AppError::Validation(message)) => {
            assert!(message.contains("sitemap area 'core' has negative position"));
        }
        _ => panic!("expected sitemap structure validation failure"),
    }
}

#[tokio::test]
async fn save_sitemap_rejects_duplicate_area_logical_names() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository,
        runtime_record_service,
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "general",
                        "General",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "help",
                                "Help",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/help".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
            SitemapArea::new(
                "core",
                "Core Duplicate",
                1,
                None,
                vec![
                    SitemapGroup::new(
                        "general_2",
                        "General 2",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "faq",
                                "FAQ",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/faq".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let result = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap,
            },
        )
        .await;

    match result {
        Err(AppError::Validation(message)) => {
            assert!(message.contains("duplicate sitemap area logical name"));
        }
        _ => panic!("expected sitemap structure validation failure"),
    }
}

#[tokio::test]
async fn app_publish_checks_report_negative_sub_area_position() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service.clone(),
    );

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new(
                "sales",
                "contact",
                None,
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    runtime_record_service.forms.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_form("contact", "main_form")],
    );
    runtime_record_service.views.lock().await.insert(
        (tenant_id, "contact".to_owned()),
        vec![minimal_view("contact", "main_view")],
    );

    let invalid_sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "entities",
                        "Entities",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "contacts",
                                "Contacts",
                                -1,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    app_repository
        .sitemaps
        .lock()
        .await
        .insert((tenant_id, "sales".to_owned()), invalid_sitemap);

    let result = service.publish_checks(&actor, "sales").await;

    assert!(result.is_ok());
    let errors = result.unwrap_or_default();
    assert!(errors.iter().any(|error| {
        error.contains("sitemap sub area 'core.entities.contacts' has negative position")
    }));
}

#[tokio::test]
async fn app_navigation_uses_saved_sitemap_position_order_over_binding_order() {
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
            AppEntityBinding::new(
                "sales",
                "account",
                Some("Accounts".to_owned()),
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
            AppEntityBinding::new(
                "sales",
                "contact",
                Some("Contacts".to_owned()),
                1,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
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
                entity_logical_name: "contact".to_owned(),
                can_read: true,
                can_create: false,
                can_update: false,
                can_delete: false,
            },
        ],
    );

    let saved_sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "entities",
                        "Entities",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "accounts",
                                "Accounts",
                                1,
                                SitemapTarget::Entity {
                                    entity_logical_name: "account".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "contacts",
                                "Contacts",
                                0,
                                SitemapTarget::Entity {
                                    entity_logical_name: "contact".to_owned(),
                                    default_form: None,
                                    default_view: None,
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());
    app_repository
        .sitemaps
        .lock()
        .await
        .insert((tenant_id, "sales".to_owned()), saved_sitemap);

    let navigation = service.app_navigation_for_subject(&actor, "sales").await;

    assert!(navigation.is_ok());
    let navigation = navigation.unwrap_or_else(|_| unreachable!());
    let sub_areas = navigation.areas()[0].groups()[0].sub_areas();
    assert_eq!(sub_areas.len(), 2);
    assert_eq!(sub_areas[0].logical_name().as_str(), "contacts");
    assert_eq!(sub_areas[1].logical_name().as_str(), "accounts");
}

#[tokio::test]
async fn save_sitemap_rejects_sparse_sub_area_positions() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository,
        runtime_record_service,
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "general",
                        "General",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "help",
                                "Help",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/help".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "faq",
                                "FAQ",
                                2,
                                SitemapTarget::CustomPage {
                                    url: "/faq".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let result = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap,
            },
        )
        .await;

    match result {
        Err(AppError::Validation(message)) => {
            assert!(
                message.contains(
                    "sitemap sub area positions in group 'core.general' must form contiguous sequence starting at zero"
                )
            );
        }
        _ => panic!("expected sitemap structure validation failure"),
    }
}

#[tokio::test]
async fn app_publish_checks_report_sparse_group_positions() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service,
    );

    let invalid_sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "general",
                        "General",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "help",
                                "Help",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/help".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                    SitemapGroup::new(
                        "references",
                        "References",
                        2,
                        vec![
                            SitemapSubArea::new(
                                "faq",
                                "FAQ",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/faq".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    app_repository
        .sitemaps
        .lock()
        .await
        .insert((tenant_id, "sales".to_owned()), invalid_sitemap);

    let result = service.publish_checks(&actor, "sales").await;

    assert!(result.is_ok());
    let errors = result.unwrap_or_default();
    assert!(errors.iter().any(|error| {
        error.contains(
            "sitemap group positions in area 'core' must form contiguous sequence starting at zero",
        )
    }));
}

#[tokio::test]
async fn get_sitemap_returns_saved_nodes_sorted_by_position() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service,
    );

    let saved_sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "later",
                "Later",
                1,
                None,
                vec![
                    SitemapGroup::new(
                        "secondary",
                        "Secondary",
                        1,
                        vec![
                            SitemapSubArea::new(
                                "docs",
                                "Docs",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/docs".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "about",
                                "About",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/about".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                    SitemapGroup::new(
                        "primary",
                        "Primary",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "home",
                                "Home",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "main",
                        "Main",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "welcome",
                                "Welcome",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/welcome".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    app_repository
        .sitemaps
        .lock()
        .await
        .insert((tenant_id, "sales".to_owned()), saved_sitemap);

    let sitemap = service.get_sitemap(&actor, "sales").await;

    assert!(sitemap.is_ok());
    let sitemap = sitemap.unwrap_or_else(|_| unreachable!());
    assert_eq!(sitemap.areas()[0].logical_name().as_str(), "core");
    assert_eq!(sitemap.areas()[1].logical_name().as_str(), "later");

    let groups = sitemap.areas()[1].groups();
    assert_eq!(groups[0].logical_name().as_str(), "primary");
    assert_eq!(groups[1].logical_name().as_str(), "secondary");

    let sub_areas = groups[1].sub_areas();
    assert_eq!(sub_areas[0].logical_name().as_str(), "about");
    assert_eq!(sub_areas[1].logical_name().as_str(), "docs");
}

#[tokio::test]
async fn save_sitemap_normalizes_and_persists_sorted_positions() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service,
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "later",
                "Later",
                1,
                None,
                vec![
                    SitemapGroup::new(
                        "secondary",
                        "Secondary",
                        1,
                        vec![
                            SitemapSubArea::new(
                                "docs",
                                "Docs",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/docs".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "about",
                                "About",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/about".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                    SitemapGroup::new(
                        "primary",
                        "Primary",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "home",
                                "Home",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "main",
                        "Main",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "welcome",
                                "Welcome",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/welcome".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let saved = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap,
            },
        )
        .await;

    assert!(saved.is_ok());
    let saved = saved.unwrap_or_else(|_| unreachable!());
    assert_eq!(saved.areas()[0].logical_name().as_str(), "core");
    assert_eq!(saved.areas()[1].logical_name().as_str(), "later");

    let persisted = app_repository
        .sitemaps
        .lock()
        .await
        .get(&(tenant_id, "sales".to_owned()))
        .cloned();
    assert!(persisted.is_some());
    let persisted = persisted.unwrap_or_else(|| unreachable!());
    assert_eq!(persisted.areas()[0].logical_name().as_str(), "core");
    assert_eq!(persisted.areas()[1].logical_name().as_str(), "later");
    let persisted_groups = persisted.areas()[1].groups();
    assert_eq!(persisted_groups[0].logical_name().as_str(), "primary");
    assert_eq!(persisted_groups[1].logical_name().as_str(), "secondary");
    let persisted_sub_areas = persisted_groups[1].sub_areas();
    assert_eq!(persisted_sub_areas[0].logical_name().as_str(), "about");
    assert_eq!(persisted_sub_areas[1].logical_name().as_str(), "docs");
}

#[tokio::test]
async fn save_sitemap_supports_reorder_then_undo_redo_across_saves() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "admin");
    let app_repository = Arc::new(FakeAppRepository::default());
    let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
    let service = build_service(
        HashMap::from([(
            (tenant_id, "admin".to_owned()),
            vec![Permission::SecurityRoleManage],
        )]),
        app_repository.clone(),
        runtime_record_service,
    );

    let initial = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "work",
                        "Work",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "home",
                                "Home",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/home".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "tasks",
                                "Tasks",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/tasks".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                    SitemapGroup::new(
                        "admin",
                        "Admin",
                        1,
                        vec![
                            SitemapSubArea::new(
                                "settings",
                                "Settings",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/settings".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
            SitemapArea::new(
                "analytics",
                "Analytics",
                1,
                None,
                vec![
                    SitemapGroup::new(
                        "dashboards",
                        "Dashboards",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "overview",
                                "Overview",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/overview".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "pipeline",
                                "Pipeline",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/pipeline".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let saved_initial = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap: initial,
            },
        )
        .await;
    assert!(saved_initial.is_ok());

    let reordered = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "analytics",
                "Analytics",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "dashboards",
                        "Dashboards",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "pipeline",
                                "Pipeline",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/pipeline".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "overview",
                                "Overview",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/overview".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
            SitemapArea::new(
                "core",
                "Core",
                1,
                None,
                vec![
                    SitemapGroup::new(
                        "admin",
                        "Admin",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "settings",
                                "Settings",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/settings".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                    SitemapGroup::new(
                        "work",
                        "Work",
                        1,
                        vec![
                            SitemapSubArea::new(
                                "tasks",
                                "Tasks",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/tasks".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "home",
                                "Home",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/home".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let saved_reordered = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap: reordered,
            },
        )
        .await;
    assert!(saved_reordered.is_ok());
    let saved_reordered = saved_reordered.unwrap_or_else(|_| unreachable!());
    assert_eq!(
        saved_reordered.areas()[0].logical_name().as_str(),
        "analytics"
    );
    assert_eq!(saved_reordered.areas()[1].logical_name().as_str(), "core");
    let core_groups = saved_reordered.areas()[1].groups();
    assert_eq!(core_groups[0].logical_name().as_str(), "admin");
    assert_eq!(core_groups[1].logical_name().as_str(), "work");
    let work_sub_areas = core_groups[1].sub_areas();
    assert_eq!(work_sub_areas[0].logical_name().as_str(), "tasks");
    assert_eq!(work_sub_areas[1].logical_name().as_str(), "home");

    let undone = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "core",
                "Core",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "work",
                        "Work",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "home",
                                "Home",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/home".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "tasks",
                                "Tasks",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/tasks".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                    SitemapGroup::new(
                        "admin",
                        "Admin",
                        1,
                        vec![
                            SitemapSubArea::new(
                                "settings",
                                "Settings",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/settings".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
            SitemapArea::new(
                "analytics",
                "Analytics",
                1,
                None,
                vec![
                    SitemapGroup::new(
                        "dashboards",
                        "Dashboards",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "overview",
                                "Overview",
                                0,
                                SitemapTarget::CustomPage {
                                    url: "/overview".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                            SitemapSubArea::new(
                                "pipeline",
                                "Pipeline",
                                1,
                                SitemapTarget::CustomPage {
                                    url: "/pipeline".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());

    let saved_undone = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap: undone,
            },
        )
        .await;
    assert!(saved_undone.is_ok());

    let redone = service
        .save_sitemap(
            &actor,
            SaveAppSitemapInput {
                app_logical_name: "sales".to_owned(),
                sitemap: saved_reordered,
            },
        )
        .await;
    assert!(redone.is_ok());
    let redone = redone.unwrap_or_else(|_| unreachable!());

    assert_eq!(redone.areas()[0].logical_name().as_str(), "analytics");
    assert_eq!(redone.areas()[1].logical_name().as_str(), "core");
    let core_groups = redone.areas()[1].groups();
    assert_eq!(core_groups[0].logical_name().as_str(), "admin");
    assert_eq!(core_groups[1].logical_name().as_str(), "work");
    let work_sub_areas = core_groups[1].sub_areas();
    assert_eq!(work_sub_areas[0].logical_name().as_str(), "tasks");
    assert_eq!(work_sub_areas[1].logical_name().as_str(), "home");

    let persisted = app_repository
        .sitemaps
        .lock()
        .await
        .get(&(tenant_id, "sales".to_owned()))
        .cloned();
    assert!(persisted.is_some());
    let persisted = persisted.unwrap_or_else(|| unreachable!());
    assert_eq!(persisted.areas()[0].logical_name().as_str(), "analytics");
    assert_eq!(persisted.areas()[1].logical_name().as_str(), "core");
}

#[tokio::test]
async fn get_dashboard_for_subject_returns_metadata_from_sitemap_target() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "worker");
    let app_repository = Arc::new(FakeAppRepository::default());
    let service = build_service(
        HashMap::new(),
        app_repository.clone(),
        Arc::new(FakeRuntimeRecordService::default()),
    );

    app_repository
        .subject_access
        .lock()
        .await
        .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);

    app_repository.bindings.lock().await.insert(
        (tenant_id, "sales".to_owned()),
        vec![
            AppEntityBinding::new(
                "sales",
                "account",
                Some("Accounts".to_owned()),
                0,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
            AppEntityBinding::new(
                "sales",
                "invoice",
                Some("Invoices".to_owned()),
                1,
                vec![
                    AppEntityForm::new("main_form", "Main Form", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                vec![
                    AppEntityView::new("main_view", "Main View", Vec::new())
                        .unwrap_or_else(|_| unreachable!()),
                ],
                "main_form",
                "main_view",
                AppEntityViewMode::Grid,
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    );

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "analytics",
                "Analytics",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "dashboards",
                        "Dashboards",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "sales_overview",
                                "Sales Overview",
                                0,
                                SitemapTarget::Dashboard {
                                    dashboard_logical_name: "sales_overview".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());
    app_repository
        .sitemaps
        .lock()
        .await
        .insert((tenant_id, "sales".to_owned()), sitemap);

    let dashboard = service
        .get_dashboard_for_subject(&actor, "sales", "sales_overview")
        .await;
    assert!(dashboard.is_ok());
    let dashboard = dashboard.unwrap_or_else(|_| unreachable!());
    assert_eq!(dashboard.logical_name().as_str(), "sales_overview");
    assert_eq!(dashboard.display_name().as_str(), "Sales Overview");
    assert_eq!(dashboard.widgets().len(), 2);
    assert_eq!(
        dashboard.widgets()[0]
            .chart()
            .entity_logical_name()
            .as_str(),
        "account"
    );
    assert_eq!(
        dashboard.widgets()[1]
            .chart()
            .entity_logical_name()
            .as_str(),
        "invoice"
    );
}

#[tokio::test]
async fn get_dashboard_for_subject_rejects_unknown_dashboard_logical_name() {
    let tenant_id = TenantId::new();
    let actor = actor(tenant_id, "worker");
    let app_repository = Arc::new(FakeAppRepository::default());
    let service = build_service(
        HashMap::new(),
        app_repository.clone(),
        Arc::new(FakeRuntimeRecordService::default()),
    );

    app_repository
        .subject_access
        .lock()
        .await
        .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);

    let sitemap = AppSitemap::new(
        "sales",
        vec![
            SitemapArea::new(
                "analytics",
                "Analytics",
                0,
                None,
                vec![
                    SitemapGroup::new(
                        "dashboards",
                        "Dashboards",
                        0,
                        vec![
                            SitemapSubArea::new(
                                "sales_overview",
                                "Sales Overview",
                                0,
                                SitemapTarget::Dashboard {
                                    dashboard_logical_name: "sales_overview".to_owned(),
                                },
                                None,
                            )
                            .unwrap_or_else(|_| unreachable!()),
                        ],
                    )
                    .unwrap_or_else(|_| unreachable!()),
                ],
            )
            .unwrap_or_else(|_| unreachable!()),
        ],
    )
    .unwrap_or_else(|_| unreachable!());
    app_repository
        .sitemaps
        .lock()
        .await
        .insert((tenant_id, "sales".to_owned()), sitemap);

    let result = service
        .get_dashboard_for_subject(&actor, "sales", "missing")
        .await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}
