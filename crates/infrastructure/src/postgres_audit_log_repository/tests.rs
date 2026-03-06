use qryvanta_application::{AuditLogQuery, AuditLogRepository};
use qryvanta_core::TenantId;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresAuditLogRepository;
use crate::audit_chain::{AuditChainInput, compute_audit_entry_hash};

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

async fn insert_audit_entry(
    pool: &PgPool,
    tenant_id: TenantId,
    subject: &str,
    action: &str,
    resource_id: &str,
    detail: Option<&str>,
    created_at_sql: &str,
    chain_position: i64,
    previous_entry_hash: Option<&str>,
) -> String {
    let created_at_utc = sqlx::query_scalar::<_, String>(&format!(
        "SELECT to_char(({}) AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')",
        created_at_sql
    ))
    .fetch_one(pool)
    .await
    .unwrap_or_else(|error| panic!("failed to resolve created_at for test audit entry: {error}"));
    let entry_hash = compute_audit_entry_hash(&AuditChainInput {
        tenant_id,
        chain_position,
        previous_entry_hash,
        subject,
        action,
        resource_type: "runtime_record",
        resource_id,
        detail,
        created_at_utc: created_at_utc.as_str(),
    });
    let insert_sql = r#"
            INSERT INTO audit_log_entries (
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                created_at,
                chain_position,
                previous_entry_hash,
                entry_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, (#CREATED_AT#), $7, $8, $9)
            "#
    .replace("(#CREATED_AT#)", created_at_sql);

    let insert = sqlx::query(insert_sql.as_str())
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(action)
        .bind("runtime_record")
        .bind(resource_id)
        .bind(detail)
        .bind(chain_position)
        .bind(previous_entry_hash)
        .bind(entry_hash.as_str())
        .execute(pool)
        .await;

    assert!(insert.is_ok());
    entry_hash
}

#[tokio::test]
async fn export_and_purge_entries_follow_retention_window() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresAuditLogRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Audit Tenant").await;

    let old_hash = insert_audit_entry(
        &pool,
        tenant_id,
        "alice",
        "runtime.record.created",
        "record-old",
        Some("old entry"),
        "now() - interval '45 days'",
        1,
        None,
    )
    .await;
    let _recent_hash = insert_audit_entry(
        &pool,
        tenant_id,
        "alice",
        "runtime.record.updated",
        "record-new",
        Some("recent entry"),
        "now() - interval '1 day'",
        2,
        Some(old_hash.as_str()),
    )
    .await;

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
    assert_eq!(listed[0].chain_position, 2);
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

    let left_old_hash = insert_audit_entry(
        &pool,
        left_tenant,
        "alice",
        "runtime.record.created",
        "left-old",
        Some("left old entry"),
        "now() - interval '45 days'",
        1,
        None,
    )
    .await;
    let _left_recent_hash = insert_audit_entry(
        &pool,
        left_tenant,
        "alice",
        "runtime.record.updated",
        "left-new",
        Some("left recent entry"),
        "now() - interval '1 day'",
        2,
        Some(left_old_hash.as_str()),
    )
    .await;
    let _right_old_hash = insert_audit_entry(
        &pool,
        right_tenant,
        "alice",
        "runtime.record.created",
        "right-old",
        Some("right old entry"),
        "now() - interval '45 days'",
        1,
        None,
    )
    .await;

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

#[tokio::test]
async fn verify_integrity_reports_valid_and_tampered_chains() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresAuditLogRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Audit Integrity Tenant").await;

    let first_hash = insert_audit_entry(
        &pool,
        tenant_id,
        "alice",
        "runtime.record.created",
        "record-1",
        Some("first entry"),
        "TIMESTAMPTZ '2026-03-01T00:00:00Z'",
        1,
        None,
    )
    .await;
    let _second_hash = insert_audit_entry(
        &pool,
        tenant_id,
        "alice",
        "runtime.record.updated",
        "record-2",
        Some("second entry"),
        "TIMESTAMPTZ '2026-03-02T00:00:00Z'",
        2,
        Some(first_hash.as_str()),
    )
    .await;

    let valid = repository.verify_integrity(tenant_id).await;
    assert!(valid.is_ok());
    let valid = valid.unwrap_or_else(|_| unreachable!());
    assert!(valid.is_valid);
    assert_eq!(valid.verified_entries, 2);
    assert_eq!(valid.latest_chain_position, Some(2));
    assert!(valid.failures.is_empty());

    let tamper = sqlx::query(
        r#"
            UPDATE audit_log_entries
            SET detail = 'tampered entry'
            WHERE tenant_id = $1 AND chain_position = 2
            "#,
    )
    .bind(tenant_id.as_uuid())
    .execute(&pool)
    .await;
    assert!(tamper.is_ok());

    let invalid = repository.verify_integrity(tenant_id).await;
    assert!(invalid.is_ok());
    let invalid = invalid.unwrap_or_else(|_| unreachable!());
    assert!(!invalid.is_valid);
    assert_eq!(invalid.verified_entries, 2);
    assert!(
        invalid
            .failures
            .iter()
            .any(|failure| failure.contains("entry_hash mismatch"))
    );
}
