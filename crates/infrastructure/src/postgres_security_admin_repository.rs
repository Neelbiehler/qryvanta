use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

use qryvanta_application::{
    CreateRoleInput, RoleAssignment, RoleDefinition, SecurityAdminRepository,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::Permission;

/// PostgreSQL-backed repository for role administration.
#[derive(Clone)]
pub struct PostgresSecurityAdminRepository {
    pool: PgPool,
}

impl PostgresSecurityAdminRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct RoleRow {
    role_id: uuid::Uuid,
    role_name: String,
    is_system: bool,
    permission: Option<String>,
}

#[derive(Debug, FromRow)]
struct RoleAssignmentRow {
    subject: String,
    role_id: uuid::Uuid,
    role_name: String,
    assigned_at: String,
}

#[async_trait]
impl SecurityAdminRepository for PostgresSecurityAdminRepository {
    async fn list_roles(&self, tenant_id: TenantId) -> AppResult<Vec<RoleDefinition>> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT
                roles.id AS role_id,
                roles.name AS role_name,
                roles.is_system,
                grants.permission
            FROM rbac_roles AS roles
            LEFT JOIN rbac_role_grants AS grants
                ON grants.role_id = roles.id
            WHERE roles.tenant_id = $1
            ORDER BY roles.name, grants.permission
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to list roles: {error}")))?;

        aggregate_roles(rows, tenant_id)
    }

    async fn create_role(
        &self,
        tenant_id: TenantId,
        input: CreateRoleInput,
    ) -> AppResult<RoleDefinition> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        let role_id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            INSERT INTO rbac_roles (tenant_id, name, is_system)
            VALUES ($1, $2, false)
            RETURNING id
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.name.trim())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| map_role_conflict(error, input.name.as_str()))?;

        for permission in &input.permissions {
            sqlx::query(
                r#"
                INSERT INTO rbac_role_grants (role_id, permission)
                VALUES ($1, $2)
                ON CONFLICT (role_id, permission) DO NOTHING
                "#,
            )
            .bind(role_id)
            .bind(permission.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                AppError::Internal(format!("failed to persist role grants: {error}"))
            })?;
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        Ok(RoleDefinition {
            role_id: role_id.to_string(),
            name: input.name,
            is_system: false,
            permissions: input.permissions,
        })
    }

    async fn assign_role_to_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        let role_id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            SELECT id
            FROM rbac_roles
            WHERE tenant_id = $1 AND name = $2
            LIMIT 1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(role_name)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to resolve role: {error}")))?
        .ok_or_else(|| AppError::NotFound(format!("role '{role_name}' was not found")))?;

        let membership_exists = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM tenant_memberships
            WHERE tenant_id = $1
                AND subject = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to resolve membership: {error}")))?;

        if membership_exists == 0 {
            return Err(AppError::NotFound(format!(
                "subject '{subject}' does not belong to tenant '{tenant_id}'"
            )));
        }

        sqlx::query(
            r#"
            INSERT INTO rbac_subject_roles (tenant_id, subject, role_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (tenant_id, subject, role_id) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(role_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to assign role: {error}")))?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        Ok(())
    }

    async fn remove_role_from_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM rbac_subject_roles AS subject_roles
            USING rbac_roles AS roles
            WHERE subject_roles.role_id = roles.id
                AND subject_roles.tenant_id = $1
                AND subject_roles.subject = $2
                AND roles.name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(role_name)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to remove role assignment: {error}")))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "role assignment '{subject}:{role_name}' was not found"
            )));
        }

        Ok(())
    }

    async fn list_role_assignments(&self, tenant_id: TenantId) -> AppResult<Vec<RoleAssignment>> {
        let rows = sqlx::query_as::<_, RoleAssignmentRow>(
            r#"
            SELECT
                subject_roles.subject,
                subject_roles.role_id,
                roles.name AS role_name,
                to_char(subject_roles.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS assigned_at
            FROM rbac_subject_roles AS subject_roles
            INNER JOIN rbac_roles AS roles
                ON roles.id = subject_roles.role_id
            WHERE subject_roles.tenant_id = $1
            ORDER BY subject_roles.subject, roles.name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to list role assignments: {error}")))?;

        Ok(rows
            .into_iter()
            .map(|row| RoleAssignment {
                subject: row.subject,
                role_id: row.role_id.to_string(),
                role_name: row.role_name,
                assigned_at: row.assigned_at,
            })
            .collect())
    }
}

fn aggregate_roles(rows: Vec<RoleRow>, tenant_id: TenantId) -> AppResult<Vec<RoleDefinition>> {
    let mut by_id: HashMap<uuid::Uuid, RoleDefinition> = HashMap::new();

    for row in rows {
        let role = by_id.entry(row.role_id).or_insert_with(|| RoleDefinition {
            role_id: row.role_id.to_string(),
            name: row.role_name.clone(),
            is_system: row.is_system,
            permissions: Vec::new(),
        });

        if let Some(permission_value) = row.permission {
            let permission = Permission::from_str(permission_value.as_str()).map_err(|error| {
                AppError::Internal(format!(
                    "invalid stored permission '{}' for tenant '{}': {error}",
                    permission_value, tenant_id
                ))
            })?;

            role.permissions.push(permission);
        }
    }

    let mut roles = by_id.into_values().collect::<Vec<_>>();
    roles.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(roles)
}

fn map_role_conflict(error: sqlx::Error, role_name: &str) -> AppError {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.code().as_deref() == Some("23505")
    {
        return AppError::Conflict(format!("role '{role_name}' already exists"));
    }

    AppError::Internal(format!("failed to create role: {error}"))
}

/// Ensures the system owner role has full baseline grants.
pub async fn assign_owner_role_grants(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_id: TenantId,
    subject: &str,
) -> AppResult<()> {
    let role_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO rbac_roles (tenant_id, name, is_system)
        VALUES ($1, $2, true)
        ON CONFLICT (tenant_id, name) DO UPDATE
        SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind("tenant_owner")
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| AppError::Internal(format!("failed to ensure tenant owner role: {error}")))?;

    for permission in Permission::all() {
        sqlx::query(
            r#"
            INSERT INTO rbac_role_grants (role_id, permission)
            VALUES ($1, $2)
            ON CONFLICT (role_id, permission) DO NOTHING
            "#,
        )
        .bind(role_id)
        .bind(permission.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to ensure role grant: {error}")))?;
    }

    sqlx::query(
        r#"
        INSERT INTO rbac_subject_roles (tenant_id, subject, role_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (tenant_id, subject, role_id) DO NOTHING
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(subject)
    .bind(role_id)
    .execute(&mut **transaction)
    .await
    .map_err(|error| AppError::Internal(format!("failed to assign subject role: {error}")))?;

    Ok(())
}
