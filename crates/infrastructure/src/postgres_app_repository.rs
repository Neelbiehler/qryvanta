use async_trait::async_trait;

use qryvanta_application::{AppRepository, SubjectEntityPermission};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    AppDefinition, AppEntityBinding, AppEntityForm, AppEntityRolePermission, AppEntityView,
    AppEntityViewMode, AppSitemap,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};

/// PostgreSQL-backed repository for app definitions and app-scoped permissions.
#[derive(Clone)]
pub struct PostgresAppRepository {
    pool: PgPool,
}

impl PostgresAppRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct AppRow {
    logical_name: String,
    display_name: String,
    description: Option<String>,
}

#[derive(Debug, FromRow)]
struct AppEntityBindingRow {
    app_logical_name: String,
    entity_logical_name: String,
    navigation_label: Option<String>,
    navigation_order: i32,
    forms: Json<Vec<AppEntityFormDocument>>,
    list_views: Json<Vec<AppEntityViewDocument>>,
    default_form_logical_name: String,
    default_list_view_logical_name: String,
    default_view_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppEntityFormDocument {
    logical_name: String,
    display_name: String,
    field_logical_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppEntityViewDocument {
    logical_name: String,
    display_name: String,
    field_logical_names: Vec<String>,
}

#[derive(Debug, FromRow)]
struct AppRoleEntityPermissionRow {
    app_logical_name: String,
    role_name: String,
    entity_logical_name: String,
    can_read: bool,
    can_create: bool,
    can_update: bool,
    can_delete: bool,
}

#[derive(Debug, FromRow)]
struct SubjectEntityPermissionSummaryRow {
    row_count: i64,
    can_read: bool,
    can_create: bool,
    can_update: bool,
    can_delete: bool,
}

#[derive(Debug, FromRow)]
struct SubjectEntityPermissionRow {
    entity_logical_name: String,
    can_read: bool,
    can_create: bool,
    can_update: bool,
    can_delete: bool,
}

#[derive(Debug, FromRow)]
struct AppSitemapRow {
    definition_json: serde_json::Value,
}

#[async_trait]
impl AppRepository for PostgresAppRepository {
    async fn create_app(&self, tenant_id: TenantId, app: AppDefinition) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO app_definitions (tenant_id, logical_name, display_name, description)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app.logical_name().as_str())
        .bind(app.display_name().as_str())
        .bind(app.description())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                if let sqlx::Error::Database(database_error) = &error
                    && database_error.code().as_deref() == Some("23505")
                {
                    return Err(AppError::Conflict(format!(
                        "app '{}' already exists for tenant '{}'",
                        app.logical_name().as_str(),
                        tenant_id
                    )));
                }

                Err(AppError::Internal(format!(
                    "failed to create app '{}' for tenant '{}': {error}",
                    app.logical_name().as_str(),
                    tenant_id
                )))
            }
        }
    }

    async fn list_apps(&self, tenant_id: TenantId) -> AppResult<Vec<AppDefinition>> {
        let rows = sqlx::query_as::<_, AppRow>(
            r#"
            SELECT logical_name, display_name, description
            FROM app_definitions
            WHERE tenant_id = $1
            ORDER BY display_name, logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list apps for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| AppDefinition::new(row.logical_name, row.display_name, row.description))
            .collect()
    }

    async fn find_app(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppDefinition>> {
        let row = sqlx::query_as::<_, AppRow>(
            r#"
            SELECT logical_name, display_name, description
            FROM app_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find app '{}' for tenant '{}': {error}",
                app_logical_name, tenant_id
            ))
        })?;

        row.map(|value| {
            AppDefinition::new(value.logical_name, value.display_name, value.description)
        })
        .transpose()
    }

    async fn save_app_entity_binding(
        &self,
        tenant_id: TenantId,
        binding: AppEntityBinding,
    ) -> AppResult<()> {
        let forms = Json(
            binding
                .forms()
                .iter()
                .map(|form| AppEntityFormDocument {
                    logical_name: form.logical_name().as_str().to_owned(),
                    display_name: form.display_name().as_str().to_owned(),
                    field_logical_names: form.field_logical_names().to_vec(),
                })
                .collect::<Vec<_>>(),
        );
        let list_views = Json(
            binding
                .list_views()
                .iter()
                .map(|view| AppEntityViewDocument {
                    logical_name: view.logical_name().as_str().to_owned(),
                    display_name: view.display_name().as_str().to_owned(),
                    field_logical_names: view.field_logical_names().to_vec(),
                })
                .collect::<Vec<_>>(),
        );
        let default_view_mode = binding.default_view_mode().as_str();

        sqlx::query(
            r#"
            INSERT INTO app_entity_bindings (
                tenant_id,
                app_logical_name,
                entity_logical_name,
                navigation_label,
                navigation_order,
                forms,
                list_views,
                default_form_logical_name,
                default_list_view_logical_name,
                default_view_mode,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (tenant_id, app_logical_name, entity_logical_name)
            DO UPDATE SET
                navigation_label = EXCLUDED.navigation_label,
                navigation_order = EXCLUDED.navigation_order,
                forms = EXCLUDED.forms,
                list_views = EXCLUDED.list_views,
                default_form_logical_name = EXCLUDED.default_form_logical_name,
                default_list_view_logical_name = EXCLUDED.default_list_view_logical_name,
                default_view_mode = EXCLUDED.default_view_mode,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(binding.app_logical_name().as_str())
        .bind(binding.entity_logical_name().as_str())
        .bind(binding.navigation_label())
        .bind(binding.navigation_order())
        .bind(forms)
        .bind(list_views)
        .bind(binding.default_form_logical_name().as_str())
        .bind(binding.default_list_view_logical_name().as_str())
        .bind(default_view_mode)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save app entity binding '{}.{}' for tenant '{}': {error}",
                binding.app_logical_name().as_str(),
                binding.entity_logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    async fn list_app_entity_bindings(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>> {
        let rows = sqlx::query_as::<_, AppEntityBindingRow>(
            r#"
            SELECT
                app_logical_name,
                entity_logical_name,
                navigation_label,
                navigation_order,
                COALESCE(
                    NULLIF(forms, '[]'::jsonb),
                    jsonb_build_array(
                        jsonb_build_object(
                            'logical_name', 'main_form',
                            'display_name', 'Main Form',
                            'field_logical_names', form_field_logical_names
                        )
                    )
                ) AS forms,
                COALESCE(
                    NULLIF(list_views, '[]'::jsonb),
                    jsonb_build_array(
                        jsonb_build_object(
                            'logical_name', 'main_view',
                            'display_name', 'Main View',
                            'field_logical_names', list_field_logical_names
                        )
                    )
                ) AS list_views,
                COALESCE(default_form_logical_name, 'main_form') AS default_form_logical_name,
                COALESCE(default_list_view_logical_name, 'main_view') AS default_list_view_logical_name,
                default_view_mode
            FROM app_entity_bindings
            WHERE tenant_id = $1 AND app_logical_name = $2
            ORDER BY navigation_order, entity_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list app entity bindings for app '{}' and tenant '{}': {error}",
                app_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                AppEntityBinding::new(
                    row.app_logical_name,
                    row.entity_logical_name,
                    row.navigation_label,
                    row.navigation_order,
                    row.forms
                        .0
                        .into_iter()
                        .map(|form| {
                            AppEntityForm::new(
                                form.logical_name,
                                form.display_name,
                                form.field_logical_names,
                            )
                        })
                        .collect::<AppResult<Vec<_>>>()?,
                    row.list_views
                        .0
                        .into_iter()
                        .map(|view| {
                            AppEntityView::new(
                                view.logical_name,
                                view.display_name,
                                view.field_logical_names,
                            )
                        })
                        .collect::<AppResult<Vec<_>>>()?,
                    row.default_form_logical_name,
                    row.default_list_view_logical_name,
                    app_entity_view_mode_from_str(row.default_view_mode.as_str())?,
                )
            })
            .collect()
    }

    async fn save_sitemap(&self, tenant_id: TenantId, sitemap: AppSitemap) -> AppResult<()> {
        let definition_json = serde_json::to_value(&sitemap).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize sitemap for app '{}' in tenant '{}': {error}",
                sitemap.app_logical_name().as_str(),
                tenant_id
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO app_sitemaps (tenant_id, app_logical_name, definition_json, updated_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (tenant_id, app_logical_name)
            DO UPDATE SET
                definition_json = EXCLUDED.definition_json,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(sitemap.app_logical_name().as_str())
        .bind(definition_json)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save sitemap for app '{}' in tenant '{}': {error}",
                sitemap.app_logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    async fn get_sitemap(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppSitemap>> {
        let row = sqlx::query_as::<_, AppSitemapRow>(
            r#"
            SELECT definition_json
            FROM app_sitemaps
            WHERE tenant_id = $1 AND app_logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load sitemap for app '{}' in tenant '{}': {error}",
                app_logical_name, tenant_id
            ))
        })?;

        row.map(|value| {
            serde_json::from_value::<AppSitemap>(value.definition_json).map_err(|error| {
                AppError::Internal(format!(
                    "persisted sitemap for app '{}' in tenant '{}' is invalid: {error}",
                    app_logical_name, tenant_id
                ))
            })
        })
        .transpose()
    }

    async fn save_app_role_entity_permission(
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

    async fn list_app_role_entity_permissions(
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

    async fn list_accessible_apps(
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

    async fn subject_can_access_app(
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

    async fn subject_entity_permission(
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

    async fn list_subject_entity_permissions(
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

fn app_entity_view_mode_from_str(value: &str) -> AppResult<AppEntityViewMode> {
    AppEntityViewMode::parse(value)
}

#[cfg(test)]
mod tests;
