use axum::Json;
use axum::extract::{ConnectInfo, Extension, State};
use axum::response::IntoResponse;
use qryvanta_application::{
    AppEntityFormInput, AppEntityViewInput, BindAppEntityInput, CreateAppInput,
    SaveAppRoleEntityPermissionInput, SaveBusinessRuleInput, SaveFieldInput, SaveFormInput,
    SaveOptionSetInput, SaveViewInput, SaveWorkflowInput, WorkflowExecutionMode,
    WorkflowRunListQuery,
};
use qryvanta_core::UserIdentity;
use qryvanta_domain::{
    BusinessRuleAction, BusinessRuleActionType, BusinessRuleCondition, BusinessRuleOperator,
    BusinessRuleScope, FieldType, FormFieldPlacement, FormSection, FormTab, FormType,
    LogicalMode as ViewLogicalMode, OptionSetItem, SortDirection, ViewColumn, ViewFilterCondition,
    ViewFilterGroup, ViewSort, ViewType, WorkflowAction, WorkflowTrigger,
};
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower_sessions::{MemoryStore, Session};
use uuid::Uuid;

use crate::api_config::{
    ApiConfig, EmailProviderConfig, PhysicalIsolationMode, RateLimitStoreConfig,
    SessionStoreBackend, TotpEncryptionConfig, WorkflowQueueStatsCacheBackend,
};
use crate::api_services::{build_app_state, build_postgres_session_layer};
use crate::dto::{AuthStepUpRequest, CreateRoleRequest};
use crate::state::AppState;

use super::build_router;

static MIGRATOR: Migrator = sqlx::migrate!("../../crates/infrastructure/migrations");

const FRONTEND_URL: &str = "http://localhost:3100";
const TEST_PASSWORD: &str = "Password123!";
const TENANT_OWNER_ROLE: &str = "tenant_owner";
const TOTP_ENCRYPTION_KEY: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct ServerGuard {
    handle: JoinHandle<()>,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct TestHarness {
    base_url: String,
    client: reqwest::Client,
    state: AppState,
    _server: ServerGuard,
}

struct SeededUser {
    actor: UserIdentity,
    email: String,
}

struct SeededScenario {
    left_user: SeededUser,
    shared_app_logical_name: String,
    shared_entity_logical_name: String,
    left_record_id: String,
    right_record_id: String,
    right_secret_form_logical_name: String,
    right_secret_view_logical_name: String,
    right_secret_option_set_logical_name: String,
    right_secret_business_rule_logical_name: String,
    right_hidden_entity_logical_name: String,
    right_hidden_workflow_logical_name: String,
    right_run_id: String,
    right_secret_field_logical_name: String,
}

#[tokio::test]
async fn entity_routes_hide_cross_tenant_metadata_surfaces() {
    let Some(harness) = TestHarness::spawn().await else {
        return;
    };
    let scenario = seed_scenario(&harness.state).await;
    let cookie = harness
        .login(scenario.left_user.email.as_str(), TEST_PASSWORD)
        .await;

    let entity_list = harness
        .request(
            Method::GET,
            "/api/entities",
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(entity_list.status(), StatusCode::OK);
    let entity_list = entity_list
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &entity_list,
        "logical_name",
        scenario.right_hidden_entity_logical_name.as_str(),
    );

    let update_hidden_entity = harness
        .request(
            Method::PUT,
            format!(
                "/api/entities/{}",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "display_name": "Blocked Entity Update",
                "description": null,
                "plural_display_name": null,
                "icon": null
            })),
            true,
        )
        .await;
    assert_eq!(update_hidden_entity.status(), StatusCode::NOT_FOUND);

    let shared_fields = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/fields",
                scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(shared_fields.status(), StatusCode::OK);
    let shared_fields = shared_fields
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &shared_fields,
        "logical_name",
        scenario.right_secret_field_logical_name.as_str(),
    );

    let save_hidden_field = harness
        .request(
            Method::POST,
            format!(
                "/api/entities/{}/fields",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "logical_name": "blocked_field",
                "display_name": "Blocked Field",
                "field_type": "text",
                "is_required": false,
                "is_unique": false,
                "default_value": null,
                "calculation_expression": null,
                "relation_target_entity": null,
                "option_set_logical_name": null
            })),
            true,
        )
        .await;
    assert_eq!(save_hidden_field.status(), StatusCode::NOT_FOUND);

    let update_hidden_field = harness
        .request(
            Method::PUT,
            format!(
                "/api/entities/{}/fields/blocked_field",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "display_name": "Blocked Field",
                "description": null,
                "default_value": null,
                "calculation_expression": null,
                "max_length": null,
                "min_value": null,
                "max_value": null
            })),
            true,
        )
        .await;
    assert_eq!(update_hidden_field.status(), StatusCode::NOT_FOUND);

    let delete_hidden_field = harness
        .request(
            Method::DELETE,
            format!(
                "/api/entities/{}/fields/blocked_field",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            true,
        )
        .await;
    assert_eq!(delete_hidden_field.status(), StatusCode::NOT_FOUND);

    let option_sets = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/option-sets",
                scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(option_sets.status(), StatusCode::OK);
    let option_sets = option_sets
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &option_sets,
        "logical_name",
        scenario.right_secret_option_set_logical_name.as_str(),
    );

    let get_secret_option_set = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/option-sets/{}",
                scenario.shared_entity_logical_name, scenario.right_secret_option_set_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(get_secret_option_set.status(), StatusCode::NOT_FOUND);

    let save_hidden_option_set = harness
        .request(
            Method::POST,
            format!(
                "/api/entities/{}/option-sets",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "logical_name": "blocked_status",
                "display_name": "Blocked Status",
                "options": [{
                    "value": 1,
                    "label": "Blocked",
                    "color": "#991b1b",
                    "position": 0
                }]
            })),
            true,
        )
        .await;
    assert_eq!(save_hidden_option_set.status(), StatusCode::NOT_FOUND);

    let delete_hidden_option_set = harness
        .request(
            Method::DELETE,
            format!(
                "/api/entities/{}/option-sets/blocked_status",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            true,
        )
        .await;
    assert_eq!(delete_hidden_option_set.status(), StatusCode::NOT_FOUND);

    let forms = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/forms",
                scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(forms.status(), StatusCode::OK);
    let forms = forms
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &forms,
        "logical_name",
        scenario.right_secret_form_logical_name.as_str(),
    );

    let get_secret_form = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/forms/{}",
                scenario.shared_entity_logical_name, scenario.right_secret_form_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(get_secret_form.status(), StatusCode::NOT_FOUND);

    let save_hidden_form = harness
        .request(
            Method::POST,
            format!(
                "/api/entities/{}/forms",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "logical_name": "blocked_form",
                "display_name": "Blocked Form",
                "form_type": "main",
                "tabs": form_tabs_json(),
                "header_fields": []
            })),
            true,
        )
        .await;
    assert_eq!(save_hidden_form.status(), StatusCode::NOT_FOUND);

    let delete_hidden_form = harness
        .request(
            Method::DELETE,
            format!(
                "/api/entities/{}/forms/blocked_form",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            true,
        )
        .await;
    assert_eq!(delete_hidden_form.status(), StatusCode::NOT_FOUND);

    let views = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/views",
                scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(views.status(), StatusCode::OK);
    let views = views
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &views,
        "logical_name",
        scenario.right_secret_view_logical_name.as_str(),
    );

    let get_secret_view = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/views/{}",
                scenario.shared_entity_logical_name, scenario.right_secret_view_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(get_secret_view.status(), StatusCode::NOT_FOUND);

    let save_hidden_view = harness
        .request(
            Method::POST,
            format!(
                "/api/entities/{}/views",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "logical_name": "blocked_view",
                "display_name": "Blocked View",
                "view_type": "grid",
                "columns": view_columns_json(),
                "default_sort": view_sort_json(),
                "filter_criteria": view_filter_group_json(),
                "is_default": false
            })),
            true,
        )
        .await;
    assert_eq!(save_hidden_view.status(), StatusCode::NOT_FOUND);

    let delete_hidden_view = harness
        .request(
            Method::DELETE,
            format!(
                "/api/entities/{}/views/blocked_view",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            true,
        )
        .await;
    assert_eq!(delete_hidden_view.status(), StatusCode::NOT_FOUND);

    let business_rules = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/business-rules",
                scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(business_rules.status(), StatusCode::OK);
    let business_rules = business_rules
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &business_rules,
        "logical_name",
        scenario.right_secret_business_rule_logical_name.as_str(),
    );

    let get_secret_business_rule = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/business-rules/{}",
                scenario.shared_entity_logical_name,
                scenario.right_secret_business_rule_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(get_secret_business_rule.status(), StatusCode::NOT_FOUND);

    let save_hidden_business_rule = harness
        .request(
            Method::POST,
            format!(
                "/api/entities/{}/business-rules",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "logical_name": "blocked_rule",
                "display_name": "Blocked Rule",
                "scope": "entity",
                "form_logical_name": null,
                "conditions": business_rule_conditions_json(),
                "actions": business_rule_actions_json(),
                "is_active": true
            })),
            true,
        )
        .await;
    assert_eq!(save_hidden_business_rule.status(), StatusCode::NOT_FOUND);

    let delete_hidden_business_rule = harness
        .request(
            Method::DELETE,
            format!(
                "/api/entities/{}/business-rules/blocked_rule",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            true,
        )
        .await;
    assert_eq!(delete_hidden_business_rule.status(), StatusCode::NOT_FOUND);

    let publish_checks = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/publish-checks",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(publish_checks.status(), StatusCode::NOT_FOUND);

    let publish_hidden_entity = harness
        .request(
            Method::POST,
            format!(
                "/api/entities/{}/publish",
                scenario.right_hidden_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            true,
        )
        .await;
    assert_eq!(publish_hidden_entity.status(), StatusCode::NOT_FOUND);

    let published_schema = harness
        .request(
            Method::GET,
            format!(
                "/api/entities/{}/published",
                scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(published_schema.status(), StatusCode::OK);
    let published_schema = published_schema
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &published_schema["fields"],
        "logical_name",
        scenario.right_secret_field_logical_name.as_str(),
    );
}

#[tokio::test]
async fn workspace_routes_reject_cross_tenant_records_and_components() {
    let Some(harness) = TestHarness::spawn().await else {
        return;
    };
    let scenario = seed_scenario(&harness.state).await;
    let cookie = harness
        .login(scenario.left_user.email.as_str(), TEST_PASSWORD)
        .await;

    let records = harness
        .request(
            Method::GET,
            format!(
                "/api/workspace/apps/{}/entities/{}/records",
                scenario.shared_app_logical_name, scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(records.status(), StatusCode::OK);
    let records = records
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_contains_string(&records, "record_id", scenario.left_record_id.as_str());
    assert_array_missing_string(&records, "record_id", scenario.right_record_id.as_str());

    let query_records = harness
        .request(
            Method::POST,
            format!(
                "/api/workspace/apps/{}/entities/{}/records/query",
                scenario.shared_app_logical_name, scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "limit": 25,
                "offset": 0,
                "logical_mode": "and",
                "filters": {
                    "name": "Left Record"
                }
            })),
            true,
        )
        .await;
    assert_eq!(query_records.status(), StatusCode::OK);
    let query_records = query_records
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_contains_string(
        &query_records,
        "record_id",
        scenario.left_record_id.as_str(),
    );
    assert_array_missing_string(
        &query_records,
        "record_id",
        scenario.right_record_id.as_str(),
    );

    let get_foreign_record = harness
        .request(
            Method::GET,
            format!(
                "/api/workspace/apps/{}/entities/{}/records/{}",
                scenario.shared_app_logical_name,
                scenario.shared_entity_logical_name,
                scenario.right_record_id
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(get_foreign_record.status(), StatusCode::NOT_FOUND);

    let update_foreign_record = harness
        .request(
            Method::PUT,
            format!(
                "/api/workspace/apps/{}/entities/{}/records/{}",
                scenario.shared_app_logical_name,
                scenario.shared_entity_logical_name,
                scenario.right_record_id
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "data": {
                    "name": "Compromised"
                }
            })),
            true,
        )
        .await;
    assert_eq!(update_foreign_record.status(), StatusCode::NOT_FOUND);

    let delete_foreign_record = harness
        .request(
            Method::DELETE,
            format!(
                "/api/workspace/apps/{}/entities/{}/records/{}",
                scenario.shared_app_logical_name,
                scenario.shared_entity_logical_name,
                scenario.right_record_id
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            true,
        )
        .await;
    assert_eq!(delete_foreign_record.status(), StatusCode::NOT_FOUND);

    let workspace_forms = harness
        .request(
            Method::GET,
            format!(
                "/api/workspace/apps/{}/entities/{}/forms",
                scenario.shared_app_logical_name, scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(workspace_forms.status(), StatusCode::OK);
    let workspace_forms = workspace_forms
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &workspace_forms,
        "logical_name",
        scenario.right_secret_form_logical_name.as_str(),
    );

    let get_secret_workspace_form = harness
        .request(
            Method::GET,
            format!(
                "/api/workspace/apps/{}/entities/{}/forms/{}",
                scenario.shared_app_logical_name,
                scenario.shared_entity_logical_name,
                scenario.right_secret_form_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(get_secret_workspace_form.status(), StatusCode::NOT_FOUND);

    let workspace_views = harness
        .request(
            Method::GET,
            format!(
                "/api/workspace/apps/{}/entities/{}/views",
                scenario.shared_app_logical_name, scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(workspace_views.status(), StatusCode::OK);
    let workspace_views = workspace_views
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &workspace_views,
        "logical_name",
        scenario.right_secret_view_logical_name.as_str(),
    );

    let get_secret_workspace_view = harness
        .request(
            Method::GET,
            format!(
                "/api/workspace/apps/{}/entities/{}/views/{}",
                scenario.shared_app_logical_name,
                scenario.shared_entity_logical_name,
                scenario.right_secret_view_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(get_secret_workspace_view.status(), StatusCode::NOT_FOUND);

    let workspace_schema = harness
        .request(
            Method::GET,
            format!(
                "/api/workspace/apps/{}/entities/{}/schema",
                scenario.shared_app_logical_name, scenario.shared_entity_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(workspace_schema.status(), StatusCode::OK);
    let workspace_schema = workspace_schema
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &workspace_schema["fields"],
        "logical_name",
        scenario.right_secret_field_logical_name.as_str(),
    );
}

#[tokio::test]
async fn workflow_routes_hide_cross_tenant_runs_and_dispatch_surfaces() {
    let Some(harness) = TestHarness::spawn().await else {
        return;
    };
    let scenario = seed_scenario(&harness.state).await;
    let cookie = harness
        .login(scenario.left_user.email.as_str(), TEST_PASSWORD)
        .await;

    let workflows = harness
        .request(
            Method::GET,
            "/api/workflows",
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(workflows.status(), StatusCode::OK);
    let workflows = workflows
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &workflows,
        "logical_name",
        scenario.right_hidden_workflow_logical_name.as_str(),
    );

    let execute_hidden_workflow = harness
        .request(
            Method::POST,
            format!(
                "/api/workflows/{}/execute",
                scenario.right_hidden_workflow_logical_name
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "trigger_payload": {
                    "source": "idor_probe"
                }
            })),
            true,
        )
        .await;
    assert_eq!(execute_hidden_workflow.status(), StatusCode::NOT_FOUND);

    let list_runs = harness
        .request(
            Method::GET,
            "/api/workflows/runs?limit=25&offset=0",
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(list_runs.status(), StatusCode::OK);
    let list_runs = list_runs
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(&list_runs, "run_id", scenario.right_run_id.as_str());

    let foreign_attempts = harness
        .request(
            Method::GET,
            format!("/api/workflows/runs/{}/attempts", scenario.right_run_id).as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(foreign_attempts.status(), StatusCode::OK);
    let foreign_attempts = foreign_attempts
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(foreign_attempts, json!([]));

    let replay_foreign_run = harness
        .request(
            Method::GET,
            format!(
                "/api/workflows/shared_ops/runs/{}/replay",
                scenario.right_run_id
            )
            .as_str(),
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(replay_foreign_run.status(), StatusCode::NOT_FOUND);

    let retry_foreign_run = harness
        .request(
            Method::POST,
            format!(
                "/api/workflows/shared_ops/runs/{}/retry-step",
                scenario.right_run_id
            )
            .as_str(),
            Some(cookie.as_str()),
            Some(json!({
                "step_path": "0",
                "strategy": "immediate",
                "backoff_ms": null
            })),
            true,
        )
        .await;
    assert_eq!(retry_foreign_run.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn schedule_dispatch_stays_tenant_scoped_for_shared_schedule_keys() {
    let Some(harness) = TestHarness::spawn().await else {
        return;
    };

    let suffix = Uuid::new_v4().simple().to_string();
    let shared_schedule_key = format!("hourly_{suffix}");
    let left_workflow_logical_name = format!("left_schedule_{suffix}");
    let right_workflow_logical_name = format!("right_schedule_{suffix}");

    let left_user = seed_user(
        &harness.state,
        format!("left_schedule_{suffix}@example.com").as_str(),
        "Left Schedule",
    )
    .await;
    let right_user = seed_user(
        &harness.state,
        format!("right_schedule_{suffix}@example.com").as_str(),
        "Right Schedule",
    )
    .await;

    let left_workflow = save_schedule_workflow(
        &harness.state,
        &left_user.actor,
        left_workflow_logical_name.as_str(),
        shared_schedule_key.as_str(),
    )
    .await;
    let right_workflow = save_schedule_workflow(
        &harness.state,
        &right_user.actor,
        right_workflow_logical_name.as_str(),
        shared_schedule_key.as_str(),
    )
    .await;

    let cookie = harness.login(left_user.email.as_str(), TEST_PASSWORD).await;

    let dispatch_response = harness
        .request(
            Method::POST,
            "/api/workflows/triggers/schedule/dispatch",
            Some(cookie.as_str()),
            Some(json!({
                "schedule_key": shared_schedule_key,
                "payload": {
                    "tick_at": "2026-03-05T12:00:00Z",
                    "source": "tenant-isolation-regression"
                }
            })),
            true,
        )
        .await;
    assert_eq!(dispatch_response.status(), StatusCode::OK);
    let dispatched = dispatch_response
        .json::<usize>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(dispatched, 1);

    let left_runs = harness
        .state
        .workflow_service
        .list_runs(
            &left_user.actor,
            WorkflowRunListQuery {
                workflow_logical_name: Some(left_workflow.logical_name().as_str().to_owned()),
                limit: 10,
                offset: 0,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(left_runs.len(), 1);
    assert_eq!(
        left_runs[0].workflow_logical_name,
        left_workflow_logical_name
    );
    assert_eq!(left_runs[0].trigger_type, "schedule_tick");
    assert_eq!(
        left_runs[0].trigger_payload["schedule_key"],
        json!(shared_schedule_key)
    );

    let right_runs = harness
        .state
        .workflow_service
        .list_runs(
            &right_user.actor,
            WorkflowRunListQuery {
                workflow_logical_name: Some(right_workflow.logical_name().as_str().to_owned()),
                limit: 10,
                offset: 0,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(right_runs.is_empty());
}

#[tokio::test]
async fn auth_me_exposes_available_tenants_and_switching_updates_scope() {
    let Some(harness) = TestHarness::spawn().await else {
        return;
    };

    let suffix = Uuid::new_v4().simple().to_string();
    let alpha_entity_logical_name = format!("alpha_entity_{suffix}");
    let alpha_app_logical_name = format!("alpha_app_{suffix}");
    let bravo_entity_logical_name = format!("bravo_entity_{suffix}");
    let bravo_app_logical_name = format!("bravo_app_{suffix}");

    let alpha_user = seed_user(
        &harness.state,
        format!("alpha_member_{suffix}@example.com").as_str(),
        "Alpha",
    )
    .await;
    let bravo_owner = seed_user(
        &harness.state,
        format!("bravo_owner_{suffix}@example.com").as_str(),
        "Bravo",
    )
    .await;

    harness
        .state
        .tenant_repository
        .create_membership(
            bravo_owner.actor.tenant_id(),
            alpha_user.actor.subject(),
            "Alpha in Bravo",
            Some(alpha_user.email.as_str()),
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    seed_workspace_surface(
        &harness.state,
        &alpha_user.actor,
        WorkspaceSurfaceSeed {
            entity_logical_name: alpha_entity_logical_name.as_str(),
            app_logical_name: alpha_app_logical_name.as_str(),
            extra_field_logical_name: None,
            extra_option_set_logical_name: None,
            extra_form_logical_name: None,
            extra_view_logical_name: None,
        },
    )
    .await;
    seed_workspace_surface(
        &harness.state,
        &bravo_owner.actor,
        WorkspaceSurfaceSeed {
            entity_logical_name: bravo_entity_logical_name.as_str(),
            app_logical_name: bravo_app_logical_name.as_str(),
            extra_field_logical_name: None,
            extra_option_set_logical_name: None,
            extra_form_logical_name: None,
            extra_view_logical_name: None,
        },
    )
    .await;

    let cookie = harness
        .login(alpha_user.email.as_str(), TEST_PASSWORD)
        .await;

    let me_response = harness
        .request(Method::GET, "/auth/me", Some(cookie.as_str()), None, false)
        .await;
    assert_eq!(me_response.status(), StatusCode::OK);
    let me_payload = me_response
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        me_payload["tenant_id"].as_str(),
        Some(alpha_user.actor.tenant_id().to_string().as_str())
    );
    assert_eq!(me_payload["display_name"].as_str(), Some("Alpha"));
    assert_tenant_option_state(
        &me_payload["available_tenants"],
        alpha_user.actor.tenant_id().to_string().as_str(),
        true,
        true,
    );
    assert_tenant_option_state(
        &me_payload["available_tenants"],
        bravo_owner.actor.tenant_id().to_string().as_str(),
        false,
        false,
    );

    let alpha_entities = harness
        .request(
            Method::GET,
            "/api/entities",
            Some(cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(alpha_entities.status(), StatusCode::OK);
    let alpha_entities = alpha_entities
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_contains_string(
        &alpha_entities,
        "logical_name",
        alpha_entity_logical_name.as_str(),
    );
    assert_array_missing_string(
        &alpha_entities,
        "logical_name",
        bravo_entity_logical_name.as_str(),
    );

    let switch_response = harness
        .request(
            Method::POST,
            "/auth/switch-tenant",
            Some(cookie.as_str()),
            Some(json!({
                "tenant_id": bravo_owner.actor.tenant_id().to_string()
            })),
            true,
        )
        .await;
    assert_eq!(switch_response.status(), StatusCode::OK);
    let switched_cookie = session_cookie(&switch_response);
    let switch_payload = switch_response
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        switch_payload["tenant_id"].as_str(),
        Some(bravo_owner.actor.tenant_id().to_string().as_str())
    );
    assert_eq!(
        switch_payload["display_name"].as_str(),
        Some("Alpha in Bravo")
    );
    assert_tenant_option_state(
        &switch_payload["available_tenants"],
        alpha_user.actor.tenant_id().to_string().as_str(),
        false,
        false,
    );
    assert_tenant_option_state(
        &switch_payload["available_tenants"],
        bravo_owner.actor.tenant_id().to_string().as_str(),
        true,
        true,
    );

    let switched_entities = harness
        .request(
            Method::GET,
            "/api/entities",
            Some(switched_cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(switched_entities.status(), StatusCode::OK);
    let switched_entities = switched_entities
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_array_missing_string(
        &switched_entities,
        "logical_name",
        alpha_entity_logical_name.as_str(),
    );
    assert_array_contains_string(
        &switched_entities,
        "logical_name",
        bravo_entity_logical_name.as_str(),
    );

    let switched_me_response = harness
        .request(
            Method::GET,
            "/auth/me",
            Some(switched_cookie.as_str()),
            None,
            false,
        )
        .await;
    assert_eq!(switched_me_response.status(), StatusCode::OK);
    let switched_me_payload = switched_me_response
        .json::<Value>()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        switched_me_payload["tenant_id"].as_str(),
        Some(bravo_owner.actor.tenant_id().to_string().as_str())
    );
    assert_tenant_option_state(
        &switched_me_payload["available_tenants"],
        alpha_user.actor.tenant_id().to_string().as_str(),
        false,
        false,
    );
    assert_tenant_option_state(
        &switched_me_payload["available_tenants"],
        bravo_owner.actor.tenant_id().to_string().as_str(),
        true,
        true,
    );
}

#[tokio::test]
async fn high_risk_security_actions_require_recent_step_up() {
    let Some(harness) = TestHarness::spawn().await else {
        return;
    };

    let suffix = Uuid::new_v4().simple().to_string();
    let actor = seed_user(
        &harness.state,
        format!("step_up_admin_{suffix}@example.com").as_str(),
        "Step Up Admin",
    )
    .await;
    let session_store = Arc::new(MemoryStore::default());
    let session = Session::new(None, session_store, None);
    session
        .insert("step_up_verified_at", 0_i64)
        .await
        .unwrap_or_else(|_| unreachable!());

    let blocked_response = match crate::handlers::security::create_role_handler(
        State(harness.state.clone()),
        Extension(actor.actor.clone()),
        session.clone(),
        Json(CreateRoleRequest {
            name: format!("auditor_{suffix}"),
            permissions: vec!["security.audit.read".to_owned()],
        }),
    )
    .await
    {
        Ok(_) => panic!("expected step-up protected role creation to be rejected"),
        Err(error) => error.into_response(),
    };
    assert_eq!(blocked_response.status(), StatusCode::FORBIDDEN);
    let blocked_payload = axum::body::to_bytes(blocked_response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|_| unreachable!());
    let blocked_payload: Value =
        serde_json::from_slice(blocked_payload.as_ref()).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        blocked_payload["code"].as_str(),
        Some("forbidden.step_up_required")
    );
    assert_eq!(
        blocked_payload["message"].as_str(),
        Some("forbidden: step-up authentication required for this action")
    );

    let step_up_response = crate::auth::step_up_handler(
        State(harness.state.clone()),
        axum::http::HeaderMap::new(),
        ConnectInfo("127.0.0.1:4000".parse().unwrap_or_else(|_| unreachable!())),
        Extension(actor.actor.clone()),
        session.clone(),
        Json(AuthStepUpRequest {
            password: Some(TEST_PASSWORD.to_owned()),
            code: None,
            method: None,
        }),
    )
    .await
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(step_up_response, StatusCode::NO_CONTENT);

    let allowed_response = crate::handlers::security::create_role_handler(
        State(harness.state),
        Extension(actor.actor),
        session,
        Json(CreateRoleRequest {
            name: format!("auditor_{suffix}"),
            permissions: vec!["security.audit.read".to_owned()],
        }),
    )
    .await
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(allowed_response.0, StatusCode::CREATED);
}

impl TestHarness {
    async fn spawn() -> Option<Self> {
        let pool = test_pool().await?;
        let database_url = std::env::var("DATABASE_URL").ok()?;
        let config = test_config(database_url.as_str());
        let state = build_app_state(pool.clone(), &config).unwrap_or_else(|_| unreachable!());
        let session_layer = build_postgres_session_layer(pool, config.cookie_secure)
            .await
            .unwrap_or_else(|_| unreachable!());
        let app = build_router(state.clone(), config.frontend_url.as_str(), session_layer)
            .unwrap_or_else(|_| unreachable!());

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|_| unreachable!());
        let address = listener.local_addr().unwrap_or_else(|_| unreachable!());
        let server = tokio::spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await;
        });

        Some(Self {
            base_url: format!("http://{address}"),
            client: reqwest::Client::new(),
            state,
            _server: ServerGuard { handle: server },
        })
    }

    async fn login(&self, email: &str, password: &str) -> String {
        let response = self
            .request(
                Method::POST,
                "/auth/login",
                None,
                Some(json!({
                    "email": email,
                    "password": password
                })),
                true,
            )
            .await;
        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| unreachable!());
            panic!("login failed with status {status}: {body}");
        }

        let cookie = session_cookie(&response);
        let _ = response.bytes().await.unwrap_or_else(|_| unreachable!());
        cookie
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        cookie: Option<&str>,
        body: Option<Value>,
        include_origin: bool,
    ) -> reqwest::Response {
        let mut request = self
            .client
            .request(method, format!("{}{}", self.base_url, path));

        if let Some(cookie) = cookie {
            request = request.header(reqwest::header::COOKIE, cookie);
        }

        if include_origin {
            request = request.header(reqwest::header::ORIGIN, FRONTEND_URL);
        }

        if let Some(body) = body {
            request = request.json(&body);
        }

        request.send().await.unwrap_or_else(|_| unreachable!())
    }
}

async fn test_pool() -> Option<PgPool> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        return None;
    };

    let pool = PgPoolOptions::new()
        .max_connections(6)
        .connect(database_url.as_str())
        .await
        .unwrap_or_else(|_| unreachable!());

    MIGRATOR.run(&pool).await.unwrap_or_else(|_| unreachable!());
    Some(pool)
}

fn test_config(database_url: &str) -> ApiConfig {
    ApiConfig {
        migrate_only: false,
        database_url: database_url.to_owned(),
        frontend_url: FRONTEND_URL.to_owned(),
        bootstrap_token: "bootstrap-test-token-32-bytes-minimum".to_owned(),
        _session_secret: "session-secret-with-more-than-32-bytes".to_owned(),
        api_host: "127.0.0.1".to_owned(),
        api_port: 0,
        session_store_backend: SessionStoreBackend::Postgres,
        webauthn_rp_id: "localhost".to_owned(),
        webauthn_rp_origin: FRONTEND_URL.to_owned(),
        cookie_secure: false,
        trust_proxy_headers: false,
        trusted_proxy_cidrs: Vec::new(),
        bootstrap_tenant_id: None,
        totp_encryption: TotpEncryptionConfig::StaticKey {
            key_hex: TOTP_ENCRYPTION_KEY.to_owned(),
        },
        email_provider: EmailProviderConfig::Console,
        workflow_execution_mode: WorkflowExecutionMode::Inline,
        worker_shared_secret: None,
        redis_url: None,
        rate_limit_store: RateLimitStoreConfig::Postgres,
        workflow_queue_stats_cache_backend: WorkflowQueueStatsCacheBackend::InMemory,
        workflow_worker_default_lease_seconds: 60,
        workflow_worker_max_claim_limit: 25,
        workflow_worker_max_partition_count: 8,
        workflow_queue_stats_cache_ttl_seconds: 2,
        runtime_query_max_limit: 200,
        runtime_query_max_in_flight: 8,
        workflow_burst_max_in_flight: 8,
        audit_immutable_mode: true,
        slow_request_threshold_ms: 2_000,
        slow_query_threshold_ms: 2_000,
        physical_isolation_mode: PhysicalIsolationMode::Shared,
        physical_isolation_tenant_id: None,
        physical_isolation_schema_template: None,
        physical_isolation_database_url_template: None,
        qrywell_api_base_url: None,
        qrywell_api_key: None,
        qrywell_sync_poll_interval_ms: 5_000,
        qrywell_sync_batch_size: 100,
        qrywell_sync_max_attempts: 3,
    }
}

async fn seed_scenario(state: &AppState) -> SeededScenario {
    let suffix = Uuid::new_v4().simple().to_string();
    let shared_entity_logical_name = format!("account_{suffix}");
    let shared_app_logical_name = format!("ops_{suffix}");
    let right_hidden_entity_logical_name = format!("hidden_entity_{suffix}");
    let right_secret_form_logical_name = format!("secret_form_{suffix}");
    let right_secret_view_logical_name = format!("secret_view_{suffix}");
    let right_secret_option_set_logical_name = format!("secret_status_{suffix}");
    let right_secret_business_rule_logical_name = format!("secret_rule_{suffix}");
    let right_secret_field_logical_name = format!("secret_code_{suffix}");
    let right_hidden_workflow_logical_name = format!("hidden_workflow_{suffix}");

    let left_user = seed_user(
        state,
        format!("left_{suffix}@example.com").as_str(),
        format!("Left {suffix}").as_str(),
    )
    .await;
    let right_user = seed_user(
        state,
        format!("right_{suffix}@example.com").as_str(),
        format!("Right {suffix}").as_str(),
    )
    .await;

    seed_workspace_surface(
        state,
        &left_user.actor,
        WorkspaceSurfaceSeed {
            entity_logical_name: shared_entity_logical_name.as_str(),
            app_logical_name: shared_app_logical_name.as_str(),
            extra_field_logical_name: None,
            extra_option_set_logical_name: None,
            extra_form_logical_name: None,
            extra_view_logical_name: None,
        },
    )
    .await;

    let left_record = state
        .app_service
        .create_record(
            &left_user.actor,
            shared_app_logical_name.as_str(),
            shared_entity_logical_name.as_str(),
            json!({ "name": "Left Record" }),
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let left_workflow = save_manual_workflow(state, &left_user.actor, "shared_ops").await;
    let _ = state
        .workflow_service
        .execute_workflow(
            &left_user.actor,
            left_workflow.logical_name().as_str(),
            json!({
                "source": "left-seed"
            }),
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    seed_workspace_surface(
        state,
        &right_user.actor,
        WorkspaceSurfaceSeed {
            entity_logical_name: shared_entity_logical_name.as_str(),
            app_logical_name: shared_app_logical_name.as_str(),
            extra_field_logical_name: Some(right_secret_field_logical_name.as_str()),
            extra_option_set_logical_name: Some(right_secret_option_set_logical_name.as_str()),
            extra_form_logical_name: Some(right_secret_form_logical_name.as_str()),
            extra_view_logical_name: Some(right_secret_view_logical_name.as_str()),
        },
    )
    .await;

    state
        .metadata_service
        .save_business_rule(
            &right_user.actor,
            SaveBusinessRuleInput {
                entity_logical_name: shared_entity_logical_name.clone(),
                logical_name: right_secret_business_rule_logical_name.clone(),
                display_name: "Secret Rule".to_owned(),
                scope: BusinessRuleScope::Entity,
                form_logical_name: None,
                conditions: minimal_business_rule_conditions(),
                actions: minimal_business_rule_actions(),
                is_active: true,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let right_record = state
        .app_service
        .create_record(
            &right_user.actor,
            shared_app_logical_name.as_str(),
            shared_entity_logical_name.as_str(),
            json!({
                "name": "Right Record",
                right_secret_field_logical_name.clone(): "RIGHT-ONLY"
            }),
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    seed_hidden_entity(
        state,
        &right_user.actor,
        right_hidden_entity_logical_name.as_str(),
    )
    .await;

    let right_shared_workflow = save_manual_workflow(state, &right_user.actor, "shared_ops").await;
    let right_run = state
        .workflow_service
        .execute_workflow(
            &right_user.actor,
            right_shared_workflow.logical_name().as_str(),
            json!({ "source": "right-seed" }),
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let _ = save_manual_workflow(
        state,
        &right_user.actor,
        right_hidden_workflow_logical_name.as_str(),
    )
    .await;

    SeededScenario {
        left_user,
        shared_app_logical_name,
        shared_entity_logical_name,
        left_record_id: left_record.record_id().as_str().to_owned(),
        right_record_id: right_record.record_id().as_str().to_owned(),
        right_secret_form_logical_name,
        right_secret_view_logical_name,
        right_secret_option_set_logical_name,
        right_secret_business_rule_logical_name,
        right_hidden_entity_logical_name,
        right_hidden_workflow_logical_name,
        right_run_id: right_run.run_id,
        right_secret_field_logical_name,
    }
}

async fn seed_user(state: &AppState, email: &str, display_name: &str) -> SeededUser {
    let password_hash = state
        .user_service
        .password_hasher()
        .hash_password(TEST_PASSWORD)
        .unwrap_or_else(|_| unreachable!());
    let user_id = state
        .user_service
        .user_repository()
        .create(email, Some(password_hash.as_str()), true)
        .await
        .unwrap_or_else(|_| unreachable!());
    let tenant_id = state
        .tenant_repository
        .ensure_membership_for_subject(
            user_id.to_string().as_str(),
            display_name,
            Some(email),
            None,
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    SeededUser {
        actor: UserIdentity::new(
            user_id.to_string(),
            email.to_owned(),
            Some(email.to_owned()),
            tenant_id,
        ),
        email: email.to_owned(),
    }
}

struct WorkspaceSurfaceSeed<'a> {
    entity_logical_name: &'a str,
    app_logical_name: &'a str,
    extra_field_logical_name: Option<&'a str>,
    extra_option_set_logical_name: Option<&'a str>,
    extra_form_logical_name: Option<&'a str>,
    extra_view_logical_name: Option<&'a str>,
}

async fn seed_workspace_surface(
    state: &AppState,
    actor: &UserIdentity,
    seed: WorkspaceSurfaceSeed<'_>,
) {
    state
        .metadata_service
        .register_entity(actor, seed.entity_logical_name, "Account")
        .await
        .unwrap_or_else(|_| unreachable!());
    state
        .metadata_service
        .save_field(
            actor,
            SaveFieldInput {
                entity_logical_name: seed.entity_logical_name.to_owned(),
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
        .await
        .unwrap_or_else(|_| unreachable!());
    state
        .metadata_service
        .publish_entity(actor, seed.entity_logical_name)
        .await
        .unwrap_or_else(|_| unreachable!());

    if let Some(extra_field_logical_name) = seed.extra_field_logical_name {
        state
            .metadata_service
            .save_field(
                actor,
                SaveFieldInput {
                    entity_logical_name: seed.entity_logical_name.to_owned(),
                    logical_name: extra_field_logical_name.to_owned(),
                    display_name: "Secret Code".to_owned(),
                    field_type: FieldType::Text,
                    is_required: false,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                    option_set_logical_name: None,
                    calculation_expression: None,
                },
            )
            .await
            .unwrap_or_else(|_| unreachable!());
        state
            .metadata_service
            .publish_entity(actor, seed.entity_logical_name)
            .await
            .unwrap_or_else(|_| unreachable!());
    }

    if let Some(extra_option_set_logical_name) = seed.extra_option_set_logical_name {
        state
            .metadata_service
            .save_option_set(
                actor,
                SaveOptionSetInput {
                    entity_logical_name: seed.entity_logical_name.to_owned(),
                    logical_name: extra_option_set_logical_name.to_owned(),
                    display_name: "Secret Status".to_owned(),
                    options: vec![
                        OptionSetItem::new(1, "Hidden", Some("#1d4ed8".to_owned()), 0)
                            .unwrap_or_else(|_| unreachable!()),
                    ],
                },
            )
            .await
            .unwrap_or_else(|_| unreachable!());
    }

    if let Some(extra_form_logical_name) = seed.extra_form_logical_name {
        state
            .metadata_service
            .save_form(
                actor,
                SaveFormInput {
                    entity_logical_name: seed.entity_logical_name.to_owned(),
                    logical_name: extra_form_logical_name.to_owned(),
                    display_name: "Secret Form".to_owned(),
                    form_type: FormType::Main,
                    tabs: minimal_form_tabs(),
                    header_fields: Vec::new(),
                },
            )
            .await
            .unwrap_or_else(|_| unreachable!());
    }

    if let Some(extra_view_logical_name) = seed.extra_view_logical_name {
        state
            .metadata_service
            .save_view(
                actor,
                SaveViewInput {
                    entity_logical_name: seed.entity_logical_name.to_owned(),
                    logical_name: extra_view_logical_name.to_owned(),
                    display_name: "Secret View".to_owned(),
                    view_type: ViewType::Grid,
                    columns: minimal_view_columns(),
                    default_sort: Some(
                        ViewSort::new("name", SortDirection::Asc)
                            .unwrap_or_else(|_| unreachable!()),
                    ),
                    filter_criteria: Some(
                        ViewFilterGroup::new(
                            ViewLogicalMode::And,
                            vec![
                                ViewFilterCondition::new(
                                    "name",
                                    qryvanta_domain::FilterOperator::Contains,
                                    json!("Record"),
                                )
                                .unwrap_or_else(|_| unreachable!()),
                            ],
                        )
                        .unwrap_or_else(|_| unreachable!()),
                    ),
                    is_default: false,
                },
            )
            .await
            .unwrap_or_else(|_| unreachable!());
    }

    state
        .app_service
        .create_app(
            actor,
            CreateAppInput {
                logical_name: seed.app_logical_name.to_owned(),
                display_name: "Operations".to_owned(),
                description: Some("Operations workspace".to_owned()),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());
    state
        .app_service
        .bind_entity(
            actor,
            BindAppEntityInput {
                app_logical_name: seed.app_logical_name.to_owned(),
                entity_logical_name: seed.entity_logical_name.to_owned(),
                navigation_label: Some("Accounts".to_owned()),
                navigation_order: 0,
                forms: Some(vec![AppEntityFormInput {
                    logical_name: "main_form".to_owned(),
                    display_name: "Main Form".to_owned(),
                    field_logical_names: vec!["name".to_owned()],
                }]),
                list_views: Some(vec![AppEntityViewInput {
                    logical_name: "main_view".to_owned(),
                    display_name: "Main View".to_owned(),
                    field_logical_names: vec!["name".to_owned()],
                }]),
                default_form_logical_name: Some("main_form".to_owned()),
                default_list_view_logical_name: Some("main_view".to_owned()),
                form_field_logical_names: None,
                list_field_logical_names: None,
                default_view_mode: None,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());
    state
        .app_service
        .save_role_entity_permission(
            actor,
            SaveAppRoleEntityPermissionInput {
                app_logical_name: seed.app_logical_name.to_owned(),
                role_name: TENANT_OWNER_ROLE.to_owned(),
                entity_logical_name: seed.entity_logical_name.to_owned(),
                can_read: true,
                can_create: true,
                can_update: true,
                can_delete: true,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());
}

async fn seed_hidden_entity(state: &AppState, actor: &UserIdentity, entity_logical_name: &str) {
    state
        .metadata_service
        .register_entity(actor, entity_logical_name, "Hidden Entity")
        .await
        .unwrap_or_else(|_| unreachable!());
    state
        .metadata_service
        .save_field(
            actor,
            SaveFieldInput {
                entity_logical_name: entity_logical_name.to_owned(),
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
        .await
        .unwrap_or_else(|_| unreachable!());
    state
        .metadata_service
        .publish_entity(actor, entity_logical_name)
        .await
        .unwrap_or_else(|_| unreachable!());
}

async fn save_manual_workflow(
    state: &AppState,
    actor: &UserIdentity,
    logical_name: &str,
) -> qryvanta_domain::WorkflowDefinition {
    state
        .workflow_service
        .save_workflow(
            actor,
            SaveWorkflowInput {
                logical_name: logical_name.to_owned(),
                display_name: logical_name.replace('_', " "),
                description: Some("Manual test workflow".to_owned()),
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::LogMessage {
                    message: "manual".to_owned(),
                },
                steps: None,
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await
        .unwrap_or_else(|error| panic!("failed to save scheduled workflow: {error}"))
}

async fn save_schedule_workflow(
    state: &AppState,
    actor: &UserIdentity,
    logical_name: &str,
    schedule_key: &str,
) -> qryvanta_domain::WorkflowDefinition {
    state
        .workflow_service
        .save_workflow(
            actor,
            SaveWorkflowInput {
                logical_name: logical_name.to_owned(),
                display_name: logical_name.replace('_', " "),
                description: Some("Scheduled test workflow".to_owned()),
                trigger: WorkflowTrigger::ScheduleTick {
                    schedule_key: schedule_key.to_owned(),
                },
                action: WorkflowAction::LogMessage {
                    message: "schedule".to_owned(),
                },
                steps: None,
                max_attempts: 1,
                is_enabled: true,
            },
        )
        .await
        .unwrap_or_else(|error| panic!("failed to save scheduled workflow: {error}"))
}

fn minimal_form_tabs() -> Vec<FormTab> {
    let field = FormFieldPlacement::new("name", 0, 0, true, false, None, None)
        .unwrap_or_else(|_| unreachable!());
    let section = FormSection::new(
        "main_section",
        "Main Section",
        0,
        true,
        1,
        vec![field],
        Vec::new(),
    )
    .unwrap_or_else(|_| unreachable!());
    vec![
        FormTab::new("main_tab", "Main Tab", 0, true, vec![section])
            .unwrap_or_else(|_| unreachable!()),
    ]
}

fn minimal_view_columns() -> Vec<ViewColumn> {
    vec![ViewColumn::new("name", 0, None, None).unwrap_or_else(|_| unreachable!())]
}

fn minimal_business_rule_conditions() -> Vec<BusinessRuleCondition> {
    vec![
        BusinessRuleCondition::new("name", BusinessRuleOperator::Eq, json!("Probe"))
            .unwrap_or_else(|_| unreachable!()),
    ]
}

fn minimal_business_rule_actions() -> Vec<BusinessRuleAction> {
    vec![
        BusinessRuleAction::new(
            BusinessRuleActionType::SetRequired,
            Some("name".to_owned()),
            None,
            None,
        )
        .unwrap_or_else(|_| unreachable!()),
    ]
}

fn form_tabs_json() -> Vec<Value> {
    minimal_form_tabs()
        .into_iter()
        .map(|tab| serde_json::to_value(tab).unwrap_or_else(|_| unreachable!()))
        .collect()
}

fn view_columns_json() -> Vec<Value> {
    minimal_view_columns()
        .into_iter()
        .map(|column| serde_json::to_value(column).unwrap_or_else(|_| unreachable!()))
        .collect()
}

fn view_sort_json() -> Value {
    serde_json::to_value(
        ViewSort::new("name", SortDirection::Asc).unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!())
}

fn view_filter_group_json() -> Value {
    serde_json::to_value(
        ViewFilterGroup::new(
            ViewLogicalMode::And,
            vec![
                ViewFilterCondition::new(
                    "name",
                    qryvanta_domain::FilterOperator::Contains,
                    json!("Record"),
                )
                .unwrap_or_else(|_| unreachable!()),
            ],
        )
        .unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!())
}

fn business_rule_conditions_json() -> Vec<Value> {
    minimal_business_rule_conditions()
        .into_iter()
        .map(|condition| serde_json::to_value(condition).unwrap_or_else(|_| unreachable!()))
        .collect()
}

fn business_rule_actions_json() -> Vec<Value> {
    minimal_business_rule_actions()
        .into_iter()
        .map(|action| serde_json::to_value(action).unwrap_or_else(|_| unreachable!()))
        .collect()
}

fn assert_array_missing_string(value: &Value, field_name: &str, expected_absent: &str) {
    let entries = value.as_array().unwrap_or_else(|| unreachable!());
    assert!(!entries.iter().any(|entry| {
        entry
            .get(field_name)
            .and_then(Value::as_str)
            .map(|value| value == expected_absent)
            .unwrap_or(false)
    }));
}

fn assert_array_contains_string(value: &Value, field_name: &str, expected_present: &str) {
    let entries = value.as_array().unwrap_or_else(|| unreachable!());
    assert!(entries.iter().any(|entry| {
        entry
            .get(field_name)
            .and_then(Value::as_str)
            .map(|value| value == expected_present)
            .unwrap_or(false)
    }));
}

fn assert_tenant_option_state(
    value: &Value,
    tenant_id: &str,
    expected_current: bool,
    expected_default: bool,
) {
    let entries = value.as_array().unwrap_or_else(|| unreachable!());
    let tenant = entries
        .iter()
        .find(|entry| {
            entry
                .get("tenant_id")
                .and_then(Value::as_str)
                .map(|value| value == tenant_id)
                .unwrap_or(false)
        })
        .unwrap_or_else(|| unreachable!());

    assert_eq!(
        tenant.get("is_current").and_then(Value::as_bool),
        Some(expected_current)
    );
    assert_eq!(
        tenant.get("is_default").and_then(Value::as_bool),
        Some(expected_default)
    );
}

fn session_cookie(response: &reqwest::Response) -> String {
    response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            value
                .to_str()
                .ok()
                .and_then(|cookie| cookie.split(';').next().map(str::to_owned))
        })
        .unwrap_or_else(|| unreachable!())
}
