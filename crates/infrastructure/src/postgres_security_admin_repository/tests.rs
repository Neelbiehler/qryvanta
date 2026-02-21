use qryvanta_application::{
    CreateTemporaryAccessGrantInput, SaveRuntimeFieldPermissionsInput, SecurityAdminRepository,
    TemporaryAccessGrantQuery,
};
use qryvanta_core::TenantId;
use qryvanta_domain::Permission;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresSecurityAdminRepository;

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
        panic!("failed to run migrations for postgres security admin tests: {error}");
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
async fn save_runtime_field_permissions_replaces_existing_entries() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresSecurityAdminRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Security Tenant").await;

    let first_save = repository
        .save_runtime_field_permissions(
            tenant_id,
            SaveRuntimeFieldPermissionsInput {
                subject: "alice".to_owned(),
                entity_logical_name: "contact".to_owned(),
                fields: vec![
                    qryvanta_application::RuntimeFieldPermissionInput {
                        field_logical_name: "email".to_owned(),
                        can_read: true,
                        can_write: false,
                    },
                    qryvanta_application::RuntimeFieldPermissionInput {
                        field_logical_name: "phone".to_owned(),
                        can_read: true,
                        can_write: false,
                    },
                ],
            },
        )
        .await;
    assert!(first_save.is_ok());
    assert_eq!(first_save.unwrap_or_default().len(), 2);

    let second_save = repository
        .save_runtime_field_permissions(
            tenant_id,
            SaveRuntimeFieldPermissionsInput {
                subject: "alice".to_owned(),
                entity_logical_name: "contact".to_owned(),
                fields: vec![qryvanta_application::RuntimeFieldPermissionInput {
                    field_logical_name: "email".to_owned(),
                    can_read: true,
                    can_write: true,
                }],
            },
        )
        .await;
    assert!(second_save.is_ok());
    let second_save = second_save.unwrap_or_default();
    assert_eq!(second_save.len(), 1);
    assert!(second_save[0].can_write);

    let listed = repository
        .list_runtime_field_permissions(tenant_id, Some("alice"), Some("contact"))
        .await;
    assert!(listed.is_ok());
    let listed = listed.unwrap_or_default();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].field_logical_name, "email");
}

#[tokio::test]
async fn temporary_access_grant_lifecycle_is_persisted() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresSecurityAdminRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Temporary Access Tenant").await;

    let created_grant = repository
        .create_temporary_access_grant(
            tenant_id,
            "admin",
            CreateTemporaryAccessGrantInput {
                subject: "alice".to_owned(),
                permissions: vec![
                    Permission::RuntimeRecordRead,
                    Permission::RuntimeRecordWriteOwn,
                ],
                reason: "incident response".to_owned(),
                duration_minutes: 30,
            },
        )
        .await;
    assert!(created_grant.is_ok());
    let created_grant = created_grant.unwrap_or_else(|_| unreachable!());
    assert_eq!(created_grant.subject, "alice");
    assert_eq!(created_grant.permissions.len(), 2);

    let listed_active = repository
        .list_temporary_access_grants(
            tenant_id,
            TemporaryAccessGrantQuery {
                subject: Some("alice".to_owned()),
                active_only: true,
                limit: 20,
                offset: 0,
            },
        )
        .await;
    assert!(listed_active.is_ok());
    let listed_active = listed_active.unwrap_or_default();
    assert_eq!(listed_active.len(), 1);
    assert_eq!(listed_active[0].grant_id, created_grant.grant_id);

    let revoked = repository
        .revoke_temporary_access_grant(
            tenant_id,
            "admin",
            created_grant.grant_id.as_str(),
            Some("completed"),
        )
        .await;
    assert!(revoked.is_ok());

    let listed_after_revoke = repository
        .list_temporary_access_grants(
            tenant_id,
            TemporaryAccessGrantQuery {
                subject: Some("alice".to_owned()),
                active_only: false,
                limit: 20,
                offset: 0,
            },
        )
        .await;
    assert!(listed_after_revoke.is_ok());
    let listed_after_revoke = listed_after_revoke.unwrap_or_default();
    assert_eq!(listed_after_revoke.len(), 1);
    assert!(listed_after_revoke[0].revoked_at.is_some());

    let listed_active_after_revoke = repository
        .list_temporary_access_grants(
            tenant_id,
            TemporaryAccessGrantQuery {
                subject: Some("alice".to_owned()),
                active_only: true,
                limit: 20,
                offset: 0,
            },
        )
        .await;
    assert!(listed_active_after_revoke.is_ok());
    assert!(listed_active_after_revoke.unwrap_or_default().is_empty());
}

#[tokio::test]
async fn audit_retention_policy_round_trip_succeeds() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresSecurityAdminRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Retention Tenant").await;

    let current_policy = repository.audit_retention_policy(tenant_id).await;
    assert!(current_policy.is_ok());
    assert_eq!(
        current_policy
            .unwrap_or_else(|_| unreachable!())
            .retention_days,
        365
    );

    let updated_policy = repository.set_audit_retention_policy(tenant_id, 90).await;
    assert!(updated_policy.is_ok());
    assert_eq!(
        updated_policy
            .unwrap_or_else(|_| unreachable!())
            .retention_days,
        90
    );

    let reloaded_policy = repository.audit_retention_policy(tenant_id).await;
    assert!(reloaded_policy.is_ok());
    assert_eq!(
        reloaded_policy
            .unwrap_or_else(|_| unreachable!())
            .retention_days,
        90
    );
}
