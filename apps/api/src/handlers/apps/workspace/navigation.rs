use axum::Json;
use axum::extract::{Extension, Path, State};
use qryvanta_core::UserIdentity;

use crate::dto::{
    AppEntityCapabilitiesResponse, AppResponse, AppSitemapResponse, FormResponse,
    PublishedSchemaResponse, ViewResponse, WorkspaceDashboardResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_workspace_apps_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<AppResponse>>> {
    let apps = state
        .app_service
        .list_accessible_apps(&user)
        .await?
        .into_iter()
        .map(AppResponse::from)
        .collect();

    Ok(Json(apps))
}

pub async fn app_navigation_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
) -> ApiResult<Json<AppSitemapResponse>> {
    let sitemap = state
        .app_service
        .app_navigation_for_subject(&user, app_logical_name.as_str())
        .await?;

    Ok(Json(AppSitemapResponse::from(sitemap)))
}

pub async fn workspace_dashboard_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, dashboard_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<WorkspaceDashboardResponse>> {
    let dashboard = state
        .app_service
        .get_dashboard_for_subject(
            &user,
            app_logical_name.as_str(),
            dashboard_logical_name.as_str(),
        )
        .await?;

    Ok(Json(WorkspaceDashboardResponse::from(dashboard)))
}

pub async fn workspace_entity_schema_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<PublishedSchemaResponse>> {
    let schema = state
        .app_service
        .schema_for_subject(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
        )
        .await?;

    Ok(Json(PublishedSchemaResponse::from(schema)))
}

pub async fn workspace_entity_capabilities_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<AppEntityCapabilitiesResponse>> {
    let capabilities = state
        .app_service
        .entity_capabilities_for_subject(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
        )
        .await?;

    Ok(Json(AppEntityCapabilitiesResponse::from(capabilities)))
}

pub async fn workspace_list_forms_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<Vec<FormResponse>>> {
    let forms = state
        .app_service
        .list_entity_forms(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
        )
        .await?
        .into_iter()
        .map(FormResponse::from)
        .collect();

    Ok(Json(forms))
}

pub async fn workspace_get_form_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name, form_logical_name)): Path<(
        String,
        String,
        String,
    )>,
) -> ApiResult<Json<FormResponse>> {
    let form = state
        .app_service
        .get_entity_form(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            form_logical_name.as_str(),
        )
        .await?;

    Ok(Json(FormResponse::from(form)))
}

pub async fn workspace_list_views_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name)): Path<(String, String)>,
) -> ApiResult<Json<Vec<ViewResponse>>> {
    let views = state
        .app_service
        .list_entity_views(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
        )
        .await?
        .into_iter()
        .map(ViewResponse::from)
        .collect();

    Ok(Json(views))
}

pub async fn workspace_get_view_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((app_logical_name, entity_logical_name, view_logical_name)): Path<(
        String,
        String,
        String,
    )>,
) -> ApiResult<Json<ViewResponse>> {
    let view = state
        .app_service
        .get_entity_view(
            &user,
            app_logical_name.as_str(),
            entity_logical_name.as_str(),
            view_logical_name.as_str(),
        )
        .await?;

    Ok(Json(ViewResponse::from(view)))
}
