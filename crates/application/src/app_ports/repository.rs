use async_trait::async_trait;

use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{AppDefinition, AppEntityBinding, AppEntityRolePermission, AppSitemap};

use super::permissions::SubjectEntityPermission;

/// Repository port for app definitions and app-scoped permissions.
#[async_trait]
pub trait AppRepository: Send + Sync {
    /// Creates a new app definition.
    async fn create_app(&self, tenant_id: TenantId, app: AppDefinition) -> AppResult<()>;

    /// Lists all apps for a tenant.
    async fn list_apps(&self, tenant_id: TenantId) -> AppResult<Vec<AppDefinition>>;

    /// Returns one app by logical name.
    async fn find_app(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppDefinition>>;

    /// Saves an app entity navigation binding.
    async fn save_app_entity_binding(
        &self,
        tenant_id: TenantId,
        binding: AppEntityBinding,
    ) -> AppResult<()>;

    /// Lists entities bound into an app navigation.
    async fn list_app_entity_bindings(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>>;

    /// Saves app sitemap definition.
    async fn save_sitemap(&self, tenant_id: TenantId, sitemap: AppSitemap) -> AppResult<()>;

    /// Returns app sitemap definition when configured.
    async fn get_sitemap(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppSitemap>>;

    /// Saves app-scoped role permissions for an entity.
    async fn save_app_role_entity_permission(
        &self,
        tenant_id: TenantId,
        permission: AppEntityRolePermission,
    ) -> AppResult<()>;

    /// Lists configured role permissions for an app.
    async fn list_app_role_entity_permissions(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityRolePermission>>;

    /// Lists apps accessible to the subject by role bindings.
    async fn list_accessible_apps(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<AppDefinition>>;

    /// Returns whether the subject can access an app.
    async fn subject_can_access_app(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<bool>;

    /// Returns effective subject permissions for one entity in an app.
    async fn subject_entity_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<Option<SubjectEntityPermission>>;

    /// Returns effective subject permissions for every entity in an app.
    async fn list_subject_entity_permissions(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<Vec<SubjectEntityPermission>>;
}
