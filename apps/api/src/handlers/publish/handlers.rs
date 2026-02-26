use axum::Json;
use axum::extract::{Extension, Query, State};

use qryvanta_application::{AuditLogQuery, WorkspacePublishRunAuditInput};
use qryvanta_core::UserIdentity;
use qryvanta_domain::AuditAction;

use crate::dto::{
    AppBindingDiffResponse, AppPublishDiffResponse, EntityPublishDiffResponse,
    PublishCheckScopeDto, RunWorkspacePublishRequest, RunWorkspacePublishResponse,
    WorkspacePublishChecksResponse, WorkspacePublishDiffRequest, WorkspacePublishDiffResponse,
    WorkspacePublishHistoryEntryResponse,
};
use crate::error::ApiResult;

use super::diff::{compute_field_diff, compute_form_surface_delta, compute_view_surface_delta};
use super::history::map_workspace_publish_history_entries;
use super::issues::{
    build_unknown_selection_issues, collect_workspace_issues, partition_known_names,
    resolve_requested_names,
};
use super::{PublishHistoryQuery, PublishState};

pub async fn workspace_publish_checks_handler(
    State(state): State<PublishState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<WorkspacePublishChecksResponse>> {
    let entities = state.metadata_service.list_entities(&user).await?;
    let apps = state.app_service.list_apps(&user).await?;

    let entity_names = entities
        .iter()
        .map(|entity| entity.logical_name().as_str().to_owned())
        .collect::<Vec<_>>();
    let app_names = apps
        .iter()
        .map(|app| app.logical_name().as_str().to_owned())
        .collect::<Vec<_>>();

    let issues = collect_workspace_issues(&state, &user, &entity_names, &app_names).await?;

    Ok(Json(WorkspacePublishChecksResponse {
        is_publishable: issues.is_empty(),
        checked_entities: entity_names.len(),
        checked_apps: app_names.len(),
        issues,
    }))
}

pub async fn run_workspace_publish_handler(
    State(state): State<PublishState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<RunWorkspacePublishRequest>,
) -> ApiResult<Json<RunWorkspacePublishResponse>> {
    let entities = state.metadata_service.list_entities(&user).await?;
    let apps = state.app_service.list_apps(&user).await?;

    let available_entity_names = entities
        .iter()
        .map(|entity| entity.logical_name().as_str().to_owned())
        .collect::<Vec<_>>();
    let available_app_names = apps
        .iter()
        .map(|app| app.logical_name().as_str().to_owned())
        .collect::<Vec<_>>();

    let requested_entities =
        resolve_requested_names(payload.entity_logical_names, available_entity_names.clone());
    let requested_apps =
        resolve_requested_names(payload.app_logical_names, available_app_names.clone());

    let (known_entity_names, unknown_entity_names) =
        partition_known_names(&requested_entities, &available_entity_names);
    let (known_app_names, unknown_app_names) =
        partition_known_names(&requested_apps, &available_app_names);

    let mut issues = Vec::new();
    issues.extend(build_unknown_selection_issues(
        PublishCheckScopeDto::Entity,
        &unknown_entity_names,
    ));
    issues.extend(build_unknown_selection_issues(
        PublishCheckScopeDto::App,
        &unknown_app_names,
    ));

    issues.extend(
        collect_workspace_issues(&state, &user, &known_entity_names, &known_app_names).await?,
    );

    let mut published_entities = Vec::new();
    let mut validated_apps = Vec::new();
    let should_publish = issues.is_empty() && !payload.dry_run;

    if issues.is_empty() {
        validated_apps = known_app_names.clone();

        if !should_publish {
            let response = RunWorkspacePublishResponse {
                is_publishable: true,
                requested_entities: requested_entities.len(),
                requested_apps: requested_apps.len(),
                published_entities,
                validated_apps,
                issues,
            };

            return Ok(Json(response));
        }

        for entity_logical_name in &known_entity_names {
            state
                .metadata_service
                .publish_entity_with_allowed_unpublished_entities(
                    &user,
                    entity_logical_name,
                    &known_entity_names,
                )
                .await?;
            published_entities.push(entity_logical_name.clone());
        }
    }

    let response = RunWorkspacePublishResponse {
        is_publishable: issues.is_empty(),
        requested_entities: requested_entities.len(),
        requested_apps: requested_apps.len(),
        published_entities,
        validated_apps,
        issues,
    };

    if !payload.dry_run {
        state
            .security_admin_service
            .record_workspace_publish_run(
                &user,
                WorkspacePublishRunAuditInput {
                    requested_entities: response.requested_entities,
                    requested_apps: response.requested_apps,
                    requested_entity_logical_names: requested_entities.clone(),
                    requested_app_logical_names: requested_apps.clone(),
                    published_entities: response.published_entities.clone(),
                    validated_apps: response.validated_apps.clone(),
                    issue_count: response.issues.len(),
                    is_publishable: response.is_publishable,
                },
            )
            .await?;
    }

    Ok(Json(response))
}

pub async fn workspace_publish_history_handler(
    State(state): State<PublishState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<PublishHistoryQuery>,
) -> ApiResult<Json<Vec<WorkspacePublishHistoryEntryResponse>>> {
    let entries = state
        .security_admin_service
        .list_audit_log(
            &user,
            AuditLogQuery {
                limit: query.limit.unwrap_or(20).clamp(1, 100),
                offset: 0,
                action: Some(AuditAction::MetadataWorkspacePublished.as_str().to_owned()),
                subject: None,
            },
        )
        .await?;

    Ok(Json(map_workspace_publish_history_entries(entries)))
}

pub async fn workspace_publish_diff_handler(
    State(state): State<PublishState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<WorkspacePublishDiffRequest>,
) -> ApiResult<Json<WorkspacePublishDiffResponse>> {
    let entities = state.metadata_service.list_entities(&user).await?;
    let apps = state.app_service.list_apps(&user).await?;

    let available_entity_names = entities
        .iter()
        .map(|entity| entity.logical_name().as_str().to_owned())
        .collect::<Vec<_>>();
    let available_app_names = apps
        .iter()
        .map(|app| app.logical_name().as_str().to_owned())
        .collect::<Vec<_>>();

    let requested_entities =
        resolve_requested_names(payload.entity_logical_names, available_entity_names.clone());
    let requested_apps =
        resolve_requested_names(payload.app_logical_names, available_app_names.clone());

    let (known_entity_names, unknown_entity_names) =
        partition_known_names(&requested_entities, &available_entity_names);
    let (known_app_names, unknown_app_names) =
        partition_known_names(&requested_apps, &available_app_names);

    let mut entity_diffs = Vec::new();
    for entity_logical_name in &known_entity_names {
        let draft_fields = state
            .metadata_service
            .list_fields(&user, entity_logical_name.as_str())
            .await?;
        let draft_forms = state
            .metadata_service
            .list_forms(&user, entity_logical_name.as_str())
            .await?;
        let draft_views = state
            .metadata_service
            .list_views(&user, entity_logical_name.as_str())
            .await?;
        let published_forms = state
            .metadata_service
            .list_latest_published_form_snapshots(&user, entity_logical_name.as_str())
            .await?;
        let published_views = state
            .metadata_service
            .list_latest_published_view_snapshots(&user, entity_logical_name.as_str())
            .await?;
        let published_schema = state
            .metadata_service
            .latest_published_schema(&user, entity_logical_name.as_str())
            .await?;

        entity_diffs.push(EntityPublishDiffResponse {
            entity_logical_name: entity_logical_name.clone(),
            published_schema_exists: published_schema.is_some(),
            field_diff: compute_field_diff(&draft_fields, published_schema.as_ref()),
            forms: compute_form_surface_delta(&draft_forms, &published_forms),
            views: compute_view_surface_delta(&draft_views, &published_views),
        });
    }

    let mut app_diffs = Vec::new();
    for app_logical_name in &known_app_names {
        let bindings = state
            .app_service
            .list_app_entities(&user, app_logical_name.as_str())
            .await?;

        app_diffs.push(AppPublishDiffResponse {
            app_logical_name: app_logical_name.clone(),
            bindings: bindings
                .iter()
                .map(|binding| AppBindingDiffResponse {
                    entity_logical_name: binding.entity_logical_name().as_str().to_owned(),
                    navigation_order: binding.navigation_order(),
                    navigation_label: binding.navigation_label().map(str::to_owned),
                    default_form_logical_name: binding
                        .default_form_logical_name()
                        .as_str()
                        .to_owned(),
                    default_list_view_logical_name: binding
                        .default_list_view_logical_name()
                        .as_str()
                        .to_owned(),
                    forms: binding
                        .forms()
                        .iter()
                        .map(|form| crate::dto::PublishSurfaceDiffItemResponse {
                            logical_name: form.logical_name().as_str().to_owned(),
                            display_name: form.display_name().as_str().to_owned(),
                            item_count: form.field_logical_names().len(),
                            is_default: form.logical_name().as_str()
                                == binding.default_form_logical_name().as_str(),
                        })
                        .collect(),
                    views: binding
                        .list_views()
                        .iter()
                        .map(|view| crate::dto::PublishSurfaceDiffItemResponse {
                            logical_name: view.logical_name().as_str().to_owned(),
                            display_name: view.display_name().as_str().to_owned(),
                            item_count: view.field_logical_names().len(),
                            is_default: view.logical_name().as_str()
                                == binding.default_list_view_logical_name().as_str(),
                        })
                        .collect(),
                })
                .collect(),
        });
    }

    Ok(Json(WorkspacePublishDiffResponse {
        unknown_entity_logical_names: unknown_entity_names,
        unknown_app_logical_names: unknown_app_names,
        entity_diffs,
        app_diffs,
    }))
}
