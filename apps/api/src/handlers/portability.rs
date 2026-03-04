use axum::Json;
use axum::extract::{Extension, Query, State};

use qryvanta_application::{
    ExportWorkspaceBundleOptions, ImportWorkspaceBundleOptions, WorkspacePortableBundle,
};
use qryvanta_core::{AppError, UserIdentity};

use crate::dto::{
    ImportWorkspacePortableBundleRequest, ImportWorkspacePortableBundleResponse,
    WorkspacePortableBundleResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct ExportWorkspaceBundleQuery {
    pub include_metadata: Option<bool>,
    pub include_runtime_data: Option<bool>,
}

pub async fn export_workspace_bundle_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<ExportWorkspaceBundleQuery>,
) -> ApiResult<Json<WorkspacePortableBundleResponse>> {
    let bundle = state
        .metadata_service
        .export_workspace_bundle(
            &user,
            ExportWorkspaceBundleOptions {
                include_metadata: query.include_metadata.unwrap_or(true),
                include_runtime_data: query.include_runtime_data.unwrap_or(true),
            },
        )
        .await?;

    let bundle = serde_json::to_value(bundle).map_err(|error| {
        AppError::Internal(format!("failed to encode portability bundle: {error}"))
    })?;

    Ok(Json(WorkspacePortableBundleResponse { bundle }))
}

pub async fn import_workspace_bundle_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<ImportWorkspacePortableBundleRequest>,
) -> ApiResult<Json<ImportWorkspacePortableBundleResponse>> {
    let bundle: WorkspacePortableBundle =
        serde_json::from_value(payload.bundle).map_err(|error| {
            AppError::Validation(format!(
                "invalid workspace portability bundle payload: {error}"
            ))
        })?;

    let summary = state
        .metadata_service
        .import_workspace_bundle(
            &user,
            bundle,
            ImportWorkspaceBundleOptions {
                dry_run: payload.dry_run,
                import_metadata: payload.import_metadata,
                import_runtime_data: payload.import_runtime_data,
                remap_record_ids: payload.remap_record_ids,
            },
        )
        .await?;

    Ok(Json(ImportWorkspacePortableBundleResponse {
        dry_run: summary.dry_run,
        entities_processed: summary.entities_processed,
        runtime_records_discovered: summary.runtime_records_discovered,
        runtime_records_created: summary.runtime_records_created,
        runtime_records_updated: summary.runtime_records_updated,
        runtime_records_remapped: summary.runtime_records_remapped,
        relation_rewrites: summary.relation_rewrites,
    }))
}
