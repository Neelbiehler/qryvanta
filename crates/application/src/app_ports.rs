use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityAction, AppEntityBinding, AppEntityRolePermission,
    PublishedEntitySchema, RuntimeRecord,
};
use serde_json::Value;

use crate::metadata_ports::{RecordListQuery, RuntimeRecordQuery};

/// Input payload for app creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAppInput {
    /// Stable app logical name.
    pub logical_name: String,
    /// App display name.
    pub display_name: String,
    /// Optional app description.
    pub description: Option<String>,
}

/// Input payload for binding an entity into app navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindAppEntityInput {
    /// Parent app logical name.
    pub app_logical_name: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Optional display label in navigation.
    pub navigation_label: Option<String>,
    /// Display ordering value.
    pub navigation_order: i32,
}

/// Input payload for app role entity permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveAppRoleEntityPermissionInput {
    /// Parent app logical name.
    pub app_logical_name: String,
    /// Role name to configure.
    pub role_name: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Whether read access is granted.
    pub can_read: bool,
    /// Whether create access is granted.
    pub can_create: bool,
    /// Whether update access is granted.
    pub can_update: bool,
    /// Whether delete access is granted.
    pub can_delete: bool,
}

/// Effective subject permissions for an entity in an app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectEntityPermission {
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Read access.
    pub can_read: bool,
    /// Create access.
    pub can_create: bool,
    /// Update access.
    pub can_update: bool,
    /// Delete access.
    pub can_delete: bool,
}

impl SubjectEntityPermission {
    /// Returns whether an action is allowed by this capability.
    #[must_use]
    pub fn allows(&self, action: AppEntityAction) -> bool {
        match action {
            AppEntityAction::Read => self.can_read,
            AppEntityAction::Create => self.can_create,
            AppEntityAction::Update => self.can_update,
            AppEntityAction::Delete => self.can_delete,
        }
    }
}

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

/// Runtime record gateway used by app-scoped execution.
#[async_trait]
pub trait RuntimeRecordService: Send + Sync {
    /// Returns latest published schema for an entity.
    async fn latest_published_schema_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>>;

    /// Lists runtime records without global permission checks.
    async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>>;

    /// Queries runtime records without global permission checks.
    async fn query_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>>;

    /// Fetches one runtime record without global permission checks.
    async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord>;

    /// Creates runtime record without global permission checks.
    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Updates runtime record without global permission checks.
    async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Deletes runtime record without global permission checks.
    async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()>;
}
