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

#[tokio::test]
async fn audit_log_queries_and_purge_are_tenant_scoped() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresAuditLogRepository::new(pool.clone());
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();
    ensure_tenant(&pool, left_tenant, "Audit Left Tenant").await;
    ensure_tenant(&pool, right_tenant, "Audit Right Tenant").await;

    let insert_left_old = sqlx::query(
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
    .bind(left_tenant.as_uuid())
    .bind("alice")
    .bind("runtime.record.created")
    .bind("runtime_record")
    .bind("left-old")
    .bind(Some("left old entry"))
    .execute(&pool)
    .await;
    assert!(insert_left_old.is_ok());

    let insert_left_recent = sqlx::query(
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
    .bind(left_tenant.as_uuid())
    .bind("alice")
    .bind("runtime.record.updated")
    .bind("runtime_record")
    .bind("left-new")
    .bind(Some("left recent entry"))
    .execute(&pool)
    .await;
    assert!(insert_left_recent.is_ok());

    let insert_right_old = sqlx::query(
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
    .bind(right_tenant.as_uuid())
    .bind("alice")
    .bind("runtime.record.created")
    .bind("runtime_record")
    .bind("right-old")
    .bind(Some("right old entry"))
    .execute(&pool)
    .await;
    assert!(insert_right_old.is_ok());

    let listed_left = repository
        .list_recent_entries(
            left_tenant,
            AuditLogQuery {
                limit: 100,
                offset: 0,
                action: None,
                subject: Some("alice".to_owned()),
            },
        )
        .await;
    assert!(listed_left.is_ok());
    let listed_left = listed_left.unwrap_or_default();
    assert_eq!(listed_left.len(), 2);
    assert!(
        listed_left
            .iter()
            .all(|entry| entry.resource_id.starts_with("left-"))
    );

    let listed_right = repository
        .list_recent_entries(
            right_tenant,
            AuditLogQuery {
                limit: 100,
                offset: 0,
                action: None,
                subject: Some("alice".to_owned()),
            },
        )
        .await;
    assert!(listed_right.is_ok());
    let listed_right = listed_right.unwrap_or_default();
    assert_eq!(listed_right.len(), 1);
    assert_eq!(listed_right[0].resource_id, "right-old");

    let purged_left = repository.purge_entries_older_than(left_tenant, 30).await;
    assert!(purged_left.is_ok());
    assert_eq!(purged_left.unwrap_or(0), 1);

    let after_left_purge = repository
        .list_recent_entries(
            left_tenant,
            AuditLogQuery {
                limit: 100,
                offset: 0,
                action: None,
                subject: Some("alice".to_owned()),
            },
        )
        .await;
    assert!(after_left_purge.is_ok());
    let after_left_purge = after_left_purge.unwrap_or_default();
    assert_eq!(after_left_purge.len(), 1);
    assert_eq!(after_left_purge[0].resource_id, "left-new");

    let after_right_purge = repository
        .list_recent_entries(
            right_tenant,
            AuditLogQuery {
                limit: 100,
                offset: 0,
                action: None,
                subject: Some("alice".to_owned()),
            },
        )
        .await;
    assert!(after_right_purge.is_ok());
    let after_right_purge = after_right_purge.unwrap_or_default();
    assert_eq!(after_right_purge.len(), 1);
    assert_eq!(after_right_purge[0].resource_id, "right-old");
}
