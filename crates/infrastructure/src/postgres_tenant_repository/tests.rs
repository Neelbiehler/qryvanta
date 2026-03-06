use qryvanta_application::TenantRepository;
use qryvanta_core::TenantId;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresTenantRepository;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

fn unique_subject(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

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
        panic!("failed to run migrations for postgres tenant tests: {error}");
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

#[tokio::test]
async fn ensure_membership_bootstraps_and_reads_via_subject_lookup_scope() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresTenantRepository::new(pool.clone());
    let preferred_tenant_id = TenantId::new();
    let subject = unique_subject("bootstrap-subject");

    let tenant_id = repository
        .ensure_membership_for_subject(
            subject.as_str(),
            "Bootstrap Subject",
            Some("bootstrap@example.com"),
            Some(preferred_tenant_id),
        )
        .await;
    assert!(tenant_id.is_ok());
    let tenant_id = tenant_id.unwrap_or_default();
    assert_eq!(tenant_id, preferred_tenant_id);

    let resolved = repository.find_tenant_for_subject(subject.as_str()).await;
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap_or_default(), Some(preferred_tenant_id));
}

#[tokio::test]
async fn create_membership_persists_under_tenant_scope() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresTenantRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    let subject = unique_subject("tenant-scoped-subject");
    ensure_tenant(&pool, tenant_id, "Tenant Scope Membership").await;

    let created = repository
        .create_membership(
            tenant_id,
            subject.as_str(),
            "Tenant Scoped Subject",
            Some("tenant-scoped@example.com"),
        )
        .await;
    assert!(created.is_ok());

    let resolved = repository.find_tenant_for_subject(subject.as_str()).await;
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap_or_default(), Some(tenant_id));
}
