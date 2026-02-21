use std::str::FromStr;

use async_trait::async_trait;

use qryvanta_application::{AuthorizationRepository, RuntimeFieldGrant, TemporaryPermissionGrant};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::Permission;

use sqlx::{FromRow, PgPool};

/// PostgreSQL-backed repository for subject permission lookups.
#[derive(Clone)]
pub struct PostgresAuthorizationRepository {
    pool: PgPool,
}

impl PostgresAuthorizationRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct PermissionRow {
    permission: String,
}

#[derive(Debug, FromRow)]
struct RuntimeFieldGrantRow {
    field_logical_name: String,
    can_read: bool,
    can_write: bool,
}

#[derive(Debug, FromRow)]
struct TemporaryPermissionGrantRow {
    grant_id: uuid::Uuid,
    reason: String,
    expires_at: String,
}

#[async_trait]
impl AuthorizationRepository for PostgresAuthorizationRepository {
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT DISTINCT grants.permission
            FROM rbac_subject_roles AS subject_roles
            INNER JOIN rbac_role_grants AS grants
                ON grants.role_id = subject_roles.role_id
            WHERE subject_roles.tenant_id = $1
                AND subject_roles.subject = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to load permissions: {error}")))?;

        rows.into_iter()
            .map(|row| {
                Permission::from_str(row.permission.as_str()).map_err(|error| {
                    AppError::Internal(format!(
                        "failed to decode permission '{}' for tenant '{}': {error}",
                        row.permission, tenant_id
                    ))
                })
            })
            .collect()
    }

    async fn list_runtime_field_grants_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        let rows = sqlx::query_as::<_, RuntimeFieldGrantRow>(
            r#"
            SELECT field_logical_name, can_read, can_write
            FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND subject = $2
              AND entity_logical_name = $3
            ORDER BY field_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load runtime field grants for subject '{}' in tenant '{}': {error}",
                subject, tenant_id
            ))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| RuntimeFieldGrant {
                field_logical_name: row.field_logical_name,
                can_read: row.can_read,
                can_write: row.can_write,
            })
            .collect())
    }

    async fn find_active_temporary_permission_grant(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>> {
        let row = sqlx::query_as::<_, TemporaryPermissionGrantRow>(
            r#"
            SELECT
                grants.id AS grant_id,
                grants.reason,
                to_char(grants.expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS expires_at
            FROM security_temporary_access_grants AS grants
            INNER JOIN security_temporary_access_grant_permissions AS permissions
                ON permissions.grant_id = grants.id
            WHERE grants.tenant_id = $1
              AND grants.subject = $2
              AND permissions.permission = $3
              AND grants.revoked_at IS NULL
              AND grants.expires_at > now()
            ORDER BY grants.expires_at DESC
            LIMIT 1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(permission.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve temporary permission grant for subject '{}' in tenant '{}': {error}",
                subject, tenant_id
            ))
        })?;

        Ok(row.map(|row| TemporaryPermissionGrant {
            grant_id: row.grant_id.to_string(),
            reason: row.reason,
            expires_at: row.expires_at,
        }))
    }
}
