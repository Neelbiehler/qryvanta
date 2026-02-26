use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::Json;
use axum::extract::{Extension, Query, State};
use qryvanta_application::{
    AppEntityFormInput, AppEntityViewInput, AppRepository, AppService, AuditEvent, AuditLogEntry,
    AuditLogQuery, AuditLogRepository, AuditRepository, AuthorizationRepository,
    AuthorizationService, BindAppEntityInput, CreateAppInput, MetadataService, RuntimeFieldGrant,
    RuntimeRecordService, SaveFieldInput, SaveFormInput, SaveViewInput, SecurityAdminService,
    SubjectEntityPermission, TemporaryPermissionGrant, WorkspacePublishRunAuditInput,
};
use qryvanta_core::{AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityRolePermission, AppSitemap, FieldType, FormDefinition,
    FormFieldPlacement, FormSection, FormTab, FormType, Permission, ViewColumn, ViewDefinition,
    ViewType,
};
use qryvanta_infrastructure::{InMemoryMetadataRepository, PostgresSecurityAdminRepository};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::Mutex;

use super::diff::{compute_form_surface_delta, compute_view_surface_delta};
use super::history::map_workspace_publish_history_entries;
use super::issues::{
    build_unknown_selection_issues, extract_dependency_path, partition_known_names,
    resolve_requested_names,
};
use super::{
    PublishCheckCategoryDto, PublishCheckScopeDto, PublishHistoryQuery, PublishState,
    run_workspace_publish_handler, workspace_publish_diff_handler,
    workspace_publish_history_handler,
};
use crate::dto::{RunWorkspacePublishRequest, WorkspacePublishDiffRequest};

#[derive(Default)]
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
struct SharedAuditSink {
    events: Mutex<Vec<AuditEvent>>,
}

#[derive(Clone)]
struct FakeAuditRepository {
    sink: Arc<SharedAuditSink>,
}

#[async_trait]
impl AuditRepository for FakeAuditRepository {
    async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
        self.sink.events.lock().await.push(event);
        Ok(())
    }
}

#[derive(Clone)]
struct FakeAuditLogRepository {
    sink: Arc<SharedAuditSink>,
}

#[async_trait]
impl AuditLogRepository for FakeAuditLogRepository {
    async fn list_recent_entries(
        &self,
        _tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        let mut entries = self
            .sink
            .events
            .lock()
            .await
            .iter()
            .enumerate()
            .filter(|(_, event)| {
                query
                    .action
                    .as_ref()
                    .map(|action| event.action.as_str() == action)
                    .unwrap_or(true)
                    && query
                        .subject
                        .as_ref()
                        .map(|subject| &event.subject == subject)
                        .unwrap_or(true)
            })
            .map(|(index, event)| AuditLogEntry {
                event_id: format!("run-{index}"),
                subject: event.subject.clone(),
                action: event.action.as_str().to_owned(),
                resource_type: event.resource_type.clone(),
                resource_id: event.resource_id.clone(),
                detail: event.detail.clone(),
                created_at: format!("2026-02-24T00:00:{index:02}Z"),
            })
            .collect::<Vec<_>>();

        entries.reverse();
        let offset = query.offset.min(entries.len());
        let limit = query.limit.min(entries.len().saturating_sub(offset));
        Ok(entries.into_iter().skip(offset).take(limit).collect())
    }

    async fn export_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        self.list_recent_entries(tenant_id, query).await
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
struct FakeAppRepository {
    apps: Mutex<HashMap<TenantId, Vec<AppDefinition>>>,
    bindings: Mutex<HashMap<(TenantId, String), Vec<qryvanta_domain::AppEntityBinding>>>,
}

#[async_trait]
impl AppRepository for FakeAppRepository {
    async fn create_app(&self, tenant_id: TenantId, app: AppDefinition) -> AppResult<()> {
        let mut apps = self.apps.lock().await;
        let tenant_apps = apps.entry(tenant_id).or_default();
        tenant_apps.retain(|existing| existing.logical_name() != app.logical_name());
        tenant_apps.push(app);
        Ok(())
    }

    async fn list_apps(&self, tenant_id: TenantId) -> AppResult<Vec<AppDefinition>> {
        Ok(self
            .apps
            .lock()
            .await
            .get(&tenant_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_app(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppDefinition>> {
        Ok(self.apps.lock().await.get(&tenant_id).and_then(|apps| {
            apps.iter()
                .find(|app| app.logical_name().as_str() == app_logical_name)
                .cloned()
        }))
    }

    async fn save_app_entity_binding(
        &self,
        tenant_id: TenantId,
        binding: qryvanta_domain::AppEntityBinding,
    ) -> AppResult<()> {
        let mut bindings = self.bindings.lock().await;
        let key = (tenant_id, binding.app_logical_name().as_str().to_owned());
        let tenant_bindings = bindings.entry(key).or_default();
        tenant_bindings
            .retain(|existing| existing.entity_logical_name() != binding.entity_logical_name());
        tenant_bindings.push(binding);
        Ok(())
    }

    async fn list_app_entity_bindings(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<qryvanta_domain::AppEntityBinding>> {
        Ok(self
            .bindings
            .lock()
            .await
            .get(&(tenant_id, app_logical_name.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn save_sitemap(&self, _tenant_id: TenantId, _sitemap: AppSitemap) -> AppResult<()> {
        Ok(())
    }

    async fn get_sitemap(
        &self,
        _tenant_id: TenantId,
        _app_logical_name: &str,
    ) -> AppResult<Option<AppSitemap>> {
        Ok(None)
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
        tenant_id: TenantId,
        _subject: &str,
    ) -> AppResult<Vec<AppDefinition>> {
        self.list_apps(tenant_id).await
    }

    async fn subject_can_access_app(
        &self,
        tenant_id: TenantId,
        _subject: &str,
        app_logical_name: &str,
    ) -> AppResult<bool> {
        Ok(self
            .apps
            .lock()
            .await
            .get(&tenant_id)
            .map(|apps| {
                apps.iter()
                    .any(|app| app.logical_name().as_str() == app_logical_name)
            })
            .unwrap_or(false))
    }

    async fn subject_entity_permission(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _app_logical_name: &str,
        _entity_logical_name: &str,
    ) -> AppResult<Option<SubjectEntityPermission>> {
        Ok(None)
    }

    async fn list_subject_entity_permissions(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _app_logical_name: &str,
    ) -> AppResult<Vec<SubjectEntityPermission>> {
        Ok(Vec::new())
    }
}

async fn build_publish_state() -> (PublishState, UserIdentity) {
    let tenant_id = TenantId::new();
    let actor = UserIdentity::new("maker", "maker", None, tenant_id);
    let audit_sink = Arc::new(SharedAuditSink::default());
    let audit_repository: Arc<dyn AuditRepository> = Arc::new(FakeAuditRepository {
        sink: audit_sink.clone(),
    });

    let authorization_service = AuthorizationService::new(
        Arc::new(FakeAuthorizationRepository {
            grants: HashMap::from([(
                (tenant_id, "maker".to_owned()),
                vec![
                    Permission::SecurityRoleManage,
                    Permission::SecurityAuditRead,
                    Permission::MetadataEntityRead,
                    Permission::MetadataEntityCreate,
                    Permission::MetadataFieldRead,
                    Permission::MetadataFieldWrite,
                ],
            )]),
        }),
        audit_repository.clone(),
    );

    let metadata_service = MetadataService::new(
        Arc::new(InMemoryMetadataRepository::new()),
        authorization_service.clone(),
        audit_repository.clone(),
    );

    let app_service = AppService::new(
        authorization_service.clone(),
        Arc::new(FakeAppRepository::default()),
        Arc::new(metadata_service.clone()) as Arc<dyn RuntimeRecordService>,
        audit_repository.clone(),
    );

    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@localhost:5432/qryvanta")
        .unwrap_or_else(|_| unreachable!());
    let security_admin_service = SecurityAdminService::new(
        authorization_service,
        Arc::new(PostgresSecurityAdminRepository::new(pool.clone())),
        Arc::new(FakeAuditLogRepository {
            sink: audit_sink.clone(),
        }),
        audit_repository,
    );

    assert!(
        metadata_service
            .register_entity(&actor, "contact", "Contact")
            .await
            .is_ok()
    );
    assert!(
        app_service
            .create_app(
                &actor,
                CreateAppInput {
                    logical_name: "sales".to_owned(),
                    display_name: "Sales".to_owned(),
                    description: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(
        app_service
            .bind_entity(
                &actor,
                BindAppEntityInput {
                    app_logical_name: "sales".to_owned(),
                    entity_logical_name: "contact".to_owned(),
                    navigation_label: Some("Contacts".to_owned()),
                    navigation_order: 0,
                    forms: Some(vec![AppEntityFormInput {
                        logical_name: "main_form".to_owned(),
                        display_name: "Main Form".to_owned(),
                        field_logical_names: Vec::new(),
                    }]),
                    list_views: Some(vec![AppEntityViewInput {
                        logical_name: "main_view".to_owned(),
                        display_name: "Main View".to_owned(),
                        field_logical_names: Vec::new(),
                    }]),
                    default_form_logical_name: Some("main_form".to_owned()),
                    default_list_view_logical_name: Some("main_view".to_owned()),
                    form_field_logical_names: None,
                    list_field_logical_names: None,
                    default_view_mode: None,
                },
            )
            .await
            .is_ok()
    );

    (
        PublishState {
            app_service,
            metadata_service,
            security_admin_service,
        },
        actor,
    )
}

#[test]
fn resolve_requested_names_deduplicates_preserving_order() {
    let result = resolve_requested_names(
        vec![
            "contact".to_owned(),
            "account".to_owned(),
            "contact".to_owned(),
        ],
        vec!["fallback".to_owned()],
    );

    assert_eq!(result, vec!["contact".to_owned(), "account".to_owned()]);
}

#[test]
fn resolve_requested_names_uses_fallback_when_empty() {
    let result =
        resolve_requested_names(Vec::new(), vec!["contact".to_owned(), "account".to_owned()]);

    assert_eq!(result, vec!["contact".to_owned(), "account".to_owned()]);
}

#[test]
fn partition_known_names_splits_by_available_set() {
    let (known, unknown) = partition_known_names(
        &[
            "contact".to_owned(),
            "missing".to_owned(),
            "account".to_owned(),
        ],
        &["contact".to_owned(), "account".to_owned()],
    );

    assert_eq!(known, vec!["contact".to_owned(), "account".to_owned()]);
    assert_eq!(unknown, vec!["missing".to_owned()]);
}

#[test]
fn build_unknown_selection_issues_maps_scope_specific_messages() {
    let issues = build_unknown_selection_issues(
        PublishCheckScopeDto::Entity,
        &["missing_entity".to_owned()],
    );

    assert_eq!(issues.len(), 1);
    assert!(matches!(issues[0].scope, PublishCheckScopeDto::Entity));
    assert!(issues[0].message.contains("does not exist"));
    assert_eq!(issues[0].fix_path.as_deref(), Some("/maker/entities"));
    assert!(issues[0].dependency_path.is_none());
}

#[test]
fn extract_dependency_path_reads_app_entity_edge() {
    let edge = extract_dependency_path(
        "dependency check failed: app 'sales' -> entity 'contact' requires a published schema or inclusion in this publish selection",
    );

    assert_eq!(edge.as_deref(), Some("sales -> contact"));
}

#[test]
fn extract_dependency_path_reads_entity_relation_edge() {
    let edge = extract_dependency_path(
        "dependency check failed: entity 'contact' relation field 'account_id' -> entity 'account' requires a published schema or inclusion in this publish selection",
    );

    assert_eq!(edge.as_deref(), Some("contact.account_id -> account"));
}

#[test]
fn map_workspace_publish_history_entries_skips_invalid_payloads_and_preserves_order() {
    let valid_detail = serde_json::json!({
        "requested_entities": 2,
        "requested_apps": 1,
        "published_entities": ["contact"],
        "validated_apps": ["sales"],
        "issue_count": 0,
        "is_publishable": true,
    })
    .to_string();

    let history = map_workspace_publish_history_entries(vec![
        AuditLogEntry {
            event_id: "run-2".to_owned(),
            subject: "maker-b".to_owned(),
            action: "metadata.workspace.published".to_owned(),
            resource_type: "workspace_publish_run".to_owned(),
            resource_id: "maker-b-2".to_owned(),
            detail: Some(valid_detail.clone()),
            created_at: "2026-02-24T15:00:00Z".to_owned(),
        },
        AuditLogEntry {
            event_id: "run-invalid".to_owned(),
            subject: "maker-x".to_owned(),
            action: "metadata.workspace.published".to_owned(),
            resource_type: "workspace_publish_run".to_owned(),
            resource_id: "maker-x-9".to_owned(),
            detail: Some("not-json".to_owned()),
            created_at: "2026-02-24T14:00:00Z".to_owned(),
        },
        AuditLogEntry {
            event_id: "run-1".to_owned(),
            subject: "maker-a".to_owned(),
            action: "metadata.workspace.published".to_owned(),
            resource_type: "workspace_publish_run".to_owned(),
            resource_id: "maker-a-1".to_owned(),
            detail: Some(valid_detail),
            created_at: "2026-02-24T13:00:00Z".to_owned(),
        },
    ]);

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].run_id, "run-2");
    assert_eq!(history[0].run_at, "2026-02-24T15:00:00Z");
    assert_eq!(history[1].run_id, "run-1");
    assert_eq!(history[1].run_at, "2026-02-24T13:00:00Z");
}

fn test_form(
    logical_name: &str,
    display_name: &str,
    form_type: FormType,
    field_names: &[&str],
) -> FormDefinition {
    let fields = field_names
        .iter()
        .enumerate()
        .map(|(index, field_name)| {
            FormFieldPlacement::new(*field_name, 0, index as i32, true, false, None, None)
                .unwrap_or_else(|_| unreachable!())
        })
        .collect::<Vec<_>>();
    let section = FormSection::new("general", "General", 0, true, 1, fields, Vec::new())
        .unwrap_or_else(|_| unreachable!());
    let tab =
        FormTab::new("main", "Main", 0, true, vec![section]).unwrap_or_else(|_| unreachable!());

    FormDefinition::new(
        "contact",
        logical_name,
        display_name,
        form_type,
        vec![tab],
        Vec::new(),
    )
    .unwrap_or_else(|_| unreachable!())
}

fn test_view(
    logical_name: &str,
    display_name: &str,
    is_default: bool,
    columns: &[&str],
) -> ViewDefinition {
    let resolved_columns = columns
        .iter()
        .enumerate()
        .map(|(index, field_name)| {
            ViewColumn::new(*field_name, index as i32, None, None)
                .unwrap_or_else(|_| unreachable!())
        })
        .collect::<Vec<_>>();

    ViewDefinition::new(
        "contact",
        logical_name,
        display_name,
        ViewType::Grid,
        resolved_columns,
        None,
        None,
        is_default,
    )
    .unwrap_or_else(|_| unreachable!())
}

async fn save_text_field(
    state: &PublishState,
    actor: &UserIdentity,
    logical_name: &str,
    display_name: &str,
) {
    let saved = state
        .metadata_service
        .save_field(
            actor,
            SaveFieldInput {
                entity_logical_name: "contact".to_owned(),
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                field_type: FieldType::Text,
                is_required: false,
                is_unique: false,
                default_value: None,
                relation_target_entity: None,
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await;
    assert!(saved.is_ok());
}

async fn save_form_definition(state: &PublishState, actor: &UserIdentity, form: FormDefinition) {
    let saved = state
        .metadata_service
        .save_form(
            actor,
            SaveFormInput {
                entity_logical_name: form.entity_logical_name().as_str().to_owned(),
                logical_name: form.logical_name().as_str().to_owned(),
                display_name: form.display_name().as_str().to_owned(),
                form_type: form.form_type(),
                tabs: form.tabs().to_vec(),
                header_fields: form.header_fields().to_vec(),
            },
        )
        .await;
    assert!(saved.is_ok());
}

async fn save_view_definition(state: &PublishState, actor: &UserIdentity, view: ViewDefinition) {
    let saved = state
        .metadata_service
        .save_view(
            actor,
            SaveViewInput {
                entity_logical_name: view.entity_logical_name().as_str().to_owned(),
                logical_name: view.logical_name().as_str().to_owned(),
                display_name: view.display_name().as_str().to_owned(),
                view_type: view.view_type(),
                columns: view.columns().to_vec(),
                default_sort: view.default_sort().cloned(),
                filter_criteria: view.filter_criteria().cloned(),
                is_default: view.is_default(),
            },
        )
        .await;
    assert!(saved.is_ok());
}

#[test]
fn compute_form_surface_delta_reports_change_types() {
    let draft_forms = vec![
        test_form("added_form", "Added Form", FormType::QuickCreate, &["name"]),
        test_form(
            "unchanged_form",
            "Unchanged Form",
            FormType::Main,
            &["name"],
        ),
        test_form(
            "updated_form",
            "Updated Form Draft",
            FormType::QuickCreate,
            &["name", "email"],
        ),
    ];
    let published_forms = vec![
        test_form(
            "removed_form",
            "Removed Form",
            FormType::QuickCreate,
            &["name"],
        ),
        test_form(
            "unchanged_form",
            "Unchanged Form",
            FormType::Main,
            &["name"],
        ),
        test_form(
            "updated_form",
            "Updated Form Published",
            FormType::Main,
            &["name"],
        ),
    ];

    let by_name = compute_form_surface_delta(&draft_forms, &published_forms)
        .into_iter()
        .map(|item| (item.logical_name, item.change_type))
        .collect::<HashMap<_, _>>();

    assert_eq!(by_name.get("added_form").map(String::as_str), Some("added"));
    assert_eq!(
        by_name.get("removed_form").map(String::as_str),
        Some("removed")
    );
    assert_eq!(
        by_name.get("updated_form").map(String::as_str),
        Some("updated")
    );
    assert_eq!(
        by_name.get("unchanged_form").map(String::as_str),
        Some("unchanged")
    );
}

#[test]
fn compute_view_surface_delta_reports_change_types() {
    let draft_views = vec![
        test_view("added_view", "Added View", false, &["name"]),
        test_view("unchanged_view", "Unchanged View", false, &["name"]),
        test_view(
            "updated_view",
            "Updated View Draft",
            true,
            &["name", "email"],
        ),
    ];
    let published_views = vec![
        test_view("removed_view", "Removed View", false, &["name"]),
        test_view("unchanged_view", "Unchanged View", false, &["name"]),
        test_view("updated_view", "Updated View Published", false, &["name"]),
    ];

    let by_name = compute_view_surface_delta(&draft_views, &published_views)
        .into_iter()
        .map(|item| (item.logical_name, item.change_type))
        .collect::<HashMap<_, _>>();

    assert_eq!(by_name.get("added_view").map(String::as_str), Some("added"));
    assert_eq!(
        by_name.get("removed_view").map(String::as_str),
        Some("removed")
    );
    assert_eq!(
        by_name.get("updated_view").map(String::as_str),
        Some("updated")
    );
    assert_eq!(
        by_name.get("unchanged_view").map(String::as_str),
        Some("unchanged")
    );
}

#[tokio::test]
async fn workspace_publish_diff_handler_compares_against_latest_published_snapshots() {
    let (state, actor) = build_publish_state().await;

    save_text_field(&state, &actor, "name", "Name").await;
    save_text_field(&state, &actor, "email", "Email").await;

    assert!(
        state
            .metadata_service
            .publish_entity(&actor, "contact")
            .await
            .is_ok()
    );

    save_form_definition(
        &state,
        &actor,
        test_form(
            "unchanged_form",
            "Unchanged Form",
            FormType::QuickCreate,
            &["name"],
        ),
    )
    .await;
    save_form_definition(
        &state,
        &actor,
        test_form(
            "removed_form",
            "Removed Form",
            FormType::QuickCreate,
            &["name"],
        ),
    )
    .await;
    save_form_definition(
        &state,
        &actor,
        test_form(
            "updated_form",
            "Updated Form v1",
            FormType::QuickCreate,
            &["name"],
        ),
    )
    .await;

    save_view_definition(
        &state,
        &actor,
        test_view("unchanged_view", "Unchanged View", false, &["name"]),
    )
    .await;
    save_view_definition(
        &state,
        &actor,
        test_view("removed_view", "Removed View", false, &["name"]),
    )
    .await;
    save_view_definition(
        &state,
        &actor,
        test_view("updated_view", "Updated View v1", false, &["name"]),
    )
    .await;

    assert!(
        state
            .metadata_service
            .publish_entity(&actor, "contact")
            .await
            .is_ok()
    );

    save_form_definition(
        &state,
        &actor,
        test_form(
            "updated_form",
            "Updated Form v2",
            FormType::QuickCreate,
            &["name"],
        ),
    )
    .await;
    save_view_definition(
        &state,
        &actor,
        test_view("updated_view", "Updated View v2", false, &["name"]),
    )
    .await;

    assert!(
        state
            .metadata_service
            .publish_entity(&actor, "contact")
            .await
            .is_ok()
    );

    save_form_definition(
        &state,
        &actor,
        test_form(
            "updated_form",
            "Updated Form v3",
            FormType::QuickCreate,
            &["name", "email"],
        ),
    )
    .await;
    assert!(
        state
            .metadata_service
            .delete_form(&actor, "contact", "removed_form")
            .await
            .is_ok()
    );
    save_form_definition(
        &state,
        &actor,
        test_form("added_form", "Added Form", FormType::QuickCreate, &["name"]),
    )
    .await;

    save_view_definition(
        &state,
        &actor,
        test_view("updated_view", "Updated View v3", false, &["name", "email"]),
    )
    .await;
    assert!(
        state
            .metadata_service
            .delete_view(&actor, "contact", "removed_view")
            .await
            .is_ok()
    );
    save_view_definition(
        &state,
        &actor,
        test_view("added_view", "Added View", false, &["name"]),
    )
    .await;

    save_text_field(&state, &actor, "phone", "Phone").await;

    let response = workspace_publish_diff_handler(
        State(state),
        Extension(actor),
        Json(WorkspacePublishDiffRequest {
            entity_logical_names: vec![
                "contact".to_owned(),
                "missing_entity".to_owned(),
                "contact".to_owned(),
            ],
            app_logical_names: vec![
                "sales".to_owned(),
                "missing_app".to_owned(),
                "sales".to_owned(),
            ],
        }),
    )
    .await;

    assert!(response.is_ok());
    let Json(payload) = response.unwrap_or_else(|_| unreachable!());

    assert_eq!(
        payload.unknown_entity_logical_names,
        vec!["missing_entity".to_owned()]
    );
    assert_eq!(
        payload.unknown_app_logical_names,
        vec!["missing_app".to_owned()]
    );
    assert_eq!(payload.entity_diffs.len(), 1);
    assert_eq!(payload.app_diffs.len(), 1);

    let entity_diff = &payload.entity_diffs[0];
    assert_eq!(entity_diff.entity_logical_name, "contact");
    assert!(entity_diff.published_schema_exists);

    let field_changes = entity_diff
        .field_diff
        .iter()
        .map(|item| (item.field_logical_name.clone(), item.change_type.clone()))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        field_changes.get("phone").map(String::as_str),
        Some("added")
    );

    let form_changes = entity_diff
        .forms
        .iter()
        .map(|item| (item.logical_name.clone(), item.change_type.clone()))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        form_changes.get("added_form").map(String::as_str),
        Some("added")
    );
    assert_eq!(
        form_changes.get("removed_form").map(String::as_str),
        Some("removed")
    );
    assert_eq!(
        form_changes.get("updated_form").map(String::as_str),
        Some("updated")
    );
    assert_eq!(
        form_changes.get("unchanged_form").map(String::as_str),
        Some("unchanged")
    );

    let updated_form = entity_diff
        .forms
        .iter()
        .find(|item| item.logical_name == "updated_form")
        .unwrap_or_else(|| unreachable!());
    assert_eq!(
        updated_form.published_display_name.as_deref(),
        Some("Updated Form v2")
    );
    assert_eq!(
        updated_form.draft_display_name.as_deref(),
        Some("Updated Form v3")
    );

    let view_changes = entity_diff
        .views
        .iter()
        .map(|item| (item.logical_name.clone(), item.change_type.clone()))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        view_changes.get("added_view").map(String::as_str),
        Some("added")
    );
    assert_eq!(
        view_changes.get("removed_view").map(String::as_str),
        Some("removed")
    );
    assert_eq!(
        view_changes.get("updated_view").map(String::as_str),
        Some("updated")
    );
    assert_eq!(
        view_changes.get("unchanged_view").map(String::as_str),
        Some("unchanged")
    );

    let updated_view = entity_diff
        .views
        .iter()
        .find(|item| item.logical_name == "updated_view")
        .unwrap_or_else(|| unreachable!());
    assert_eq!(
        updated_view.published_display_name.as_deref(),
        Some("Updated View v2")
    );
    assert_eq!(
        updated_view.draft_display_name.as_deref(),
        Some("Updated View v3")
    );
}

#[tokio::test]
async fn post_publish_checks_returns_unknown_selection_and_dependency_edge() {
    let (state, actor) = build_publish_state().await;
    let response = run_workspace_publish_handler(
        State(state),
        Extension(actor),
        Json(RunWorkspacePublishRequest {
            entity_logical_names: vec!["missing_entity".to_owned()],
            app_logical_names: vec!["sales".to_owned()],
            dry_run: false,
        }),
    )
    .await;

    assert!(response.is_ok());
    let Json(payload) = response.unwrap_or_else(|_| unreachable!());
    let body = serde_json::to_value(payload).unwrap_or_else(|_| json!({}));
    let issues = body
        .get("issues")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|issue| {
        issue.get("category").and_then(|value| value.as_str()) == Some("unknown")
            && issue.get("scope").and_then(|value| value.as_str()) == Some("entity")
    }));
    assert!(issues.iter().any(|issue| {
        issue
            .get("dependency_path")
            .and_then(|value| value.as_str())
            == Some("sales -> contact")
    }));
}

#[tokio::test]
async fn post_publish_checks_reports_entity_relation_dependency_edge() {
    let (state, actor) = build_publish_state().await;

    let created_account = state
        .metadata_service
        .register_entity(&actor, "account", "Account")
        .await;
    assert!(created_account.is_ok());

    let saved_relation = state
        .metadata_service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "contact".to_owned(),
                logical_name: "account_id".to_owned(),
                display_name: "Account".to_owned(),
                field_type: FieldType::Relation,
                is_required: false,
                is_unique: false,
                default_value: None,
                relation_target_entity: Some("account".to_owned()),
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await;
    assert!(saved_relation.is_ok());

    let response = run_workspace_publish_handler(
        State(state),
        Extension(actor),
        Json(RunWorkspacePublishRequest {
            entity_logical_names: vec!["contact".to_owned()],
            app_logical_names: Vec::new(),
            dry_run: true,
        }),
    )
    .await;

    assert!(response.is_ok());
    let Json(payload) = response.unwrap_or_else(|_| unreachable!());
    let body = serde_json::to_value(payload).unwrap_or_else(|_| json!({}));
    let issues = body
        .get("issues")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    assert!(issues.iter().any(|issue| {
        issue
            .get("dependency_path")
            .and_then(|value| value.as_str())
            == Some("contact.account_id -> account")
    }));
}

#[tokio::test]
async fn run_workspace_publish_allows_selected_relation_dependencies() {
    let (state, actor) = build_publish_state().await;

    let created_account = state
        .metadata_service
        .register_entity(&actor, "account", "Account")
        .await;
    assert!(created_account.is_ok());

    let saved_contact_name = state
        .metadata_service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "contact".to_owned(),
                logical_name: "name".to_owned(),
                display_name: "Name".to_owned(),
                field_type: FieldType::Text,
                is_required: true,
                is_unique: false,
                default_value: None,
                relation_target_entity: None,
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await;
    assert!(saved_contact_name.is_ok());

    let published_contact_v1 = state
        .metadata_service
        .publish_entity(&actor, "contact")
        .await;
    assert!(published_contact_v1.is_ok());

    let saved_account_name = state
        .metadata_service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "account".to_owned(),
                logical_name: "name".to_owned(),
                display_name: "Name".to_owned(),
                field_type: FieldType::Text,
                is_required: true,
                is_unique: false,
                default_value: None,
                relation_target_entity: None,
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await;
    assert!(saved_account_name.is_ok());

    let saved_relation = state
        .metadata_service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "contact".to_owned(),
                logical_name: "account_id".to_owned(),
                display_name: "Account".to_owned(),
                field_type: FieldType::Relation,
                is_required: false,
                is_unique: false,
                default_value: None,
                relation_target_entity: Some("account".to_owned()),
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await;
    assert!(saved_relation.is_ok());

    save_form_definition(
        &state,
        &actor,
        test_form("main_form", "Main Form", FormType::Main, &["name"]),
    )
    .await;
    save_view_definition(
        &state,
        &actor,
        test_view("main_view", "Main View", false, &["name"]),
    )
    .await;

    let response = run_workspace_publish_handler(
        State(state.clone()),
        Extension(actor.clone()),
        Json(RunWorkspacePublishRequest {
            entity_logical_names: vec!["contact".to_owned(), "account".to_owned()],
            app_logical_names: vec!["sales".to_owned()],
            dry_run: false,
        }),
    )
    .await;

    assert!(response.is_ok());
    let Json(payload) = response.unwrap_or_else(|_| unreachable!());
    assert!(payload.is_publishable);
    assert!(payload.issues.is_empty());
    assert_eq!(
        payload.published_entities,
        vec!["contact".to_owned(), "account".to_owned()]
    );

    let contact_schema = state
        .metadata_service
        .latest_published_schema(&actor, "contact")
        .await;
    assert!(contact_schema.is_ok());
    assert!(contact_schema.unwrap_or_else(|_| unreachable!()).is_some());

    let account_schema = state
        .metadata_service
        .latest_published_schema(&actor, "account")
        .await;
    assert!(account_schema.is_ok());
    assert!(account_schema.unwrap_or_else(|_| unreachable!()).is_some());
}

#[tokio::test]
async fn publish_history_endpoint_returns_latest_runs_first() {
    let (state, actor) = build_publish_state().await;
    let _ = run_workspace_publish_handler(
        State(state.clone()),
        Extension(actor.clone()),
        Json(RunWorkspacePublishRequest {
            entity_logical_names: vec!["missing_entity".to_owned()],
            app_logical_names: vec!["sales".to_owned()],
            dry_run: false,
        }),
    )
    .await;

    let _ = run_workspace_publish_handler(
        State(state.clone()),
        Extension(actor.clone()),
        Json(RunWorkspacePublishRequest {
            entity_logical_names: vec!["contact".to_owned()],
            app_logical_names: vec!["sales".to_owned()],
            dry_run: false,
        }),
    )
    .await;

    let history = workspace_publish_history_handler(
        State(state),
        Extension(actor),
        Query(PublishHistoryQuery { limit: Some(10) }),
    )
    .await;
    assert!(history.is_ok());

    let Json(entries) = history.unwrap_or_else(|_| unreachable!());
    assert!(entries.len() >= 2);

    let latest = &entries[0];
    let previous = &entries[1];
    assert_ne!(latest.run_id, previous.run_id);
    assert_eq!(latest.subject, "maker");
    assert!(!latest.run_at.is_empty());
    assert!(!latest.requested_entity_logical_names.is_empty());
    assert!(!latest.requested_app_logical_names.is_empty());
}

#[tokio::test]
async fn dry_run_publish_does_not_write_history_entry() {
    let (state, actor) = build_publish_state().await;
    let response = run_workspace_publish_handler(
        State(state.clone()),
        Extension(actor.clone()),
        Json(RunWorkspacePublishRequest {
            entity_logical_names: vec!["contact".to_owned()],
            app_logical_names: vec!["sales".to_owned()],
            dry_run: true,
        }),
    )
    .await;
    assert!(response.is_ok());

    let history = workspace_publish_history_handler(
        State(state),
        Extension(actor),
        Query(PublishHistoryQuery { limit: Some(10) }),
    )
    .await;
    assert!(history.is_ok());

    let Json(entries) = history.unwrap_or_else(|_| unreachable!());
    assert!(entries.is_empty());
}

#[tokio::test]
async fn publish_history_limit_is_clamped() {
    let (state, actor) = build_publish_state().await;

    for index in 0..120 {
        let recorded = state
            .security_admin_service
            .record_workspace_publish_run(
                &actor,
                WorkspacePublishRunAuditInput {
                    requested_entities: 1,
                    requested_apps: 1,
                    requested_entity_logical_names: vec!["contact".to_owned()],
                    requested_app_logical_names: vec!["sales".to_owned()],
                    published_entities: vec!["contact".to_owned()],
                    validated_apps: vec!["sales".to_owned()],
                    issue_count: index % 2,
                    is_publishable: index % 2 == 0,
                },
            )
            .await;
        assert!(recorded.is_ok());
    }

    let low_limit = workspace_publish_history_handler(
        State(state.clone()),
        Extension(actor.clone()),
        Query(PublishHistoryQuery { limit: Some(0) }),
    )
    .await;
    assert!(low_limit.is_ok());
    let Json(low_entries) = low_limit.unwrap_or_else(|_| unreachable!());
    assert_eq!(low_entries.len(), 1);

    let default_limit = workspace_publish_history_handler(
        State(state.clone()),
        Extension(actor.clone()),
        Query(PublishHistoryQuery { limit: None }),
    )
    .await;
    assert!(default_limit.is_ok());
    let Json(default_entries) = default_limit.unwrap_or_else(|_| unreachable!());
    assert_eq!(default_entries.len(), 20);

    let high_limit = workspace_publish_history_handler(
        State(state),
        Extension(actor),
        Query(PublishHistoryQuery {
            limit: Some(10_000),
        }),
    )
    .await;
    assert!(high_limit.is_ok());
    let Json(high_entries) = high_limit.unwrap_or_else(|_| unreachable!());
    assert_eq!(high_entries.len(), 100);
}

#[tokio::test]
async fn run_workspace_publish_deduplicates_requested_selections() {
    let (state, actor) = build_publish_state().await;

    let response = run_workspace_publish_handler(
        State(state),
        Extension(actor),
        Json(RunWorkspacePublishRequest {
            entity_logical_names: vec![
                "missing_entity".to_owned(),
                "contact".to_owned(),
                "missing_entity".to_owned(),
            ],
            app_logical_names: vec![
                "sales".to_owned(),
                "missing_app".to_owned(),
                "sales".to_owned(),
            ],
            dry_run: false,
        }),
    )
    .await;

    assert!(response.is_ok());
    let Json(payload) = response.unwrap_or_else(|_| unreachable!());
    assert_eq!(payload.requested_entities, 2);
    assert_eq!(payload.requested_apps, 2);

    let unknown_issues = payload
        .issues
        .iter()
        .filter(|issue| matches!(issue.category, PublishCheckCategoryDto::Unknown))
        .collect::<Vec<_>>();
    assert_eq!(unknown_issues.len(), 2);
    assert!(unknown_issues.iter().any(|issue| {
        matches!(issue.scope, PublishCheckScopeDto::Entity)
            && issue.scope_logical_name == "missing_entity"
    }));
    assert!(unknown_issues.iter().any(|issue| {
        matches!(issue.scope, PublishCheckScopeDto::App)
            && issue.scope_logical_name == "missing_app"
    }));
}
