use super::*;

impl PostgresAppRepository {
    pub(super) async fn save_app_role_entity_permission_impl(
        &self,
        tenant_id: TenantId,
        permission: AppEntityRolePermission,
    ) -> AppResult<()> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start app role permission transaction for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        let role_id = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            SELECT id
            FROM rbac_roles
            WHERE tenant_id = $1 AND name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(permission.role_name().as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to look up role '{}' for tenant '{}': {error}",
                permission.role_name().as_str(),
                tenant_id
            ))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "role '{}' does not exist for tenant '{}'",
                permission.role_name().as_str(),
                tenant_id
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO app_role_bindings (tenant_id, app_logical_name, role_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (tenant_id, app_logical_name, role_id) DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(permission.app_logical_name().as_str())
        .bind(role_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save app role binding for app '{}' role '{}' in tenant '{}': {error}",
                permission.app_logical_name().as_str(),
                permission.role_name().as_str(),
                tenant_id
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO app_role_entity_permissions (
                tenant_id,
                app_logical_name,
                role_id,
                entity_logical_name,
                can_read,
                can_create,
                can_update,
                can_delete,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            ON CONFLICT (tenant_id, app_logical_name, role_id, entity_logical_name)
            DO UPDATE SET
                can_read = EXCLUDED.can_read,
                can_create = EXCLUDED.can_create,
                can_update = EXCLUDED.can_update,
                can_delete = EXCLUDED.can_delete,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(permission.app_logical_name().as_str())
        .bind(role_id)
        .bind(permission.entity_logical_name().as_str())
        .bind(permission.can_read())
        .bind(permission.can_create())
        .bind(permission.can_update())
        .bind(permission.can_delete())
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save app role entity permissions for app '{}' role '{}' entity '{}' in tenant '{}': {error}",
                permission.app_logical_name().as_str(),
                permission.role_name().as_str(),
                permission.entity_logical_name().as_str(),
                tenant_id
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit app role permission transaction for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_app_role_entity_permissions_impl(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityRolePermission>> {
        let rows = sqlx::query_as::<_, AppRoleEntityPermissionRow>(
            r#"
            SELECT
                p.app_logical_name,
                r.name AS role_name,
                p.entity_logical_name,
                p.can_read,
                p.can_create,
                p.can_update,
                p.can_delete
            FROM app_role_entity_permissions p
            INNER JOIN rbac_roles r
                ON r.id = p.role_id
            WHERE p.tenant_id = $1 AND p.app_logical_name = $2
            ORDER BY r.name, p.entity_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list app role entity permissions for app '{}' in tenant '{}': {error}",
                app_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                AppEntityRolePermission::new(
                    row.app_logical_name,
                    row.role_name,
                    row.entity_logical_name,
                    row.can_read,
                    row.can_create,
                    row.can_update,
                    row.can_delete,
                )
            })
            .collect()
    }

    pub(super) async fn list_accessible_apps_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<AppDefinition>> {
        let rows = sqlx::query_as::<_, AppRow>(
            r#"
            SELECT DISTINCT app.logical_name, app.display_name, app.description
            FROM app_definitions app
            INNER JOIN app_role_bindings app_role
                ON app_role.tenant_id = app.tenant_id
                AND app_role.app_logical_name = app.logical_name
            INNER JOIN rbac_subject_roles subject_roles
                ON subject_roles.role_id = app_role.role_id
                AND subject_roles.tenant_id = app.tenant_id
            WHERE app.tenant_id = $1 AND subject_roles.subject = $2
            ORDER BY app.display_name, app.logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list accessible apps for subject '{}' in tenant '{}': {error}",
                subject, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| AppDefinition::new(row.logical_name, row.display_name, row.description))
            .collect()
    }

    pub(super) async fn subject_can_access_app_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<bool> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM app_role_bindings app_role
                INNER JOIN rbac_subject_roles subject_roles
                    ON subject_roles.role_id = app_role.role_id
                    AND subject_roles.tenant_id = app_role.tenant_id
                WHERE app_role.tenant_id = $1
                  AND app_role.app_logical_name = $2
                  AND subject_roles.subject = $3
            )
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .bind(subject)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to evaluate app access for subject '{}' app '{}' tenant '{}': {error}",
                subject, app_logical_name, tenant_id
            ))
        })
    }

    pub(super) async fn subject_entity_permission_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<Option<SubjectEntityPermission>> {
        let row = sqlx::query_as::<_, SubjectEntityPermissionSummaryRow>(
            r#"
            SELECT
                COUNT(*)::BIGINT AS row_count,
                COALESCE(bool_or(p.can_read), false) AS can_read,
                COALESCE(bool_or(p.can_create), false) AS can_create,
                COALESCE(bool_or(p.can_update), false) AS can_update,
                COALESCE(bool_or(p.can_delete), false) AS can_delete
            FROM app_role_entity_permissions p
            INNER JOIN rbac_subject_roles subject_roles
                ON subject_roles.role_id = p.role_id
                AND subject_roles.tenant_id = p.tenant_id
            WHERE p.tenant_id = $1
              AND p.app_logical_name = $2
              AND p.entity_logical_name = $3
              AND subject_roles.subject = $4
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .bind(entity_logical_name)
        .bind(subject)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to evaluate entity permissions for subject '{}' app '{}' entity '{}' tenant '{}': {error}",
                subject, app_logical_name, entity_logical_name, tenant_id
            ))
        })?;

        if row.row_count == 0 {
            return Ok(None);
        }

        Ok(Some(SubjectEntityPermission {
            entity_logical_name: entity_logical_name.to_owned(),
            can_read: row.can_read,
            can_create: row.can_create,
            can_update: row.can_update,
            can_delete: row.can_delete,
        }))
    }

    pub(super) async fn list_subject_entity_permissions_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<Vec<SubjectEntityPermission>> {
        let rows = sqlx::query_as::<_, SubjectEntityPermissionRow>(
            r#"
            SELECT
                p.entity_logical_name,
                COALESCE(bool_or(p.can_read), false) AS can_read,
                COALESCE(bool_or(p.can_create), false) AS can_create,
                COALESCE(bool_or(p.can_update), false) AS can_update,
                COALESCE(bool_or(p.can_delete), false) AS can_delete
            FROM app_role_entity_permissions p
            INNER JOIN rbac_subject_roles subject_roles
                ON subject_roles.role_id = p.role_id
                AND subject_roles.tenant_id = p.tenant_id
            WHERE p.tenant_id = $1
              AND p.app_logical_name = $2
              AND subject_roles.subject = $3
            GROUP BY p.entity_logical_name
            ORDER BY p.entity_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .bind(subject)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list entity permissions for subject '{}' app '{}' tenant '{}': {error}",
                subject, app_logical_name, tenant_id
            ))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| SubjectEntityPermission {
                entity_logical_name: row.entity_logical_name,
                can_read: row.can_read,
                can_create: row.can_create,
                can_update: row.can_update,
                can_delete: row.can_delete,
            })
            .collect())
    }
}
