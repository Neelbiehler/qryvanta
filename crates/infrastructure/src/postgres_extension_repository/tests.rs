use qryvanta_application::ExtensionRepository;
use qryvanta_core::TenantId;
use qryvanta_domain::{
    ExtensionCapability, ExtensionIsolationPolicy, ExtensionManifest, ExtensionManifestInput,
    ExtensionRuntimeKind,
};
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresExtensionRepository;

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
        panic!("failed to run migrations for postgres extension repository tests: {error}");
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

fn sample_manifest(
    logical_name: &str,
    capabilities: Vec<ExtensionCapability>,
    allow_network: bool,
    allowed_hosts: Vec<String>,
) -> ExtensionManifest {
    ExtensionManifest::new(ExtensionManifestInput {
        logical_name: logical_name.to_owned(),
        display_name: logical_name.to_owned(),
        package_version: "1.0.0".to_owned(),
        runtime_api_version: "1.0".to_owned(),
        runtime_kind: ExtensionRuntimeKind::Wasm,
        requested_capabilities: capabilities,
        isolation_policy: ExtensionIsolationPolicy::new(
            256,
            5_000,
            64,
            allow_network,
            allowed_hosts,
        )
        .unwrap_or_else(|_| unreachable!()),
    })
    .unwrap_or_else(|_| unreachable!())
}

#[tokio::test]
async fn save_and_list_extensions_round_trip() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresExtensionRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Extension Repo Tenant").await;

    let definition = qryvanta_domain::ExtensionDefinition::new(
        sample_manifest(
            "audit_guard",
            vec![
                ExtensionCapability::RuntimeRecordRead,
                ExtensionCapability::OutboundHttp,
            ],
            true,
            vec!["api.example.com".to_owned()],
        ),
        "abc123",
    )
    .unwrap_or_else(|_| unreachable!())
    .publish();

    let saved = repository.save_extension(tenant_id, definition).await;
    assert!(saved.is_ok());

    let listed = repository.list_extensions(tenant_id).await;
    assert!(listed.is_ok());
    let listed = listed.unwrap_or_default();

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].manifest().logical_name().as_str(), "audit_guard");
    assert!(listed[0].is_published());
    assert_eq!(
        listed[0].manifest().requested_capabilities().len(),
        2,
        "capability list should round-trip"
    );
    assert_eq!(
        listed[0].manifest().isolation_policy().allowed_hosts(),
        ["api.example.com".to_owned()]
    );
}

#[tokio::test]
async fn extension_queries_are_tenant_scoped() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresExtensionRepository::new(pool.clone());
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();
    ensure_tenant(&pool, left_tenant, "Left Extension Tenant").await;
    ensure_tenant(&pool, right_tenant, "Right Extension Tenant").await;

    let left_saved = repository
        .save_extension(
            left_tenant,
            qryvanta_domain::ExtensionDefinition::new(
                sample_manifest(
                    "left_extension",
                    vec![ExtensionCapability::RuntimeRecordRead],
                    false,
                    Vec::new(),
                ),
                "left-sha",
            )
            .unwrap_or_else(|_| unreachable!()),
        )
        .await;
    assert!(left_saved.is_ok());

    let right_listed = repository.list_extensions(right_tenant).await;
    assert!(right_listed.is_ok());
    assert!(right_listed.unwrap_or_default().is_empty());

    let right_found = repository
        .find_extension(right_tenant, "left_extension")
        .await;
    assert!(right_found.is_ok());
    assert!(right_found.unwrap_or_default().is_none());
}
