use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

use qryvanta_application::{
    AuditRetentionPolicy, CreateRoleInput, CreateTemporaryAccessGrantInput, RoleAssignment,
    RoleDefinition, RuntimeFieldPermissionEntry, SaveRuntimeFieldPermissionsInput,
    SecurityAdminRepository, TemporaryAccessGrant, TemporaryAccessGrantQuery,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{Permission, RegistrationMode};

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

#[derive(Debug, FromRow)]
struct RuntimeFieldPermissionRow {
    subject: String,
    entity_logical_name: String,
    field_logical_name: String,
    can_read: bool,
    can_write: bool,
    updated_at: String,
}

#[derive(Debug, FromRow)]
struct TemporaryAccessGrantRow {
    grant_id: uuid::Uuid,
    subject: String,
    reason: String,
    created_by_subject: String,
    expires_at: String,
    revoked_at: Option<String>,
    permission: Option<String>,
}

mod governance;
mod roles;
mod runtime_permissions;
mod temporary_access;

#[async_trait]
impl SecurityAdminRepository for PostgresSecurityAdminRepository {
    async fn list_roles(&self, tenant_id: TenantId) -> AppResult<Vec<RoleDefinition>> {
        self.list_roles_impl(tenant_id).await
    }

    async fn create_role(
        &self,
        tenant_id: TenantId,
        input: CreateRoleInput,
    ) -> AppResult<RoleDefinition> {
        self.create_role_impl(tenant_id, input).await
    }

    async fn assign_role_to_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()> {
        self.assign_role_to_subject_impl(tenant_id, subject, role_name)
            .await
    }

    async fn remove_role_from_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()> {
        self.remove_role_from_subject_impl(tenant_id, subject, role_name)
            .await
    }

    async fn list_role_assignments(&self, tenant_id: TenantId) -> AppResult<Vec<RoleAssignment>> {
        self.list_role_assignments_impl(tenant_id).await
    }

    async fn save_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        input: SaveRuntimeFieldPermissionsInput,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        self.save_runtime_field_permissions_impl(tenant_id, input)
            .await
    }

    async fn list_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        subject: Option<&str>,
        entity_logical_name: Option<&str>,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        self.list_runtime_field_permissions_impl(tenant_id, subject, entity_logical_name)
            .await
    }

    async fn create_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        created_by_subject: &str,
        input: CreateTemporaryAccessGrantInput,
    ) -> AppResult<TemporaryAccessGrant> {
        self.create_temporary_access_grant_impl(tenant_id, created_by_subject, input)
            .await
    }

    async fn revoke_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        revoked_by_subject: &str,
        grant_id: &str,
        revoke_reason: Option<&str>,
    ) -> AppResult<()> {
        self.revoke_temporary_access_grant_impl(
            tenant_id,
            revoked_by_subject,
            grant_id,
            revoke_reason,
        )
        .await
    }

    async fn list_temporary_access_grants(
        &self,
        tenant_id: TenantId,
        query: TemporaryAccessGrantQuery,
    ) -> AppResult<Vec<TemporaryAccessGrant>> {
        self.list_temporary_access_grants_impl(tenant_id, query)
            .await
    }

    async fn registration_mode(&self, tenant_id: TenantId) -> AppResult<RegistrationMode> {
        self.registration_mode_impl(tenant_id).await
    }

    async fn set_registration_mode(
        &self,
        tenant_id: TenantId,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode> {
        self.set_registration_mode_impl(tenant_id, registration_mode)
            .await
    }

    async fn audit_retention_policy(&self, tenant_id: TenantId) -> AppResult<AuditRetentionPolicy> {
        self.audit_retention_policy_impl(tenant_id).await
    }

    async fn set_audit_retention_policy(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<AuditRetentionPolicy> {
        self.set_audit_retention_policy_impl(tenant_id, retention_days)
            .await
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

fn aggregate_temporary_access_grants(
    rows: Vec<TemporaryAccessGrantRow>,
    tenant_id: TenantId,
) -> AppResult<Vec<TemporaryAccessGrant>> {
    let mut grants = HashMap::<uuid::Uuid, TemporaryAccessGrant>::new();
    let mut grant_order = Vec::<uuid::Uuid>::new();

    for row in rows {
        let grant_entry = grants.entry(row.grant_id).or_insert_with(|| {
            grant_order.push(row.grant_id);
            TemporaryAccessGrant {
                grant_id: row.grant_id.to_string(),
                subject: row.subject.clone(),
                permissions: Vec::new(),
                reason: row.reason.clone(),
                created_by_subject: row.created_by_subject.clone(),
                expires_at: row.expires_at.clone(),
                revoked_at: row.revoked_at.clone(),
            }
        });

        if let Some(permission_value) = row.permission {
            let permission = Permission::from_str(permission_value.as_str()).map_err(|error| {
                AppError::Internal(format!(
                    "invalid temporary access permission '{}' for tenant '{}': {error}",
                    permission_value, tenant_id
                ))
            })?;

            grant_entry.permissions.push(permission);
        }
    }

    Ok(grant_order
        .into_iter()
        .filter_map(|grant_id| grants.remove(&grant_id))
        .collect())
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

#[cfg(test)]
mod tests;
