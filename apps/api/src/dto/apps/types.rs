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

/// Worker-facing dashboard metadata response.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workspace-dashboard-response.ts"
)]
pub struct WorkspaceDashboardResponse {
    pub logical_name: String,
    pub display_name: String,
    pub widgets: Vec<DashboardWidgetResponse>,
}

/// Worker-facing dashboard widget metadata response.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/dashboard-widget-response.ts"
)]
pub struct DashboardWidgetResponse {
    pub logical_name: String,
    pub display_name: String,
    pub position: i32,
    pub width: i32,
    pub height: i32,
    pub chart: ChartResponse,
}

/// Worker-facing chart metadata response.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/chart-response.ts"
)]
pub struct ChartResponse {
    pub logical_name: String,
    pub display_name: String,
    pub entity_logical_name: String,
    pub view_logical_name: Option<String>,
    pub chart_type: ChartTypeDto,
    pub aggregation: ChartAggregationDto,
    pub category_field_logical_name: Option<String>,
    pub value_field_logical_name: Option<String>,
}

/// API transport enum for chart visualization type.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/chart-type-dto.ts"
)]
pub enum ChartTypeDto {
    Kpi,
    Bar,
    Line,
    Pie,
}

/// API transport enum for chart aggregation.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/chart-aggregation-dto.ts"
)]
pub enum ChartAggregationDto {
    Count,
    Sum,
    Avg,
    Min,
    Max,
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

/// App-level publish validation report.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-publish-checks-response.ts"
)]
pub struct AppPublishChecksResponse {
    pub is_publishable: bool,
    pub errors: Vec<String>,
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
