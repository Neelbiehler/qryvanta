use super::*;

impl PostgresSecurityAdminRepository {
    pub(super) async fn list_roles_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<Vec<RoleDefinition>> {
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

    pub(super) async fn create_role_impl(
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

    pub(super) async fn assign_role_to_subject_impl(
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

    pub(super) async fn remove_role_from_subject_impl(
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

    pub(super) async fn list_role_assignments_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<Vec<RoleAssignment>> {
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
