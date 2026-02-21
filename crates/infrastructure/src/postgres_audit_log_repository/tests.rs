use qryvanta_application::{AuditLogQuery, AuditLogRepository};
use qryvanta_core::TenantId;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresAuditLogRepository;

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
        panic!("failed to run migrations for postgres audit log tests: {error}");
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
async fn export_and_purge_entries_follow_retention_window() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresAuditLogRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Audit Tenant").await;

    let old_insert = sqlx::query(
        r#"
            INSERT INTO audit_log_entries (
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now() - interval '45 days')
            "#,
    )
    .bind(tenant_id.as_uuid())
    .bind("alice")
    .bind("runtime.record.created")
    .bind("runtime_record")
    .bind("record-old")
    .bind(Some("old entry"))
    .execute(&pool)
    .await;
    assert!(old_insert.is_ok());

    let recent_insert = sqlx::query(
        r#"
            INSERT INTO audit_log_entries (
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now() - interval '1 day')
            "#,
    )
    .bind(tenant_id.as_uuid())
    .bind("alice")
    .bind("runtime.record.updated")
    .bind("runtime_record")
    .bind("record-new")
    .bind(Some("recent entry"))
    .execute(&pool)
    .await;
    assert!(recent_insert.is_ok());

    let exported = repository
        .export_entries(
            tenant_id,
            AuditLogQuery {
                limit: 100,
                offset: 0,
                action: None,
                subject: Some("alice".to_owned()),
            },
        )
        .await;
    assert!(exported.is_ok());
    assert_eq!(exported.unwrap_or_default().len(), 2);

    let purged = repository.purge_entries_older_than(tenant_id, 30).await;
    assert!(purged.is_ok());
    assert_eq!(purged.unwrap_or(0), 1);

    let listed = repository
        .list_recent_entries(
            tenant_id,
            AuditLogQuery {
                limit: 100,
                offset: 0,
                action: None,
                subject: Some("alice".to_owned()),
            },
        )
        .await;
    assert!(listed.is_ok());
    let listed = listed.unwrap_or_default();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].resource_id, "record-new");
}
