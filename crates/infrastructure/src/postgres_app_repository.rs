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

mod bindings;
mod definitions;
mod permissions;
mod sitemap;

#[async_trait]
impl AppRepository for PostgresAppRepository {
    async fn create_app(&self, tenant_id: TenantId, app: AppDefinition) -> AppResult<()> {
        self.create_app_impl(tenant_id, app).await
    }

    async fn list_apps(&self, tenant_id: TenantId) -> AppResult<Vec<AppDefinition>> {
        self.list_apps_impl(tenant_id).await
    }

    async fn find_app(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppDefinition>> {
        self.find_app_impl(tenant_id, app_logical_name).await
    }

    async fn save_app_entity_binding(
        &self,
        tenant_id: TenantId,
        binding: AppEntityBinding,
    ) -> AppResult<()> {
        self.save_app_entity_binding_impl(tenant_id, binding).await
    }

    async fn list_app_entity_bindings(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>> {
        self.list_app_entity_bindings_impl(tenant_id, app_logical_name)
            .await
    }

    async fn save_sitemap(&self, tenant_id: TenantId, sitemap: AppSitemap) -> AppResult<()> {
        self.save_sitemap_impl(tenant_id, sitemap).await
    }

    async fn get_sitemap(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppSitemap>> {
        self.get_sitemap_impl(tenant_id, app_logical_name).await
    }

    async fn save_app_role_entity_permission(
        &self,
        tenant_id: TenantId,
        permission: AppEntityRolePermission,
    ) -> AppResult<()> {
        self.save_app_role_entity_permission_impl(tenant_id, permission)
            .await
    }

    async fn list_app_role_entity_permissions(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityRolePermission>> {
        self.list_app_role_entity_permissions_impl(tenant_id, app_logical_name)
            .await
    }

    async fn list_accessible_apps(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<AppDefinition>> {
        self.list_accessible_apps_impl(tenant_id, subject).await
    }

    async fn subject_can_access_app(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<bool> {
        self.subject_can_access_app_impl(tenant_id, subject, app_logical_name)
            .await
    }

    async fn subject_entity_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<Option<SubjectEntityPermission>> {
        self.subject_entity_permission_impl(
            tenant_id,
            subject,
            app_logical_name,
            entity_logical_name,
        )
        .await
    }

    async fn list_subject_entity_permissions(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<Vec<SubjectEntityPermission>> {
        self.list_subject_entity_permissions_impl(tenant_id, subject, app_logical_name)
            .await
    }
}

fn app_entity_view_mode_from_str(value: &str) -> AppResult<AppEntityViewMode> {
    AppEntityViewMode::parse(value)
}

#[cfg(test)]
mod tests;
