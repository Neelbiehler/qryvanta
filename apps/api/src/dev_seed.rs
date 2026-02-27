use crate::api_config::ApiConfig;
use crate::api_services;

use qryvanta_application::{
    AppEntityFormInput, AppEntityViewInput, AppService, BindAppEntityInput, CreateAppInput,
    CreateRoleInput, MetadataService, PasswordHasher, RecordListQuery,
    SaveAppRoleEntityPermissionInput, SaveAppSitemapInput, SaveFieldInput, SaveFormInput,
    SaveViewInput, SaveWorkflowInput, SecurityAdminService, WorkflowService,
};
use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppEntityViewMode, AppSitemap, FieldType, FormFieldPlacement, FormSection, FormTab, FormType,
    Permission, SitemapArea, SitemapGroup, SitemapSubArea, SitemapTarget, SortDirection,
    ViewColumn, ViewSort, ViewType, WorkflowAction, WorkflowConditionOperator, WorkflowStep,
    WorkflowTrigger,
};

use qryvanta_infrastructure::Argon2PasswordHasher;

use serde_json::{Value, json};
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use std::collections::HashSet;

const DEV_SEED_TENANT_ID: &str = "11111111-1111-1111-1111-111111111111";
const DEV_SEED_TENANT_NAME: &str = "Northwind Industrial Group";
const DEV_SEED_ADMIN_USER_ID: &str = "a2c8ea5f-4f39-4724-97f5-932f97f54f76";
const DEV_SEED_ADMIN_EMAIL: &str = "admin@qryvanta.local";
const DEV_SEED_ADMIN_DISPLAY_NAME: &str = "Tenant Admin";
const DEV_SEED_ADMIN_PASSWORD: &str = "admin";

const DEV_SEED_STANDARD_USER_ID: &str = "96d11e90-7403-4654-9727-cb1043f8bd31";
const DEV_SEED_STANDARD_EMAIL: &str = "user@qryvanta.local";
const DEV_SEED_STANDARD_DISPLAY_NAME: &str = "Standard User";
const DEV_SEED_STANDARD_PASSWORD: &str = "admin";
const DEV_SEED_STANDARD_ROLE: &str = "standard_user";

pub async fn run(pool: PgPool, config: &ApiConfig) -> AppResult<()> {
    let app_state = api_services::build_app_state(pool.clone(), config)?;

    let preferred_tenant_id = config
        .bootstrap_tenant_id
        .unwrap_or(default_dev_seed_tenant_id()?);
    let admin_user_id = parse_uuid_const(DEV_SEED_ADMIN_USER_ID, "DEV_SEED_ADMIN_USER_ID")?;
    let standard_user_id =
        parse_uuid_const(DEV_SEED_STANDARD_USER_ID, "DEV_SEED_STANDARD_USER_ID")?;

    ensure_seed_user(
        &pool,
        admin_user_id,
        DEV_SEED_ADMIN_EMAIL,
        DEV_SEED_ADMIN_PASSWORD,
    )
    .await?;
    ensure_seed_user(
        &pool,
        standard_user_id,
        DEV_SEED_STANDARD_EMAIL,
        DEV_SEED_STANDARD_PASSWORD,
    )
    .await?;

    let admin_subject = admin_user_id.to_string();
    let standard_subject = standard_user_id.to_string();

    let tenant_id = app_state
        .tenant_repository
        .ensure_membership_for_subject(
            admin_subject.as_str(),
            DEV_SEED_ADMIN_DISPLAY_NAME,
            Some(DEV_SEED_ADMIN_EMAIL),
            Some(preferred_tenant_id),
        )
        .await?;

    app_state
        .tenant_repository
        .create_membership(
            tenant_id,
            admin_subject.as_str(),
            DEV_SEED_ADMIN_DISPLAY_NAME,
            Some(DEV_SEED_ADMIN_EMAIL),
        )
        .await?;

    ensure_standard_membership(
        &pool,
        tenant_id,
        standard_user_id,
        standard_subject.as_str(),
    )
    .await?;

    sqlx::query(
        r#"
        UPDATE tenant_memberships
        SET user_id = $3, display_name = $4, email = $5
        WHERE tenant_id = $1 AND subject = $2
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(admin_subject.as_str())
    .bind(admin_user_id)
    .bind(DEV_SEED_ADMIN_DISPLAY_NAME)
    .bind(DEV_SEED_ADMIN_EMAIL)
    .execute(&pool)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to link tenant membership to dev seed user: {error}"
        ))
    })?;

    sqlx::query("UPDATE tenants SET name = $2 WHERE id = $1")
        .bind(tenant_id.as_uuid())
        .bind(DEV_SEED_TENANT_NAME)
        .execute(&pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to rename dev seed tenant: {error}"))
        })?;

    app_state
        .contact_bootstrap_service
        .ensure_subject_contact(
            tenant_id,
            admin_subject.as_str(),
            DEV_SEED_ADMIN_DISPLAY_NAME,
            Some(DEV_SEED_ADMIN_EMAIL),
        )
        .await?;

    app_state
        .contact_bootstrap_service
        .ensure_subject_contact(
            tenant_id,
            standard_subject.as_str(),
            DEV_SEED_STANDARD_DISPLAY_NAME,
            Some(DEV_SEED_STANDARD_EMAIL),
        )
        .await?;

    let actor = UserIdentity::new(
        admin_subject,
        DEV_SEED_ADMIN_DISPLAY_NAME,
        Some(DEV_SEED_ADMIN_EMAIL.to_owned()),
        tenant_id,
    );

    ensure_roles_and_assignments(
        &app_state.security_admin_service,
        &actor,
        standard_subject.as_str(),
    )
    .await?;
    ensure_crm_erp_schema(&app_state.metadata_service, &actor).await?;
    seed_model_driven_surfaces(&app_state.app_service, &app_state.metadata_service, &actor).await?;
    seed_app_role_permissions(&app_state.app_service, &actor).await?;
    seed_crm_erp_records(&app_state.metadata_service, &actor).await?;
    seed_workflows(&app_state.workflow_service, &actor).await?;

    info!(
        %tenant_id,
        tenant_name = DEV_SEED_TENANT_NAME,
        "development CRM/ERP seed completed"
    );

    Ok(())
}

fn default_dev_seed_tenant_id() -> AppResult<TenantId> {
    let parsed = Uuid::parse_str(DEV_SEED_TENANT_ID).map_err(|error| {
        AppError::Internal(format!(
            "invalid static dev seed tenant id '{DEV_SEED_TENANT_ID}': {error}"
        ))
    })?;
    Ok(TenantId::from_uuid(parsed))
}

fn parse_uuid_const(value: &str, name: &str) -> AppResult<Uuid> {
    Uuid::parse_str(value).map_err(|error| {
        AppError::Internal(format!("invalid static uuid '{name}={value}': {error}"))
    })
}

async fn ensure_seed_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    password: &str,
) -> AppResult<()> {
    let password_hash = Argon2PasswordHasher::new().hash_password(password)?;

    sqlx::query(
        r#"
        INSERT INTO users (id, email, email_verified, password_hash)
        VALUES ($1, LOWER($2), TRUE, $3)
        ON CONFLICT (id) DO UPDATE
        SET email = LOWER(EXCLUDED.email),
            email_verified = TRUE,
            password_hash = EXCLUDED.password_hash,
            failed_login_count = 0,
            locked_until = NULL,
            updated_at = now()
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to ensure seed user exists for '{email}': {error}"
        ))
    })?;

    Ok(())
}

async fn ensure_standard_membership(
    pool: &PgPool,
    tenant_id: TenantId,
    user_id: Uuid,
    subject: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO tenant_memberships (tenant_id, subject, display_name, email, user_id)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (tenant_id, subject) DO UPDATE
        SET display_name = EXCLUDED.display_name,
            email = EXCLUDED.email,
            user_id = EXCLUDED.user_id
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(subject)
    .bind(DEV_SEED_STANDARD_DISPLAY_NAME)
    .bind(DEV_SEED_STANDARD_EMAIL)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to ensure standard user membership for tenant '{}': {error}",
            tenant_id
        ))
    })?;

    sqlx::query(
        r#"
        DELETE FROM rbac_subject_roles
        WHERE tenant_id = $1
          AND subject = $2
          AND role_id IN (
              SELECT id FROM rbac_roles WHERE tenant_id = $1 AND name = 'tenant_owner'
          )
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(subject)
    .execute(pool)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to remove owner role from standard user in tenant '{}': {error}",
            tenant_id
        ))
    })?;

    Ok(())
}

async fn ensure_crm_erp_schema(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
) -> AppResult<()> {
    let mut entity_names: HashSet<String> = metadata_service
        .list_entities(actor)
        .await?
        .into_iter()
        .map(|entity| entity.logical_name().as_str().to_owned())
        .collect();

    ensure_entity(
        metadata_service,
        actor,
        &mut entity_names,
        "account",
        "Account",
        Some("Customer and supplier organizations used by CRM and ERP flows.".to_owned()),
        Some("Accounts".to_owned()),
        Some("building-2".to_owned()),
    )
    .await?;

    save_text_field(
        metadata_service,
        actor,
        "account",
        "account_code",
        "Account Code",
        true,
        true,
    )
    .await?;
    save_text_field(
        metadata_service,
        actor,
        "account",
        "name",
        "Name",
        true,
        false,
    )
    .await?;
    save_text_field(
        metadata_service,
        actor,
        "account",
        "industry",
        "Industry",
        false,
        false,
    )
    .await?;
    save_number_field(
        metadata_service,
        actor,
        "account",
        "annual_revenue",
        "Annual Revenue",
        false,
        false,
    )
    .await?;
    publish_if_missing_fields(
        metadata_service,
        actor,
        "account",
        &["account_code", "name", "industry", "annual_revenue"],
    )
    .await?;

    save_relation_field(
        metadata_service,
        actor,
        "contact",
        "account_id",
        "Account",
        "account",
        false,
        false,
    )
    .await?;
    save_text_field(
        metadata_service,
        actor,
        "contact",
        "job_title",
        "Job Title",
        false,
        false,
    )
    .await?;
    save_text_field(
        metadata_service,
        actor,
        "contact",
        "phone",
        "Phone",
        false,
        false,
    )
    .await?;
    publish_if_missing_fields(
        metadata_service,
        actor,
        "contact",
        &[
            "subject",
            "display_name",
            "email",
            "account_id",
            "job_title",
            "phone",
        ],
    )
    .await?;

    ensure_entity(
        metadata_service,
        actor,
        &mut entity_names,
        "deal",
        "Deal",
        Some("Pipeline opportunities tracked by revenue stage.".to_owned()),
        Some("Deals".to_owned()),
        Some("target".to_owned()),
    )
    .await?;
    save_text_field(
        metadata_service,
        actor,
        "deal",
        "deal_code",
        "Deal Code",
        true,
        true,
    )
    .await?;
    save_text_field(metadata_service, actor, "deal", "name", "Name", true, false).await?;
    save_text_field(
        metadata_service,
        actor,
        "deal",
        "stage",
        "Stage",
        true,
        false,
    )
    .await?;
    save_number_field(
        metadata_service,
        actor,
        "deal",
        "amount",
        "Amount",
        true,
        false,
    )
    .await?;
    save_date_field(
        metadata_service,
        actor,
        "deal",
        "close_date",
        "Close Date",
        false,
        false,
    )
    .await?;
    save_relation_field(
        metadata_service,
        actor,
        "deal",
        "account_id",
        "Account",
        "account",
        true,
        false,
    )
    .await?;
    save_relation_field(
        metadata_service,
        actor,
        "deal",
        "primary_contact_id",
        "Primary Contact",
        "contact",
        true,
        false,
    )
    .await?;
    publish_if_missing_fields(
        metadata_service,
        actor,
        "deal",
        &[
            "deal_code",
            "name",
            "stage",
            "amount",
            "close_date",
            "account_id",
            "primary_contact_id",
        ],
    )
    .await?;

    ensure_entity(
        metadata_service,
        actor,
        &mut entity_names,
        "invoice",
        "Invoice",
        Some("Billing documents linked to won deals.".to_owned()),
        Some("Invoices".to_owned()),
        Some("receipt".to_owned()),
    )
    .await?;
    save_text_field(
        metadata_service,
        actor,
        "invoice",
        "invoice_number",
        "Invoice Number",
        true,
        true,
    )
    .await?;
    save_text_field(
        metadata_service,
        actor,
        "invoice",
        "status",
        "Status",
        true,
        false,
    )
    .await?;
    save_number_field(
        metadata_service,
        actor,
        "invoice",
        "total_amount",
        "Total Amount",
        true,
        false,
    )
    .await?;
    save_date_field(
        metadata_service,
        actor,
        "invoice",
        "due_date",
        "Due Date",
        false,
        false,
    )
    .await?;
    save_relation_field(
        metadata_service,
        actor,
        "invoice",
        "account_id",
        "Account",
        "account",
        true,
        false,
    )
    .await?;
    save_relation_field(
        metadata_service,
        actor,
        "invoice",
        "deal_id",
        "Deal",
        "deal",
        false,
        false,
    )
    .await?;
    publish_if_missing_fields(
        metadata_service,
        actor,
        "invoice",
        &[
            "invoice_number",
            "status",
            "total_amount",
            "due_date",
            "account_id",
            "deal_id",
        ],
    )
    .await
}

async fn seed_crm_erp_records(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
) -> AppResult<()> {
    let aster_account_id = upsert_runtime_record(
        metadata_service,
        actor,
        "account",
        "account_code",
        "ACC-1001",
        json!({
            "account_code": "ACC-1001",
            "name": "Aster Manufacturing",
            "industry": "Industrial Equipment",
            "annual_revenue": 18500000
        }),
    )
    .await?;

    let summit_account_id = upsert_runtime_record(
        metadata_service,
        actor,
        "account",
        "account_code",
        "ACC-1002",
        json!({
            "account_code": "ACC-1002",
            "name": "Summit Retail Group",
            "industry": "Retail",
            "annual_revenue": 9200000
        }),
    )
    .await?;

    let cobalt_account_id = upsert_runtime_record(
        metadata_service,
        actor,
        "account",
        "account_code",
        "ACC-1003",
        json!({
            "account_code": "ACC-1003",
            "name": "Cobalt Logistics",
            "industry": "Logistics",
            "annual_revenue": 12700000
        }),
    )
    .await?;

    let ava_contact_id = upsert_runtime_record(
        metadata_service,
        actor,
        "contact",
        "subject",
        "ava.morgan@aster.example",
        json!({
            "subject": "ava.morgan@aster.example",
            "display_name": "Ava Morgan",
            "email": "ava.morgan@aster.example",
            "account_id": aster_account_id,
            "job_title": "Operations Director",
            "phone": "+1-555-0101"
        }),
    )
    .await?;

    let liam_contact_id = upsert_runtime_record(
        metadata_service,
        actor,
        "contact",
        "subject",
        "liam.chen@summit.example",
        json!({
            "subject": "liam.chen@summit.example",
            "display_name": "Liam Chen",
            "email": "liam.chen@summit.example",
            "account_id": summit_account_id,
            "job_title": "Head of Procurement",
            "phone": "+1-555-0102"
        }),
    )
    .await?;

    let noah_contact_id = upsert_runtime_record(
        metadata_service,
        actor,
        "contact",
        "subject",
        "noah.kim@cobalt.example",
        json!({
            "subject": "noah.kim@cobalt.example",
            "display_name": "Noah Kim",
            "email": "noah.kim@cobalt.example",
            "account_id": cobalt_account_id,
            "job_title": "Fleet Manager",
            "phone": "+1-555-0103"
        }),
    )
    .await?;

    let expansion_deal_id = upsert_runtime_record(
        metadata_service,
        actor,
        "deal",
        "deal_code",
        "DEAL-2026-001",
        json!({
            "deal_code": "DEAL-2026-001",
            "name": "Aster Plant Expansion",
            "stage": "proposal",
            "amount": 425000,
            "close_date": "2026-04-30",
            "account_id": aster_account_id,
            "primary_contact_id": ava_contact_id
        }),
    )
    .await?;

    let automation_deal_id = upsert_runtime_record(
        metadata_service,
        actor,
        "deal",
        "deal_code",
        "DEAL-2026-002",
        json!({
            "deal_code": "DEAL-2026-002",
            "name": "Summit Warehouse Automation",
            "stage": "negotiation",
            "amount": 280000,
            "close_date": "2026-05-15",
            "account_id": summit_account_id,
            "primary_contact_id": liam_contact_id
        }),
    )
    .await?;

    let fleet_deal_id = upsert_runtime_record(
        metadata_service,
        actor,
        "deal",
        "deal_code",
        "DEAL-2026-003",
        json!({
            "deal_code": "DEAL-2026-003",
            "name": "Cobalt Route Optimization",
            "stage": "qualified",
            "amount": 160000,
            "close_date": "2026-06-10",
            "account_id": cobalt_account_id,
            "primary_contact_id": noah_contact_id
        }),
    )
    .await?;

    upsert_runtime_record(
        metadata_service,
        actor,
        "invoice",
        "invoice_number",
        "INV-2026-1001",
        json!({
            "invoice_number": "INV-2026-1001",
            "status": "sent",
            "total_amount": 125000,
            "due_date": "2026-05-10",
            "account_id": aster_account_id,
            "deal_id": expansion_deal_id
        }),
    )
    .await?;

    upsert_runtime_record(
        metadata_service,
        actor,
        "invoice",
        "invoice_number",
        "INV-2026-1002",
        json!({
            "invoice_number": "INV-2026-1002",
            "status": "draft",
            "total_amount": 84000,
            "due_date": "2026-05-25",
            "account_id": summit_account_id,
            "deal_id": automation_deal_id
        }),
    )
    .await?;

    upsert_runtime_record(
        metadata_service,
        actor,
        "invoice",
        "invoice_number",
        "INV-2026-1003",
        json!({
            "invoice_number": "INV-2026-1003",
            "status": "paid",
            "total_amount": 42000,
            "due_date": "2026-04-20",
            "account_id": cobalt_account_id,
            "deal_id": fleet_deal_id
        }),
    )
    .await?;

    Ok(())
}

async fn seed_model_driven_surfaces(
    app_service: &AppService,
    metadata_service: &MetadataService,
    actor: &UserIdentity,
) -> AppResult<()> {
    save_forms_and_views(metadata_service, actor).await?;

    ensure_app_exists(
        app_service,
        actor,
        "sales_hub",
        "Sales Hub",
        Some("Pipeline and account operations workspace.".to_owned()),
    )
    .await?;
    ensure_app_exists(
        app_service,
        actor,
        "finance_ops",
        "Finance Ops",
        Some("Invoice and revenue operations workspace.".to_owned()),
    )
    .await?;

    app_service
        .bind_entity(
            actor,
            BindAppEntityInput {
                app_logical_name: "sales_hub".to_owned(),
                entity_logical_name: "account".to_owned(),
                navigation_label: Some("Accounts".to_owned()),
                navigation_order: 0,
                forms: Some(vec![
                    AppEntityFormInput {
                        logical_name: "main_form".to_owned(),
                        display_name: "Account Profile".to_owned(),
                        field_logical_names: vec![
                            "account_code".to_owned(),
                            "name".to_owned(),
                            "industry".to_owned(),
                            "annual_revenue".to_owned(),
                        ],
                    },
                    AppEntityFormInput {
                        logical_name: "quick_create".to_owned(),
                        display_name: "Quick Create Account".to_owned(),
                        field_logical_names: vec!["account_code".to_owned(), "name".to_owned()],
                    },
                ]),
                list_views: Some(vec![AppEntityViewInput {
                    logical_name: "main_view".to_owned(),
                    display_name: "Account Directory".to_owned(),
                    field_logical_names: vec![
                        "account_code".to_owned(),
                        "name".to_owned(),
                        "industry".to_owned(),
                        "annual_revenue".to_owned(),
                    ],
                }]),
                default_form_logical_name: Some("main_form".to_owned()),
                default_list_view_logical_name: Some("main_view".to_owned()),
                form_field_logical_names: None,
                list_field_logical_names: None,
                default_view_mode: Some(AppEntityViewMode::Grid),
            },
        )
        .await?;

    app_service
        .bind_entity(
            actor,
            BindAppEntityInput {
                app_logical_name: "sales_hub".to_owned(),
                entity_logical_name: "contact".to_owned(),
                navigation_label: Some("Contacts".to_owned()),
                navigation_order: 1,
                forms: Some(vec![AppEntityFormInput {
                    logical_name: "main_form".to_owned(),
                    display_name: "Contact Profile".to_owned(),
                    field_logical_names: vec![
                        "display_name".to_owned(),
                        "email".to_owned(),
                        "account_id".to_owned(),
                        "job_title".to_owned(),
                        "phone".to_owned(),
                    ],
                }]),
                list_views: Some(vec![AppEntityViewInput {
                    logical_name: "main_view".to_owned(),
                    display_name: "Contact Directory".to_owned(),
                    field_logical_names: vec![
                        "display_name".to_owned(),
                        "email".to_owned(),
                        "account_id".to_owned(),
                        "job_title".to_owned(),
                    ],
                }]),
                default_form_logical_name: Some("main_form".to_owned()),
                default_list_view_logical_name: Some("main_view".to_owned()),
                form_field_logical_names: None,
                list_field_logical_names: None,
                default_view_mode: Some(AppEntityViewMode::Grid),
            },
        )
        .await?;

    app_service
        .bind_entity(
            actor,
            BindAppEntityInput {
                app_logical_name: "sales_hub".to_owned(),
                entity_logical_name: "deal".to_owned(),
                navigation_label: Some("Opportunities".to_owned()),
                navigation_order: 2,
                forms: Some(vec![
                    AppEntityFormInput {
                        logical_name: "main_form".to_owned(),
                        display_name: "Opportunity Form".to_owned(),
                        field_logical_names: vec![
                            "deal_code".to_owned(),
                            "name".to_owned(),
                            "stage".to_owned(),
                            "amount".to_owned(),
                            "close_date".to_owned(),
                            "account_id".to_owned(),
                            "primary_contact_id".to_owned(),
                        ],
                    },
                    AppEntityFormInput {
                        logical_name: "quick_create".to_owned(),
                        display_name: "Quick Create Opportunity".to_owned(),
                        field_logical_names: vec![
                            "deal_code".to_owned(),
                            "name".to_owned(),
                            "amount".to_owned(),
                            "account_id".to_owned(),
                        ],
                    },
                ]),
                list_views: Some(vec![
                    AppEntityViewInput {
                        logical_name: "main_view".to_owned(),
                        display_name: "Pipeline".to_owned(),
                        field_logical_names: vec![
                            "deal_code".to_owned(),
                            "name".to_owned(),
                            "stage".to_owned(),
                            "amount".to_owned(),
                            "close_date".to_owned(),
                            "account_id".to_owned(),
                        ],
                    },
                    AppEntityViewInput {
                        logical_name: "open_pipeline".to_owned(),
                        display_name: "Open Opportunities".to_owned(),
                        field_logical_names: vec![
                            "name".to_owned(),
                            "stage".to_owned(),
                            "amount".to_owned(),
                            "close_date".to_owned(),
                        ],
                    },
                ]),
                default_form_logical_name: Some("main_form".to_owned()),
                default_list_view_logical_name: Some("main_view".to_owned()),
                form_field_logical_names: None,
                list_field_logical_names: None,
                default_view_mode: Some(AppEntityViewMode::Grid),
            },
        )
        .await?;

    app_service
        .bind_entity(
            actor,
            BindAppEntityInput {
                app_logical_name: "finance_ops".to_owned(),
                entity_logical_name: "invoice".to_owned(),
                navigation_label: Some("Invoices".to_owned()),
                navigation_order: 0,
                forms: Some(vec![AppEntityFormInput {
                    logical_name: "main_form".to_owned(),
                    display_name: "Invoice Form".to_owned(),
                    field_logical_names: vec![
                        "invoice_number".to_owned(),
                        "status".to_owned(),
                        "total_amount".to_owned(),
                        "due_date".to_owned(),
                        "account_id".to_owned(),
                        "deal_id".to_owned(),
                    ],
                }]),
                list_views: Some(vec![AppEntityViewInput {
                    logical_name: "main_view".to_owned(),
                    display_name: "Invoice Queue".to_owned(),
                    field_logical_names: vec![
                        "invoice_number".to_owned(),
                        "status".to_owned(),
                        "total_amount".to_owned(),
                        "due_date".to_owned(),
                        "account_id".to_owned(),
                    ],
                }]),
                default_form_logical_name: Some("main_form".to_owned()),
                default_list_view_logical_name: Some("main_view".to_owned()),
                form_field_logical_names: None,
                list_field_logical_names: None,
                default_view_mode: Some(AppEntityViewMode::Grid),
            },
        )
        .await?;

    app_service
        .bind_entity(
            actor,
            BindAppEntityInput {
                app_logical_name: "finance_ops".to_owned(),
                entity_logical_name: "account".to_owned(),
                navigation_label: Some("Customers".to_owned()),
                navigation_order: 1,
                forms: Some(vec![AppEntityFormInput {
                    logical_name: "main_form".to_owned(),
                    display_name: "Customer Account".to_owned(),
                    field_logical_names: vec![
                        "account_code".to_owned(),
                        "name".to_owned(),
                        "industry".to_owned(),
                        "annual_revenue".to_owned(),
                    ],
                }]),
                list_views: Some(vec![AppEntityViewInput {
                    logical_name: "main_view".to_owned(),
                    display_name: "Customer Accounts".to_owned(),
                    field_logical_names: vec![
                        "account_code".to_owned(),
                        "name".to_owned(),
                        "annual_revenue".to_owned(),
                    ],
                }]),
                default_form_logical_name: Some("main_form".to_owned()),
                default_list_view_logical_name: Some("main_view".to_owned()),
                form_field_logical_names: None,
                list_field_logical_names: None,
                default_view_mode: Some(AppEntityViewMode::Grid),
            },
        )
        .await?;

    let sales_sitemap = AppSitemap::new(
        "sales_hub",
        vec![
            SitemapArea::new(
                "customers",
                "Customers",
                0,
                Some("users".to_owned()),
                vec![SitemapGroup::new(
                    "crm",
                    "CRM",
                    0,
                    vec![
                        SitemapSubArea::new(
                            "accounts",
                            "Accounts",
                            0,
                            SitemapTarget::Entity {
                                entity_logical_name: "account".to_owned(),
                                default_form: Some("main_form".to_owned()),
                                default_view: Some("main_view".to_owned()),
                            },
                            Some("building-2".to_owned()),
                        )?,
                        SitemapSubArea::new(
                            "contacts",
                            "Contacts",
                            1,
                            SitemapTarget::Entity {
                                entity_logical_name: "contact".to_owned(),
                                default_form: Some("main_form".to_owned()),
                                default_view: Some("main_view".to_owned()),
                            },
                            Some("user-round".to_owned()),
                        )?,
                    ],
                )?],
            )?,
            SitemapArea::new(
                "pipeline",
                "Pipeline",
                1,
                Some("target".to_owned()),
                vec![SitemapGroup::new(
                    "deals",
                    "Deals",
                    0,
                    vec![
                        SitemapSubArea::new(
                            "opportunities",
                            "Opportunities",
                            0,
                            SitemapTarget::Entity {
                                entity_logical_name: "deal".to_owned(),
                                default_form: Some("main_form".to_owned()),
                                default_view: Some("main_view".to_owned()),
                            },
                            Some("target".to_owned()),
                        )?,
                        SitemapSubArea::new(
                            "pipeline_overview",
                            "Pipeline Overview",
                            1,
                            SitemapTarget::Dashboard {
                                dashboard_logical_name: "pipeline_overview".to_owned(),
                            },
                            Some("layout-dashboard".to_owned()),
                        )?,
                    ],
                )?],
            )?,
        ],
    )?;

    app_service
        .save_sitemap(
            actor,
            SaveAppSitemapInput {
                app_logical_name: "sales_hub".to_owned(),
                sitemap: sales_sitemap,
            },
        )
        .await?;

    let finance_sitemap = AppSitemap::new(
        "finance_ops",
        vec![SitemapArea::new(
            "billing",
            "Billing",
            0,
            Some("receipt".to_owned()),
            vec![SitemapGroup::new(
                "receivables",
                "Receivables",
                0,
                vec![
                    SitemapSubArea::new(
                        "invoice_queue",
                        "Invoice Queue",
                        0,
                        SitemapTarget::Entity {
                            entity_logical_name: "invoice".to_owned(),
                            default_form: Some("main_form".to_owned()),
                            default_view: Some("main_view".to_owned()),
                        },
                        Some("file-text".to_owned()),
                    )?,
                    SitemapSubArea::new(
                        "customer_accounts",
                        "Customer Accounts",
                        1,
                        SitemapTarget::Entity {
                            entity_logical_name: "account".to_owned(),
                            default_form: Some("main_form".to_owned()),
                            default_view: Some("main_view".to_owned()),
                        },
                        Some("building-2".to_owned()),
                    )?,
                ],
            )?],
        )?],
    )?;

    app_service
        .save_sitemap(
            actor,
            SaveAppSitemapInput {
                app_logical_name: "finance_ops".to_owned(),
                sitemap: finance_sitemap,
            },
        )
        .await?;

    Ok(())
}

async fn ensure_roles_and_assignments(
    security_admin_service: &SecurityAdminService,
    actor: &UserIdentity,
    standard_subject: &str,
) -> AppResult<()> {
    let existing_roles = security_admin_service.list_roles(actor).await?;
    if !existing_roles
        .iter()
        .any(|role| role.name == DEV_SEED_STANDARD_ROLE)
    {
        security_admin_service
            .create_role(
                actor,
                CreateRoleInput {
                    name: DEV_SEED_STANDARD_ROLE.to_owned(),
                    permissions: vec![
                        Permission::MetadataEntityRead,
                        Permission::MetadataFieldRead,
                        Permission::RuntimeRecordRead,
                        Permission::RuntimeRecordWrite,
                    ],
                },
            )
            .await?;
    }

    security_admin_service
        .assign_role(actor, standard_subject, DEV_SEED_STANDARD_ROLE)
        .await?;

    Ok(())
}

async fn seed_app_role_permissions(
    app_service: &AppService,
    actor: &UserIdentity,
) -> AppResult<()> {
    let role_names = ["tenant_owner", DEV_SEED_STANDARD_ROLE];
    let sales_entities = ["account", "contact", "deal"];
    let finance_entities = ["invoice", "account"];

    for role_name in role_names {
        for entity in sales_entities {
            app_service
                .save_role_entity_permission(
                    actor,
                    SaveAppRoleEntityPermissionInput {
                        app_logical_name: "sales_hub".to_owned(),
                        role_name: role_name.to_owned(),
                        entity_logical_name: entity.to_owned(),
                        can_read: true,
                        can_create: true,
                        can_update: true,
                        can_delete: role_name == "tenant_owner",
                    },
                )
                .await?;
        }

        for entity in finance_entities {
            app_service
                .save_role_entity_permission(
                    actor,
                    SaveAppRoleEntityPermissionInput {
                        app_logical_name: "finance_ops".to_owned(),
                        role_name: role_name.to_owned(),
                        entity_logical_name: entity.to_owned(),
                        can_read: true,
                        can_create: true,
                        can_update: true,
                        can_delete: role_name == "tenant_owner",
                    },
                )
                .await?;
        }
    }

    Ok(())
}

async fn seed_workflows(workflow_service: &WorkflowService, actor: &UserIdentity) -> AppResult<()> {
    workflow_service
        .save_workflow(
            actor,
            SaveWorkflowInput {
                logical_name: "deal_created_notify".to_owned(),
                display_name: "Deal Created Notification".to_owned(),
                description: Some(
                    "Logs a message whenever a new deal record is created.".to_owned(),
                ),
                trigger: WorkflowTrigger::RuntimeRecordCreated {
                    entity_logical_name: "deal".to_owned(),
                },
                action: WorkflowAction::LogMessage {
                    message: "Deal created trigger received".to_owned(),
                },
                steps: Some(vec![WorkflowStep::Condition {
                    field_path: "stage".to_owned(),
                    operator: WorkflowConditionOperator::Equals,
                    value: Some(json!("proposal")),
                    then_label: Some("Proposal".to_owned()),
                    else_label: Some("Other".to_owned()),
                    then_steps: vec![WorkflowStep::LogMessage {
                        message: "Deal entered proposal stage".to_owned(),
                    }],
                    else_steps: vec![WorkflowStep::LogMessage {
                        message: "Deal created in non-proposal stage".to_owned(),
                    }],
                }]),
                max_attempts: 3,
                is_enabled: true,
            },
        )
        .await?;

    workflow_service
        .save_workflow(
            actor,
            SaveWorkflowInput {
                logical_name: "invoice_overdue_followup".to_owned(),
                display_name: "Invoice Overdue Follow-up".to_owned(),
                description: Some(
                    "Creates a follow-up contact record when invoice payload indicates overdue status."
                        .to_owned(),
                ),
                trigger: WorkflowTrigger::Manual,
                action: WorkflowAction::CreateRuntimeRecord {
                    entity_logical_name: "contact".to_owned(),
                    data: json!({
                        "subject": "workflow.followup@qryvanta.local",
                        "display_name": "Workflow Follow-up",
                        "email": "workflow.followup@qryvanta.local",
                        "job_title": "Collections",
                        "phone": "+1-555-0199"
                    }),
                },
                steps: Some(vec![WorkflowStep::LogMessage {
                    message: "Invoice overdue follow-up workflow executed".to_owned(),
                }]),
                max_attempts: 3,
                is_enabled: true,
            },
        )
        .await?;

    Ok(())
}

async fn ensure_app_exists(
    app_service: &AppService,
    actor: &UserIdentity,
    logical_name: &str,
    display_name: &str,
    description: Option<String>,
) -> AppResult<()> {
    let existing_apps = app_service.list_apps(actor).await?;
    if existing_apps
        .iter()
        .any(|app| app.logical_name().as_str() == logical_name)
    {
        return Ok(());
    }

    app_service
        .create_app(
            actor,
            CreateAppInput {
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                description,
            },
        )
        .await?;

    Ok(())
}

async fn save_forms_and_views(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
) -> AppResult<()> {
    save_form(
        metadata_service,
        actor,
        "account",
        "main_form",
        "Account Profile",
        FormType::Main,
        vec![FormTab::new(
            "summary",
            "Summary",
            0,
            true,
            vec![FormSection::new(
                "profile",
                "Profile",
                0,
                true,
                2,
                vec![
                    FormFieldPlacement::new("account_code", 0, 0, true, false, None, None)?,
                    FormFieldPlacement::new("name", 0, 1, true, false, None, None)?,
                    FormFieldPlacement::new("industry", 1, 0, true, false, None, None)?,
                    FormFieldPlacement::new("annual_revenue", 1, 1, true, false, None, None)?,
                ],
                Vec::new(),
            )?],
        )?],
        vec!["account_code".to_owned(), "name".to_owned()],
    )
    .await?;

    save_form(
        metadata_service,
        actor,
        "deal",
        "quick_create",
        "Quick Create Opportunity",
        FormType::QuickCreate,
        vec![FormTab::new(
            "quick",
            "Quick",
            0,
            true,
            vec![FormSection::new(
                "quick_fields",
                "Opportunity",
                0,
                true,
                1,
                vec![
                    FormFieldPlacement::new("deal_code", 0, 0, true, false, None, None)?,
                    FormFieldPlacement::new("name", 0, 1, true, false, None, None)?,
                    FormFieldPlacement::new("amount", 0, 2, true, false, None, None)?,
                    FormFieldPlacement::new("account_id", 0, 3, true, false, None, None)?,
                ],
                Vec::new(),
            )?],
        )?],
        vec![],
    )
    .await?;

    save_form(
        metadata_service,
        actor,
        "deal",
        "main_form",
        "Opportunity Form",
        FormType::Main,
        vec![FormTab::new(
            "pipeline",
            "Pipeline",
            0,
            true,
            vec![FormSection::new(
                "details",
                "Details",
                0,
                true,
                2,
                vec![
                    FormFieldPlacement::new("deal_code", 0, 0, true, false, None, None)?,
                    FormFieldPlacement::new("name", 0, 1, true, false, None, None)?,
                    FormFieldPlacement::new("stage", 0, 2, true, false, None, None)?,
                    FormFieldPlacement::new("amount", 1, 0, true, false, None, None)?,
                    FormFieldPlacement::new("close_date", 1, 1, true, false, None, None)?,
                    FormFieldPlacement::new("account_id", 1, 2, true, false, None, None)?,
                    FormFieldPlacement::new("primary_contact_id", 1, 3, true, false, None, None)?,
                ],
                Vec::new(),
            )?],
        )?],
        vec![
            "deal_code".to_owned(),
            "name".to_owned(),
            "stage".to_owned(),
        ],
    )
    .await?;

    save_form(
        metadata_service,
        actor,
        "contact",
        "main_form",
        "Contact Profile",
        FormType::Main,
        vec![FormTab::new(
            "profile",
            "Profile",
            0,
            true,
            vec![FormSection::new(
                "contact_info",
                "Contact Info",
                0,
                true,
                2,
                vec![
                    FormFieldPlacement::new("display_name", 0, 0, true, false, None, None)?,
                    FormFieldPlacement::new("email", 0, 1, true, false, None, None)?,
                    FormFieldPlacement::new("job_title", 0, 2, true, false, None, None)?,
                    FormFieldPlacement::new("account_id", 1, 0, true, false, None, None)?,
                    FormFieldPlacement::new("phone", 1, 1, true, false, None, None)?,
                ],
                Vec::new(),
            )?],
        )?],
        vec!["display_name".to_owned(), "email".to_owned()],
    )
    .await?;

    save_form(
        metadata_service,
        actor,
        "invoice",
        "main_form",
        "Invoice Form",
        FormType::Main,
        vec![FormTab::new(
            "billing",
            "Billing",
            0,
            true,
            vec![FormSection::new(
                "invoice_details",
                "Invoice Details",
                0,
                true,
                2,
                vec![
                    FormFieldPlacement::new("invoice_number", 0, 0, true, false, None, None)?,
                    FormFieldPlacement::new("status", 0, 1, true, false, None, None)?,
                    FormFieldPlacement::new("total_amount", 0, 2, true, false, None, None)?,
                    FormFieldPlacement::new("due_date", 1, 0, true, false, None, None)?,
                    FormFieldPlacement::new("account_id", 1, 1, true, false, None, None)?,
                    FormFieldPlacement::new("deal_id", 1, 2, true, false, None, None)?,
                ],
                Vec::new(),
            )?],
        )?],
        vec!["invoice_number".to_owned(), "status".to_owned()],
    )
    .await?;

    save_view(
        metadata_service,
        actor,
        "account",
        "main_view",
        "Account Directory",
        vec!["account_code", "name", "industry", "annual_revenue"],
        Some(ViewSort::new("name", SortDirection::Asc)?),
        true,
    )
    .await?;

    save_view(
        metadata_service,
        actor,
        "contact",
        "main_view",
        "Contact Directory",
        vec!["display_name", "email", "account_id", "job_title", "phone"],
        Some(ViewSort::new("display_name", SortDirection::Asc)?),
        true,
    )
    .await?;

    save_view(
        metadata_service,
        actor,
        "deal",
        "main_view",
        "Pipeline",
        vec![
            "deal_code",
            "name",
            "stage",
            "amount",
            "close_date",
            "account_id",
        ],
        Some(ViewSort::new("amount", SortDirection::Desc)?),
        true,
    )
    .await?;

    save_view(
        metadata_service,
        actor,
        "deal",
        "open_pipeline",
        "Open Opportunities",
        vec!["name", "stage", "amount", "close_date"],
        Some(ViewSort::new("close_date", SortDirection::Asc)?),
        false,
    )
    .await?;

    save_view(
        metadata_service,
        actor,
        "invoice",
        "main_view",
        "Invoice Queue",
        vec![
            "invoice_number",
            "status",
            "total_amount",
            "due_date",
            "account_id",
        ],
        Some(ViewSort::new("due_date", SortDirection::Asc)?),
        true,
    )
    .await?;

    Ok(())
}

async fn save_form(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    logical_name: &str,
    display_name: &str,
    form_type: FormType,
    tabs: Vec<FormTab>,
    header_fields: Vec<String>,
) -> AppResult<()> {
    metadata_service
        .save_form(
            actor,
            SaveFormInput {
                entity_logical_name: entity_logical_name.to_owned(),
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                form_type,
                tabs,
                header_fields,
            },
        )
        .await?;

    Ok(())
}

async fn save_view(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    logical_name: &str,
    display_name: &str,
    columns: Vec<&str>,
    default_sort: Option<ViewSort>,
    prefer_default: bool,
) -> AppResult<()> {
    let has_other_default = metadata_service
        .list_views(actor, entity_logical_name)
        .await?
        .into_iter()
        .any(|existing| existing.is_default() && existing.logical_name().as_str() != logical_name);

    let view_columns = columns
        .into_iter()
        .enumerate()
        .map(|(position, field)| {
            ViewColumn::new(
                field,
                i32::try_from(position).map_err(|_| {
                    AppError::Validation("view column position exceeded i32 range".to_owned())
                })?,
                None,
                None,
            )
        })
        .collect::<AppResult<Vec<ViewColumn>>>()?;

    metadata_service
        .save_view(
            actor,
            SaveViewInput {
                entity_logical_name: entity_logical_name.to_owned(),
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                view_type: ViewType::Grid,
                columns: view_columns,
                default_sort,
                filter_criteria: None,
                is_default: prefer_default && !has_other_default,
            },
        )
        .await?;

    Ok(())
}

async fn ensure_entity(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    existing_entities: &mut HashSet<String>,
    logical_name: &str,
    display_name: &str,
    description: Option<String>,
    plural_display_name: Option<String>,
    icon: Option<String>,
) -> AppResult<()> {
    if existing_entities.contains(logical_name) {
        return Ok(());
    }

    metadata_service
        .register_entity_with_details(
            actor,
            logical_name,
            display_name,
            description,
            plural_display_name,
            icon,
        )
        .await?;
    existing_entities.insert(logical_name.to_owned());

    Ok(())
}

async fn save_text_field(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    logical_name: &str,
    display_name: &str,
    is_required: bool,
    is_unique: bool,
) -> AppResult<()> {
    metadata_service
        .save_field(
            actor,
            SaveFieldInput {
                entity_logical_name: entity_logical_name.to_owned(),
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                field_type: FieldType::Text,
                is_required,
                is_unique,
                default_value: None,
                relation_target_entity: None,
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await?;

    Ok(())
}

async fn save_number_field(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    logical_name: &str,
    display_name: &str,
    is_required: bool,
    is_unique: bool,
) -> AppResult<()> {
    metadata_service
        .save_field(
            actor,
            SaveFieldInput {
                entity_logical_name: entity_logical_name.to_owned(),
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                field_type: FieldType::Number,
                is_required,
                is_unique,
                default_value: None,
                relation_target_entity: None,
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await?;

    Ok(())
}

async fn save_date_field(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    logical_name: &str,
    display_name: &str,
    is_required: bool,
    is_unique: bool,
) -> AppResult<()> {
    metadata_service
        .save_field(
            actor,
            SaveFieldInput {
                entity_logical_name: entity_logical_name.to_owned(),
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                field_type: FieldType::Date,
                is_required,
                is_unique,
                default_value: None,
                relation_target_entity: None,
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await?;

    Ok(())
}

async fn save_relation_field(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    logical_name: &str,
    display_name: &str,
    relation_target_entity: &str,
    is_required: bool,
    is_unique: bool,
) -> AppResult<()> {
    metadata_service
        .save_field(
            actor,
            SaveFieldInput {
                entity_logical_name: entity_logical_name.to_owned(),
                logical_name: logical_name.to_owned(),
                display_name: display_name.to_owned(),
                field_type: FieldType::Relation,
                is_required,
                is_unique,
                default_value: None,
                relation_target_entity: Some(relation_target_entity.to_owned()),
                option_set_logical_name: None,
                calculation_expression: None,
            },
        )
        .await?;

    Ok(())
}

async fn publish_if_missing_fields(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    expected_fields: &[&str],
) -> AppResult<()> {
    let published_schema = metadata_service
        .latest_published_schema(actor, entity_logical_name)
        .await?;

    let needs_publish = published_schema
        .as_ref()
        .map(|schema| {
            expected_fields.iter().any(|expected| {
                !schema
                    .fields()
                    .iter()
                    .any(|field| field.logical_name().as_str() == *expected)
            })
        })
        .unwrap_or(true);

    if needs_publish {
        metadata_service
            .publish_entity(actor, entity_logical_name)
            .await?;
    }

    Ok(())
}

async fn upsert_runtime_record(
    metadata_service: &MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    unique_field: &str,
    unique_value: &str,
    payload: Value,
) -> AppResult<String> {
    let existing_records = metadata_service
        .list_runtime_records(
            actor,
            entity_logical_name,
            RecordListQuery {
                limit: 500,
                offset: 0,
                owner_subject: None,
            },
        )
        .await?;

    if let Some(record) = existing_records.into_iter().find(|record| {
        record
            .data()
            .get(unique_field)
            .and_then(Value::as_str)
            .map(|value| value == unique_value)
            .unwrap_or(false)
    }) {
        let updated = metadata_service
            .update_runtime_record(
                actor,
                entity_logical_name,
                record.record_id().as_str(),
                payload,
            )
            .await?;
        return Ok(updated.record_id().as_str().to_owned());
    }

    let created = metadata_service
        .create_runtime_record(actor, entity_logical_name, payload)
        .await?;
    Ok(created.record_id().as_str().to_owned())
}
