use qryvanta_application::AppRepository;
use qryvanta_core::TenantId;
use qryvanta_domain::{
    AppDefinition, AppEntityBinding, AppEntityForm, AppEntityView, AppEntityViewMode,
};
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresAppRepository;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn test_pool() -> Option<PgPool> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        return None;
    };

    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .connect(database_url.as_str())
        .await
    {
        Ok(pool) => pool,
        Err(error) => panic!("failed to connect to DATABASE_URL in test: {error}"),
    };

    if let Err(error) = MIGRATOR.run(&pool).await {
        panic!("failed to run migrations for postgres app repository tests: {error}");
    }

    Some(pool)
}

async fn ensure_tenant(pool: &PgPool, tenant_id: TenantId, name: &str) {
    let insert = sqlx::query(
        r#"
            INSERT INTO tenants (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id) DO NOTHING
            "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(name)
    .execute(pool)
    .await;

    assert!(insert.is_ok());
}

async fn ensure_entity(pool: &PgPool, tenant_id: TenantId, logical_name: &str, display_name: &str) {
    let insert = sqlx::query(
        r#"
            INSERT INTO entity_definitions (tenant_id, logical_name, display_name)
            VALUES ($1, $2, $3)
            ON CONFLICT (tenant_id, logical_name)
            DO UPDATE SET display_name = EXCLUDED.display_name
            "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(logical_name)
    .bind(display_name)
    .execute(pool)
    .await;

    assert!(insert.is_ok());
}

#[tokio::test]
async fn save_and_list_binding_round_trip_model_driven_surfaces() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresAppRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "App Repo Tenant").await;
    ensure_entity(&pool, tenant_id, "account", "Account").await;

    let create_app = repository
        .create_app(
            tenant_id,
            AppDefinition::new("sales", "Sales", None).unwrap_or_else(|_| unreachable!()),
        )
        .await;
    assert!(create_app.is_ok());

    let binding = AppEntityBinding::new(
        "sales",
        "account",
        Some("Accounts".to_owned()),
        10,
        vec![
            AppEntityForm::new(
                "main_form",
                "Main Form",
                vec!["name".to_owned(), "owner".to_owned()],
            )
            .unwrap_or_else(|_| unreachable!()),
            AppEntityForm::new("quick_form", "Quick Form", vec!["name".to_owned()])
                .unwrap_or_else(|_| unreachable!()),
        ],
        vec![
            AppEntityView::new(
                "main_view",
                "Main View",
                vec!["name".to_owned(), "status".to_owned()],
            )
            .unwrap_or_else(|_| unreachable!()),
            AppEntityView::new("compact_view", "Compact", vec!["name".to_owned()])
                .unwrap_or_else(|_| unreachable!()),
        ],
        "quick_form",
        "compact_view",
        AppEntityViewMode::Grid,
    )
    .unwrap_or_else(|_| unreachable!());

    let save = repository.save_app_entity_binding(tenant_id, binding).await;
    assert!(save.is_ok());

    let listed = repository
        .list_app_entity_bindings(tenant_id, "sales")
        .await;
    assert!(listed.is_ok());
    let listed = listed.unwrap_or_default();

    assert_eq!(listed.len(), 1);
    let listed_binding = &listed[0];
    assert_eq!(listed_binding.forms().len(), 2);
    assert_eq!(listed_binding.list_views().len(), 2);
    assert_eq!(
        listed_binding.default_form_logical_name().as_str(),
        "quick_form"
    );
    assert_eq!(
        listed_binding.default_list_view_logical_name().as_str(),
        "compact_view"
    );
    assert_eq!(listed_binding.navigation_order(), 10);
}

#[tokio::test]
async fn list_bindings_backfills_model_driven_surfaces_from_legacy_columns() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresAppRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Legacy Binding Tenant").await;
    ensure_entity(&pool, tenant_id, "contact", "Contact").await;

    let create_app = repository
        .create_app(
            tenant_id,
            AppDefinition::new("operations", "Operations", None).unwrap_or_else(|_| unreachable!()),
        )
        .await;
    assert!(create_app.is_ok());

    let insert = sqlx::query(
        r#"
            INSERT INTO app_entity_bindings (
                tenant_id,
                app_logical_name,
                entity_logical_name,
                navigation_label,
                navigation_order,
                form_field_logical_names,
                list_field_logical_names,
                default_view_mode
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
    )
    .bind(tenant_id.as_uuid())
    .bind("operations")
    .bind("contact")
    .bind(Some("Contacts".to_owned()))
    .bind(0_i32)
    .bind(vec!["name".to_owned(), "email".to_owned()])
    .bind(vec!["name".to_owned()])
    .bind("grid")
    .execute(&pool)
    .await;
    assert!(insert.is_ok());

    let listed = repository
        .list_app_entity_bindings(tenant_id, "operations")
        .await;
    assert!(listed.is_ok());
    let listed = listed.unwrap_or_default();

    assert_eq!(listed.len(), 1);
    let listed_binding = &listed[0];
    assert_eq!(listed_binding.forms().len(), 1);
    assert_eq!(listed_binding.list_views().len(), 1);
    assert_eq!(
        listed_binding.forms()[0].logical_name().as_str(),
        "main_form"
    );
    assert_eq!(
        listed_binding.default_form_logical_name().as_str(),
        "main_form"
    );
    assert_eq!(
        listed_binding.default_list_view_logical_name().as_str(),
        "main_view"
    );
    assert_eq!(
        listed_binding.forms()[0].field_logical_names(),
        ["name".to_owned(), "email".to_owned()]
    );
}
