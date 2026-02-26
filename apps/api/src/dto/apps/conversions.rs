use qryvanta_domain::{
    AppDefinition, AppEntityBinding, AppEntityRolePermission, AppEntityViewMode, AppSitemap,
    ChartAggregation, ChartDefinition, ChartType, DashboardDefinition, DashboardWidget,
    SitemapArea, SitemapGroup, SitemapSubArea, SitemapTarget,
};

use super::types::{
    AppEntityBindingResponse, AppEntityCapabilitiesResponse, AppEntityFormDto, AppEntityViewDto,
    AppEntityViewModeDto, AppResponse, AppRoleEntityPermissionResponse, AppSitemapAreaDto,
    AppSitemapGroupDto, AppSitemapResponse, AppSitemapSubAreaDto, AppSitemapTargetDto,
    ChartAggregationDto, ChartResponse, ChartTypeDto, DashboardWidgetResponse,
    WorkspaceDashboardResponse,
};

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

impl From<DashboardDefinition> for WorkspaceDashboardResponse {
    fn from(value: DashboardDefinition) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            widgets: value
                .widgets()
                .iter()
                .cloned()
                .map(DashboardWidgetResponse::from)
                .collect(),
        }
    }
}

impl From<DashboardWidget> for DashboardWidgetResponse {
    fn from(value: DashboardWidget) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            position: value.position(),
            width: value.width(),
            height: value.height(),
            chart: value.chart().clone().into(),
        }
    }
}

impl From<ChartDefinition> for ChartResponse {
    fn from(value: ChartDefinition) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            view_logical_name: value
                .view_logical_name()
                .map(|name| name.as_str().to_owned()),
            chart_type: value.chart_type().into(),
            aggregation: value.aggregation().into(),
            category_field_logical_name: value
                .category_field_logical_name()
                .map(|name| name.as_str().to_owned()),
            value_field_logical_name: value
                .value_field_logical_name()
                .map(|name| name.as_str().to_owned()),
        }
    }
}

impl From<ChartType> for ChartTypeDto {
    fn from(value: ChartType) -> Self {
        match value {
            ChartType::Kpi => Self::Kpi,
            ChartType::Bar => Self::Bar,
            ChartType::Line => Self::Line,
            ChartType::Pie => Self::Pie,
        }
    }
}

impl From<ChartAggregation> for ChartAggregationDto {
    fn from(value: ChartAggregation) -> Self {
        match value {
            ChartAggregation::Count => Self::Count,
            ChartAggregation::Sum => Self::Sum,
            ChartAggregation::Avg => Self::Avg,
            ChartAggregation::Min => Self::Min,
            ChartAggregation::Max => Self::Max,
        }
    }
}
