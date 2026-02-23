use qryvanta_domain::{
    AppDefinition, AppEntityBinding, AppEntityRolePermission, AppEntityViewMode, AppSitemap,
    SitemapArea, SitemapGroup, SitemapSubArea, SitemapTarget,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// App-scoped default worker view mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-entity-view-mode.ts"
)]
pub enum AppEntityViewModeDto {
    Grid,
    Json,
}

/// Incoming payload for app creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-app-request.ts"
)]
pub struct CreateAppRequest {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
}

/// API representation of an app definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-response.ts"
)]
pub struct AppResponse {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
}

/// Incoming payload for binding an entity into app navigation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/bind-app-entity-request.ts"
)]
pub struct BindAppEntityRequest {
    pub entity_logical_name: String,
    pub navigation_label: Option<String>,
    pub navigation_order: i32,
    #[serde(default)]
    pub forms: Option<Vec<AppEntityFormDto>>,
    #[serde(default)]
    pub list_views: Option<Vec<AppEntityViewDto>>,
    #[serde(default)]
    pub default_form_logical_name: Option<String>,
    #[serde(default)]
    pub default_list_view_logical_name: Option<String>,
    #[serde(default)]
    pub form_field_logical_names: Option<Vec<String>>,
    #[serde(default)]
    pub list_field_logical_names: Option<Vec<String>>,
    #[serde(default)]
    pub default_view_mode: Option<AppEntityViewModeDto>,
}

/// API representation of an app entity navigation binding.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-entity-binding-response.ts"
)]
pub struct AppEntityBindingResponse {
    pub app_logical_name: String,
    pub entity_logical_name: String,
    pub navigation_label: Option<String>,
    pub navigation_order: i32,
    pub forms: Vec<AppEntityFormDto>,
    pub list_views: Vec<AppEntityViewDto>,
    pub default_form_logical_name: String,
    pub default_list_view_logical_name: String,
    pub form_field_logical_names: Vec<String>,
    pub list_field_logical_names: Vec<String>,
    pub default_view_mode: AppEntityViewModeDto,
}

/// API representation of an app-scoped entity form.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-entity-form-dto.ts"
)]
pub struct AppEntityFormDto {
    pub logical_name: String,
    pub display_name: String,
    pub field_logical_names: Vec<String>,
}

/// API representation of an app-scoped entity list view.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-entity-view-dto.ts"
)]
pub struct AppEntityViewDto {
    pub logical_name: String,
    pub display_name: String,
    pub field_logical_names: Vec<String>,
}

/// Incoming payload for app role entity permission updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/save-app-role-entity-permission-request.ts"
)]
pub struct SaveAppRoleEntityPermissionRequest {
    pub role_name: String,
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// API representation of app-scoped role entity permissions.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-role-entity-permission-response.ts"
)]
pub struct AppRoleEntityPermissionResponse {
    pub app_logical_name: String,
    pub role_name: String,
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// API representation of effective app entity capabilities for the current subject.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-entity-capabilities-response.ts"
)]
pub struct AppEntityCapabilitiesResponse {
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// Incoming payload for app sitemap updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/save-app-sitemap-request.ts"
)]
pub struct SaveAppSitemapRequest {
    pub areas: Vec<AppSitemapAreaDto>,
}

/// API representation of app sitemap.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-sitemap-response.ts"
)]
pub struct AppSitemapResponse {
    pub app_logical_name: String,
    pub areas: Vec<AppSitemapAreaDto>,
}

/// API representation of sitemap area.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-sitemap-area-dto.ts"
)]
pub struct AppSitemapAreaDto {
    pub logical_name: String,
    pub display_name: String,
    pub position: i32,
    pub icon: Option<String>,
    pub groups: Vec<AppSitemapGroupDto>,
}

/// API representation of sitemap group.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-sitemap-group-dto.ts"
)]
pub struct AppSitemapGroupDto {
    pub logical_name: String,
    pub display_name: String,
    pub position: i32,
    pub sub_areas: Vec<AppSitemapSubAreaDto>,
}

/// API representation of sitemap sub area.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-sitemap-sub-area-dto.ts"
)]
pub struct AppSitemapSubAreaDto {
    pub logical_name: String,
    pub display_name: String,
    pub position: i32,
    pub icon: Option<String>,
    pub target: AppSitemapTargetDto,
}

/// API representation of sub area target.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case", tag = "type")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-sitemap-target-dto.ts"
)]
pub enum AppSitemapTargetDto {
    Entity {
        entity_logical_name: String,
        default_form: Option<String>,
        default_view: Option<String>,
    },
    Dashboard {
        dashboard_logical_name: String,
    },
    CustomPage {
        url: String,
    },
}

impl From<AppDefinition> for AppResponse {
    fn from(value: AppDefinition) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            description: value.description().map(ToOwned::to_owned),
        }
    }
}

impl From<AppEntityBinding> for AppEntityBindingResponse {
    fn from(value: AppEntityBinding) -> Self {
        Self {
            app_logical_name: value.app_logical_name().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            navigation_label: value.navigation_label().map(ToOwned::to_owned),
            navigation_order: value.navigation_order(),
            forms: value
                .forms()
                .iter()
                .map(|form| AppEntityFormDto {
                    logical_name: form.logical_name().as_str().to_owned(),
                    display_name: form.display_name().as_str().to_owned(),
                    field_logical_names: form.field_logical_names().to_vec(),
                })
                .collect(),
            list_views: value
                .list_views()
                .iter()
                .map(|view| AppEntityViewDto {
                    logical_name: view.logical_name().as_str().to_owned(),
                    display_name: view.display_name().as_str().to_owned(),
                    field_logical_names: view.field_logical_names().to_vec(),
                })
                .collect(),
            default_form_logical_name: value.default_form_logical_name().as_str().to_owned(),
            default_list_view_logical_name: value
                .default_list_view_logical_name()
                .as_str()
                .to_owned(),
            form_field_logical_names: value
                .forms()
                .iter()
                .find(|form| {
                    form.logical_name().as_str() == value.default_form_logical_name().as_str()
                })
                .map(|form| form.field_logical_names().to_vec())
                .unwrap_or_default(),
            list_field_logical_names: value
                .list_views()
                .iter()
                .find(|view| {
                    view.logical_name().as_str() == value.default_list_view_logical_name().as_str()
                })
                .map(|view| view.field_logical_names().to_vec())
                .unwrap_or_default(),
            default_view_mode: value.default_view_mode().into(),
        }
    }
}

impl From<AppEntityViewMode> for AppEntityViewModeDto {
    fn from(value: AppEntityViewMode) -> Self {
        match value {
            AppEntityViewMode::Grid => Self::Grid,
            AppEntityViewMode::Json => Self::Json,
        }
    }
}

impl From<AppEntityViewModeDto> for AppEntityViewMode {
    fn from(value: AppEntityViewModeDto) -> Self {
        match value {
            AppEntityViewModeDto::Grid => Self::Grid,
            AppEntityViewModeDto::Json => Self::Json,
        }
    }
}

impl From<AppEntityRolePermission> for AppRoleEntityPermissionResponse {
    fn from(value: AppEntityRolePermission) -> Self {
        Self {
            app_logical_name: value.app_logical_name().as_str().to_owned(),
            role_name: value.role_name().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            can_read: value.can_read(),
            can_create: value.can_create(),
            can_update: value.can_update(),
            can_delete: value.can_delete(),
        }
    }
}

impl From<AppSitemap> for AppSitemapResponse {
    fn from(value: AppSitemap) -> Self {
        Self {
            app_logical_name: value.app_logical_name().as_str().to_owned(),
            areas: value
                .areas()
                .iter()
                .cloned()
                .map(AppSitemapAreaDto::from)
                .collect(),
        }
    }
}

impl From<SitemapArea> for AppSitemapAreaDto {
    fn from(value: SitemapArea) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            position: value.position(),
            icon: value.icon().map(ToOwned::to_owned),
            groups: value
                .groups()
                .iter()
                .cloned()
                .map(AppSitemapGroupDto::from)
                .collect(),
        }
    }
}

impl From<SitemapGroup> for AppSitemapGroupDto {
    fn from(value: SitemapGroup) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            position: value.position(),
            sub_areas: value
                .sub_areas()
                .iter()
                .cloned()
                .map(AppSitemapSubAreaDto::from)
                .collect(),
        }
    }
}

impl From<SitemapSubArea> for AppSitemapSubAreaDto {
    fn from(value: SitemapSubArea) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            position: value.position(),
            icon: value.icon().map(ToOwned::to_owned),
            target: value.target().clone().into(),
        }
    }
}

impl From<SitemapTarget> for AppSitemapTargetDto {
    fn from(value: SitemapTarget) -> Self {
        match value {
            SitemapTarget::Entity {
                entity_logical_name,
                default_form,
                default_view,
            } => Self::Entity {
                entity_logical_name,
                default_form,
                default_view,
            },
            SitemapTarget::Dashboard {
                dashboard_logical_name,
            } => Self::Dashboard {
                dashboard_logical_name,
            },
            SitemapTarget::CustomPage { url } => Self::CustomPage { url },
        }
    }
}

impl TryFrom<AppSitemapTargetDto> for SitemapTarget {
    type Error = qryvanta_core::AppError;

    fn try_from(value: AppSitemapTargetDto) -> Result<Self, Self::Error> {
        Ok(match value {
            AppSitemapTargetDto::Entity {
                entity_logical_name,
                default_form,
                default_view,
            } => SitemapTarget::Entity {
                entity_logical_name,
                default_form,
                default_view,
            },
            AppSitemapTargetDto::Dashboard {
                dashboard_logical_name,
            } => SitemapTarget::Dashboard {
                dashboard_logical_name,
            },
            AppSitemapTargetDto::CustomPage { url } => SitemapTarget::CustomPage { url },
        })
    }
}

impl From<qryvanta_application::SubjectEntityPermission> for AppEntityCapabilitiesResponse {
    fn from(value: qryvanta_application::SubjectEntityPermission) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name,
            can_read: value.can_read,
            can_create: value.can_create,
            can_update: value.can_update,
            can_delete: value.can_delete,
        }
    }
}
