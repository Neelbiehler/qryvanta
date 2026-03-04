use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// API response containing one portability bundle payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workspace-portable-bundle-response.ts"
)]
pub struct WorkspacePortableBundleResponse {
    #[ts(type = "unknown")]
    pub bundle: Value,
}

/// API request for workspace portability bundle import.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/import-workspace-portable-bundle-request.ts"
)]
pub struct ImportWorkspacePortableBundleRequest {
    #[ts(type = "unknown")]
    pub bundle: Value,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default = "default_true")]
    pub import_metadata: bool,
    #[serde(default = "default_true")]
    pub import_runtime_data: bool,
    #[serde(default)]
    pub remap_record_ids: bool,
}

/// API response for workspace portability bundle import.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/import-workspace-portable-bundle-response.ts"
)]
pub struct ImportWorkspacePortableBundleResponse {
    pub dry_run: bool,
    pub entities_processed: usize,
    pub runtime_records_discovered: usize,
    pub runtime_records_created: usize,
    pub runtime_records_updated: usize,
    pub runtime_records_remapped: usize,
    pub relation_rewrites: usize,
}

const fn default_true() -> bool {
    true
}
