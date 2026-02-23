use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use qryvanta_core::UserIdentity;
use qryvanta_domain::{AppSitemap, SitemapArea, SitemapGroup, SitemapSubArea, SitemapTarget};

use crate::dto::{
    AppEntityBindingResponse, AppResponse, AppRoleEntityPermissionResponse, AppSitemapAreaDto,
    AppSitemapGroupDto, AppSitemapResponse, AppSitemapSubAreaDto, AppSitemapTargetDto,
    BindAppEntityRequest, CreateAppRequest, SaveAppRoleEntityPermissionRequest,
    SaveAppSitemapRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_apps_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<AppResponse>>> {
    let apps = state
        .app_service
        .list_apps(&user)
        .await?
        .into_iter()
        .map(AppResponse::from)
        .collect();

    Ok(Json(apps))
}

pub async fn create_app_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateAppRequest>,
) -> ApiResult<(StatusCode, Json<AppResponse>)> {
    let app = state
        .app_service
        .create_app(
            &user,
            qryvanta_application::CreateAppInput {
                logical_name: payload.logical_name,
                display_name: payload.display_name,
                description: payload.description,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(AppResponse::from(app))))
}

pub async fn list_app_entities_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
) -> ApiResult<Json<Vec<AppEntityBindingResponse>>> {
    let entities = state
        .app_service
        .list_app_entities(&user, app_logical_name.as_str())
        .await?
        .into_iter()
        .map(AppEntityBindingResponse::from)
        .collect();

    Ok(Json(entities))
}

pub async fn bind_app_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
    Json(payload): Json<BindAppEntityRequest>,
) -> ApiResult<(StatusCode, Json<AppEntityBindingResponse>)> {
    let binding = state
        .app_service
        .bind_entity(
            &user,
            qryvanta_application::BindAppEntityInput {
                app_logical_name,
                entity_logical_name: payload.entity_logical_name,
                navigation_label: payload.navigation_label,
                navigation_order: payload.navigation_order,
                forms: payload.forms.map(|forms| {
                    forms
                        .into_iter()
                        .map(|form| qryvanta_application::AppEntityFormInput {
                            logical_name: form.logical_name,
                            display_name: form.display_name,
                            field_logical_names: form.field_logical_names,
                        })
                        .collect()
                }),
                list_views: payload.list_views.map(|views| {
                    views
                        .into_iter()
                        .map(|view| qryvanta_application::AppEntityViewInput {
                            logical_name: view.logical_name,
                            display_name: view.display_name,
                            field_logical_names: view.field_logical_names,
                        })
                        .collect()
                }),
                default_form_logical_name: payload.default_form_logical_name,
                default_list_view_logical_name: payload.default_list_view_logical_name,
                form_field_logical_names: payload.form_field_logical_names,
                list_field_logical_names: payload.list_field_logical_names,
                default_view_mode: payload.default_view_mode.map(Into::into),
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(AppEntityBindingResponse::from(binding)),
    ))
}

pub async fn list_app_role_permissions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
) -> ApiResult<Json<Vec<AppRoleEntityPermissionResponse>>> {
    let permissions = state
        .app_service
        .list_role_entity_permissions(&user, app_logical_name.as_str())
        .await?
        .into_iter()
        .map(AppRoleEntityPermissionResponse::from)
        .collect();

    Ok(Json(permissions))
}

pub async fn save_app_role_permission_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
    Json(payload): Json<SaveAppRoleEntityPermissionRequest>,
) -> ApiResult<Json<AppRoleEntityPermissionResponse>> {
    let permission = state
        .app_service
        .save_role_entity_permission(
            &user,
            qryvanta_application::SaveAppRoleEntityPermissionInput {
                app_logical_name,
                role_name: payload.role_name,
                entity_logical_name: payload.entity_logical_name,
                can_read: payload.can_read,
                can_create: payload.can_create,
                can_update: payload.can_update,
                can_delete: payload.can_delete,
            },
        )
        .await?;

    Ok(Json(AppRoleEntityPermissionResponse::from(permission)))
}

pub async fn get_app_sitemap_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
) -> ApiResult<Json<AppSitemapResponse>> {
    let sitemap = state
        .app_service
        .get_sitemap(&user, app_logical_name.as_str())
        .await?;
    Ok(Json(AppSitemapResponse::from(sitemap)))
}

pub async fn save_app_sitemap_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(app_logical_name): Path<String>,
    Json(payload): Json<SaveAppSitemapRequest>,
) -> ApiResult<Json<AppSitemapResponse>> {
    let sitemap = AppSitemap::new(
        app_logical_name.clone(),
        payload
            .areas
            .into_iter()
            .map(area_dto_to_domain)
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    let saved = state
        .app_service
        .save_sitemap(
            &user,
            qryvanta_application::SaveAppSitemapInput {
                app_logical_name,
                sitemap,
            },
        )
        .await?;
    Ok(Json(AppSitemapResponse::from(saved)))
}

fn area_dto_to_domain(area: AppSitemapAreaDto) -> Result<SitemapArea, qryvanta_core::AppError> {
    SitemapArea::new(
        area.logical_name,
        area.display_name,
        area.position,
        area.icon,
        area.groups
            .into_iter()
            .map(group_dto_to_domain)
            .collect::<Result<Vec<_>, _>>()?,
    )
}

fn group_dto_to_domain(group: AppSitemapGroupDto) -> Result<SitemapGroup, qryvanta_core::AppError> {
    SitemapGroup::new(
        group.logical_name,
        group.display_name,
        group.position,
        group
            .sub_areas
            .into_iter()
            .map(sub_area_dto_to_domain)
            .collect::<Result<Vec<_>, _>>()?,
    )
}

fn sub_area_dto_to_domain(
    sub_area: AppSitemapSubAreaDto,
) -> Result<SitemapSubArea, qryvanta_core::AppError> {
    SitemapSubArea::new(
        sub_area.logical_name,
        sub_area.display_name,
        sub_area.position,
        match sub_area.target {
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
        },
        sub_area.icon,
    )
}
