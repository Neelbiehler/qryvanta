use qryvanta_domain::{AppEntityViewMode, AppSitemap};

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
    /// Optional model-driven forms for this app entity.
    pub forms: Option<Vec<AppEntityFormInput>>,
    /// Optional model-driven list views for this app entity.
    pub list_views: Option<Vec<AppEntityViewInput>>,
    /// Optional default form logical name.
    pub default_form_logical_name: Option<String>,
    /// Optional default list view logical name.
    pub default_list_view_logical_name: Option<String>,
    /// Optional app-specific form field order override.
    pub form_field_logical_names: Option<Vec<String>>,
    /// Optional app-specific list field order override.
    pub list_field_logical_names: Option<Vec<String>>,
    /// Optional default worker view mode override.
    pub default_view_mode: Option<AppEntityViewMode>,
}

/// Input payload for one app-scoped form definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppEntityFormInput {
    /// Stable form logical name.
    pub logical_name: String,
    /// Human-readable form display name.
    pub display_name: String,
    /// Ordered field logical names rendered in this form.
    pub field_logical_names: Vec<String>,
}

/// Input payload for one app-scoped list view definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppEntityViewInput {
    /// Stable list view logical name.
    pub logical_name: String,
    /// Human-readable list view display name.
    pub display_name: String,
    /// Ordered field logical names rendered as columns.
    pub field_logical_names: Vec<String>,
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

/// Input payload for saving an app sitemap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveAppSitemapInput {
    /// Parent app logical name.
    pub app_logical_name: String,
    /// Full sitemap definition.
    pub sitemap: AppSitemap,
}
