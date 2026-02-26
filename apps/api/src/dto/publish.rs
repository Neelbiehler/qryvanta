use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Publish check issue severity.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-check-severity-dto.ts"
)]
pub enum PublishCheckSeverityDto {
    Error,
}

/// Publish check issue scope.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-check-scope-dto.ts"
)]
pub enum PublishCheckScopeDto {
    Entity,
    App,
}

/// Publish check issue category.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-check-category-dto.ts"
)]
pub enum PublishCheckCategoryDto {
    Schema,
    Relationship,
    Form,
    View,
    Sitemap,
    Binding,
    Unknown,
}

/// One publish check issue.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-check-issue-response.ts"
)]
pub struct PublishCheckIssueResponse {
    pub scope: PublishCheckScopeDto,
    pub scope_logical_name: String,
    pub category: PublishCheckCategoryDto,
    pub severity: PublishCheckSeverityDto,
    pub message: String,
    pub fix_path: Option<String>,
    pub dependency_path: Option<String>,
}

/// Workspace-level publish checks response.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workspace-publish-checks-response.ts"
)]
pub struct WorkspacePublishChecksResponse {
    pub is_publishable: bool,
    pub checked_entities: usize,
    pub checked_apps: usize,
    pub issues: Vec<PublishCheckIssueResponse>,
}

/// Request payload for selective workspace publish execution.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/run-workspace-publish-request.ts"
)]
pub struct RunWorkspacePublishRequest {
    #[serde(default)]
    pub entity_logical_names: Vec<String>,
    #[serde(default)]
    pub app_logical_names: Vec<String>,
    #[serde(default)]
    pub dry_run: bool,
}

/// Result payload for selective workspace publish execution.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/run-workspace-publish-response.ts"
)]
pub struct RunWorkspacePublishResponse {
    pub is_publishable: bool,
    pub requested_entities: usize,
    pub requested_apps: usize,
    pub published_entities: Vec<String>,
    pub validated_apps: Vec<String>,
    pub issues: Vec<PublishCheckIssueResponse>,
}

/// One persisted workspace publish run history entry.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workspace-publish-history-entry-response.ts"
)]
pub struct WorkspacePublishHistoryEntryResponse {
    pub run_id: String,
    pub run_at: String,
    pub subject: String,
    pub requested_entities: usize,
    pub requested_apps: usize,
    pub requested_entity_logical_names: Vec<String>,
    pub requested_app_logical_names: Vec<String>,
    pub published_entities: Vec<String>,
    pub validated_apps: Vec<String>,
    pub issue_count: usize,
    pub is_publishable: bool,
}

/// Request payload for publish diff preview generation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workspace-publish-diff-request.ts"
)]
pub struct WorkspacePublishDiffRequest {
    #[serde(default)]
    pub entity_logical_names: Vec<String>,
    #[serde(default)]
    pub app_logical_names: Vec<String>,
}

/// Field-level diff between draft and latest published schema.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-field-diff-item-response.ts"
)]
pub struct PublishFieldDiffItemResponse {
    pub field_logical_name: String,
    pub change_type: String,
    pub draft_field_type: Option<String>,
    pub published_field_type: Option<String>,
    pub draft_relation_target: Option<String>,
    pub published_relation_target: Option<String>,
}

/// Form/view-level summary item for diff preview.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-surface-diff-item-response.ts"
)]
pub struct PublishSurfaceDiffItemResponse {
    pub logical_name: String,
    pub display_name: String,
    pub item_count: usize,
    pub is_default: bool,
}

/// Form/view-level draft-vs-published structural delta item.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/publish-surface-delta-item-response.ts"
)]
pub struct PublishSurfaceDeltaItemResponse {
    pub logical_name: String,
    pub change_type: String,
    pub draft_display_name: Option<String>,
    pub published_display_name: Option<String>,
    pub draft_item_count: Option<usize>,
    pub published_item_count: Option<usize>,
    pub draft_is_default: Option<bool>,
    pub published_is_default: Option<bool>,
}

/// Entity-level publish diff summary.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/entity-publish-diff-response.ts"
)]
pub struct EntityPublishDiffResponse {
    pub entity_logical_name: String,
    pub published_schema_exists: bool,
    pub field_diff: Vec<PublishFieldDiffItemResponse>,
    pub forms: Vec<PublishSurfaceDeltaItemResponse>,
    pub views: Vec<PublishSurfaceDeltaItemResponse>,
}

/// App-entity binding summary used in app-level diff preview.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-binding-diff-response.ts"
)]
pub struct AppBindingDiffResponse {
    pub entity_logical_name: String,
    pub navigation_order: i32,
    pub navigation_label: Option<String>,
    pub default_form_logical_name: String,
    pub default_list_view_logical_name: String,
    pub forms: Vec<PublishSurfaceDiffItemResponse>,
    pub views: Vec<PublishSurfaceDiffItemResponse>,
}

/// App-level publish diff summary.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/app-publish-diff-response.ts"
)]
pub struct AppPublishDiffResponse {
    pub app_logical_name: String,
    pub bindings: Vec<AppBindingDiffResponse>,
}

/// Full workspace publish diff preview response.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/workspace-publish-diff-response.ts"
)]
pub struct WorkspacePublishDiffResponse {
    pub unknown_entity_logical_names: Vec<String>,
    pub unknown_app_logical_names: Vec<String>,
    pub entity_diffs: Vec<EntityPublishDiffResponse>,
    pub app_diffs: Vec<AppPublishDiffResponse>,
}
