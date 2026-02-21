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

    async fn save_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        input: SaveRuntimeFieldPermissionsInput,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        sqlx::query(
            r#"
            DELETE FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND subject = $2
              AND entity_logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.subject.as_str())
        .bind(input.entity_logical_name.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to clear runtime field permissions for subject '{}' and entity '{}': {error}",
                input.subject, input.entity_logical_name
            ))
        })?;

        for field in &input.fields {
            sqlx::query(
                r#"
                INSERT INTO runtime_subject_field_permissions (
                    tenant_id,
                    subject,
                    entity_logical_name,
                    field_logical_name,
                    can_read,
                    can_write
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (tenant_id, subject, entity_logical_name, field_logical_name)
                DO UPDATE
                SET can_read = EXCLUDED.can_read,
                    can_write = EXCLUDED.can_write,
                    updated_at = now()
                "#,
            )
            .bind(tenant_id.as_uuid())
            .bind(input.subject.as_str())
            .bind(input.entity_logical_name.as_str())
            .bind(field.field_logical_name.as_str())
            .bind(field.can_read)
            .bind(field.can_write)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to save runtime field permission for field '{}': {error}",
                    field.field_logical_name
                ))
            })?;
        }

        let rows = sqlx::query_as::<_, RuntimeFieldPermissionRow>(
            r#"
            SELECT
                subject,
                entity_logical_name,
                field_logical_name,
                can_read,
                can_write,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
            FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND subject = $2
              AND entity_logical_name = $3
            ORDER BY field_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.subject.as_str())
        .bind(input.entity_logical_name.as_str())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list saved runtime field permissions for subject '{}' and entity '{}': {error}",
                input.subject, input.entity_logical_name
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| RuntimeFieldPermissionEntry {
                subject: row.subject,
                entity_logical_name: row.entity_logical_name,
                field_logical_name: row.field_logical_name,
                can_read: row.can_read,
                can_write: row.can_write,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn list_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        subject: Option<&str>,
        entity_logical_name: Option<&str>,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>> {
        let rows = sqlx::query_as::<_, RuntimeFieldPermissionRow>(
            r#"
            SELECT
                subject,
                entity_logical_name,
                field_logical_name,
                can_read,
                can_write,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
            FROM runtime_subject_field_permissions
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR subject = $2)
              AND ($3::TEXT IS NULL OR entity_logical_name = $3)
            ORDER BY subject, entity_logical_name, field_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to list runtime field permissions: {error}"))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| RuntimeFieldPermissionEntry {
                subject: row.subject,
                entity_logical_name: row.entity_logical_name,
                field_logical_name: row.field_logical_name,
                can_read: row.can_read,
                can_write: row.can_write,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn create_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        created_by_subject: &str,
        input: CreateTemporaryAccessGrantInput,
    ) -> AppResult<TemporaryAccessGrant> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        let grant_row = sqlx::query_as::<_, TemporaryAccessGrantRow>(
            r#"
            INSERT INTO security_temporary_access_grants (
                tenant_id,
                subject,
                reason,
                created_by_subject,
                expires_at
            )
            VALUES ($1, $2, $3, $4, now() + make_interval(mins => $5::INTEGER))
            RETURNING
                id AS grant_id,
                subject,
                reason,
                created_by_subject,
                to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS expires_at,
                NULL::TEXT AS revoked_at,
                NULL::TEXT AS permission
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.subject.as_str())
        .bind(input.reason.as_str())
        .bind(created_by_subject)
        .bind(i32::try_from(input.duration_minutes).map_err(|_| {
            AppError::Validation(
                "temporary access duration_minutes exceeds supported range".to_owned(),
            )
        })?)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to create temporary access grant: {error}"))
        })?;

        for permission in &input.permissions {
            sqlx::query(
                r#"
                INSERT INTO security_temporary_access_grant_permissions (grant_id, permission)
                VALUES ($1, $2)
                ON CONFLICT (grant_id, permission) DO NOTHING
                "#,
            )
            .bind(grant_row.grant_id)
            .bind(permission.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to persist temporary access grant permissions: {error}"
                ))
            })?;
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!("failed to commit transaction: {error}"))
        })?;

        Ok(TemporaryAccessGrant {
            grant_id: grant_row.grant_id.to_string(),
            subject: input.subject,
            permissions: input.permissions,
            reason: input.reason,
            created_by_subject: grant_row.created_by_subject,
            expires_at: grant_row.expires_at,
            revoked_at: None,
        })
    }

    async fn revoke_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        revoked_by_subject: &str,
        grant_id: &str,
        revoke_reason: Option<&str>,
    ) -> AppResult<()> {
        let parsed_grant_id = uuid::Uuid::parse_str(grant_id)
            .map_err(|_| AppError::Validation(format!("invalid grant_id '{}'", grant_id)))?;

        let rows_affected = sqlx::query(
            r#"
            UPDATE security_temporary_access_grants
            SET revoked_at = now(),
                revoked_by_subject = $3,
                revoke_reason = $4
            WHERE tenant_id = $1
              AND id = $2
              AND revoked_at IS NULL
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(parsed_grant_id)
        .bind(revoked_by_subject)
        .bind(revoke_reason)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to revoke temporary access grant: {error}"))
        })?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "temporary access grant '{}' was not found or already revoked",
                grant_id
            )));
        }

        Ok(())
    }

    async fn list_temporary_access_grants(
        &self,
        tenant_id: TenantId,
        query: TemporaryAccessGrantQuery,
    ) -> AppResult<Vec<TemporaryAccessGrant>> {
        let capped_limit = query.limit.clamp(1, 200) as i64;
        let capped_offset = query.offset.min(5_000) as i64;

        let rows = sqlx::query_as::<_, TemporaryAccessGrantRow>(
            r#"
            SELECT
                grants.id AS grant_id,
                grants.subject,
                grants.reason,
                grants.created_by_subject,
                to_char(grants.expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS expires_at,
                CASE
                    WHEN grants.revoked_at IS NULL THEN NULL
                    ELSE to_char(grants.revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
                END AS revoked_at,
                permissions.permission
            FROM security_temporary_access_grants AS grants
            LEFT JOIN security_temporary_access_grant_permissions AS permissions
                ON permissions.grant_id = grants.id
            WHERE grants.tenant_id = $1
              AND ($2::TEXT IS NULL OR grants.subject = $2)
              AND (
                  $3::BOOLEAN = false
                  OR (grants.revoked_at IS NULL AND grants.expires_at > now())
              )
            ORDER BY grants.created_at DESC, permissions.permission
            LIMIT $4
            OFFSET $5
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(query.subject)
        .bind(query.active_only)
        .bind(capped_limit)
        .bind(capped_offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to list temporary access grants: {error}"))
        })?;

        aggregate_temporary_access_grants(rows, tenant_id)
    }

    async fn registration_mode(&self, tenant_id: TenantId) -> AppResult<RegistrationMode> {
        let stored_mode = sqlx::query_scalar::<_, String>(
            r#"
            SELECT registration_mode
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve tenant registration mode: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        RegistrationMode::parse(stored_mode.as_str()).map_err(|error| {
            AppError::Internal(format!(
                "invalid tenant registration mode '{}' for tenant '{}': {error}",
                stored_mode, tenant_id
            ))
        })
    }

    async fn set_registration_mode(
        &self,
        tenant_id: TenantId,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode> {
        let stored_mode = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE tenants
            SET registration_mode = $2
            WHERE id = $1
            RETURNING registration_mode
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(registration_mode.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to update tenant registration mode: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        RegistrationMode::parse(stored_mode.as_str()).map_err(|error| {
            AppError::Internal(format!(
                "invalid tenant registration mode '{}' for tenant '{}': {error}",
                stored_mode, tenant_id
            ))
        })
    }

    async fn audit_retention_policy(&self, tenant_id: TenantId) -> AppResult<AuditRetentionPolicy> {
        let retention_days = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT audit_retention_days
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve tenant audit retention policy: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        Ok(AuditRetentionPolicy {
            retention_days: u16::try_from(retention_days).map_err(|_| {
                AppError::Internal(format!(
                    "invalid stored audit retention_days '{}' for tenant '{}'",
                    retention_days, tenant_id
                ))
            })?,
        })
    }

    async fn set_audit_retention_policy(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<AuditRetentionPolicy> {
        let stored_days = sqlx::query_scalar::<_, i32>(
            r#"
            UPDATE tenants
            SET audit_retention_days = $2
            WHERE id = $1
            RETURNING audit_retention_days
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(i32::from(retention_days))
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to update tenant audit retention policy: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        Ok(AuditRetentionPolicy {
            retention_days: u16::try_from(stored_days).map_err(|_| {
                AppError::Internal(format!(
                    "invalid stored audit retention_days '{}' for tenant '{}'",
                    stored_days, tenant_id
                ))
            })?,
        })
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
