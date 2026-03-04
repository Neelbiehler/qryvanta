use super::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(super) const PORTABLE_PACKAGE_FORMAT: &str = "qryvanta.workspace.portable";
pub(super) const PORTABLE_PACKAGE_VERSION: i32 = 1;

/// Export options for workspace portability bundles.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExportWorkspaceBundleOptions {
    /// Includes metadata definitions and publish state.
    pub include_metadata: bool,
    /// Includes runtime records.
    pub include_runtime_data: bool,
}

impl Default for ExportWorkspaceBundleOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_runtime_data: true,
        }
    }
}

/// A portable workspace package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePortableBundle {
    /// Stable package format identifier.
    pub package_format: String,
    /// Stable package format version.
    pub package_version: i32,
    /// UTC export timestamp.
    pub exported_at: DateTime<Utc>,
    /// SHA-256 checksum of canonicalized payload JSON.
    pub payload_sha256: String,
    /// Exported payload.
    pub payload: WorkspacePortablePayload,
}

/// Payload section of the portable package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePortablePayload {
    /// Tenant id from the source workspace.
    pub tenant_id: String,
    /// Exported entities.
    pub entities: Vec<PortableEntityBundle>,
    /// Export option echo.
    pub include_metadata: bool,
    /// Export option echo.
    pub include_runtime_data: bool,
}

/// One entity section inside a portability package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableEntityBundle {
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Entity definition when metadata is exported.
    pub entity: Option<EntityDefinition>,
    /// Field definitions when metadata is exported.
    pub fields: Vec<EntityFieldDefinition>,
    /// Option set definitions when metadata is exported.
    pub option_sets: Vec<OptionSetDefinition>,
    /// Form definitions when metadata is exported.
    pub forms: Vec<FormDefinition>,
    /// View definitions when metadata is exported.
    pub views: Vec<ViewDefinition>,
    /// Business-rule definitions when metadata is exported.
    pub business_rules: Vec<BusinessRuleDefinition>,
    /// Latest published schema snapshot when metadata is exported.
    pub published_schema: Option<PublishedEntitySchema>,
    /// Runtime records when runtime export is enabled.
    pub runtime_records: Vec<PortableRuntimeRecord>,
}

/// Runtime record payload inside a portability package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableRuntimeRecord {
    /// Stable source record id.
    pub record_id: String,
    /// Runtime JSON payload.
    pub data: Value,
}

/// Import options for workspace portability bundles.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImportWorkspaceBundleOptions {
    /// Validates bundle compatibility only.
    pub dry_run: bool,
    /// Imports metadata definitions and publish state.
    pub import_metadata: bool,
    /// Imports runtime records.
    pub import_runtime_data: bool,
    /// Remaps imported record identifiers deterministically.
    pub remap_record_ids: bool,
}

impl Default for ImportWorkspaceBundleOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            import_metadata: true,
            import_runtime_data: true,
            remap_record_ids: false,
        }
    }
}

/// Import execution summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWorkspaceBundleResult {
    /// Indicates whether import ran in dry-run mode.
    pub dry_run: bool,
    /// Number of entities considered.
    pub entities_processed: usize,
    /// Runtime records discovered in the bundle.
    pub runtime_records_discovered: usize,
    /// Runtime records created during apply phase.
    pub runtime_records_created: usize,
    /// Runtime records updated during apply phase.
    pub runtime_records_updated: usize,
    /// Runtime records remapped by deterministic id transformation.
    pub runtime_records_remapped: usize,
    /// Number of relation field values rewritten by remapping.
    pub relation_rewrites: usize,
}

pub(super) struct PlannedRuntimeRecordImport {
    pub(super) entity_logical_name: String,
    pub(super) source_record_id: String,
    pub(super) target_record_id: String,
    pub(super) rewritten_data: Value,
    pub(super) will_create: bool,
}

mod export;
mod import;
mod import_apply_metadata;
mod import_runtime;
mod transform;
mod validation;
