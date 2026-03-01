use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Search request proxied from Qryvanta to Qrywell.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-request.ts"
)]
pub struct QrywellSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub roles: Option<Vec<String>>,
    pub include_debug: Option<bool>,
}

/// Click analytics request for one search result interaction.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-click-event-request.ts"
)]
pub struct QrywellSearchClickEventRequest {
    pub search_event_id: Option<String>,
    pub query: String,
    pub result_id: String,
    pub rank: usize,
    pub score: f32,
    pub title: String,
    pub connector_type: String,
    pub group_label: Option<String>,
}

/// One search hit returned by Qrywell.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-hit-response.ts"
)]
pub struct QrywellSearchHitResponse {
    pub id: String,
    pub document_id: String,
    pub connector_type: String,
    pub title: String,
    pub url: String,
    pub text: String,
    pub score: f32,
}

/// Search response returned by Qryvanta API.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-response.ts"
)]
pub struct QrywellSearchResponse {
    pub search_event_id: Option<String>,
    pub query: String,
    pub total_hits: usize,
    pub hits: Vec<QrywellSearchHitResponse>,
    pub debug_query_normalized: Option<String>,
    pub debug_selected_entity: Option<String>,
    pub debug_planned_filter_count: Option<usize>,
    pub debug_negated_filter_count: Option<usize>,
}

/// Top query analytics row.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-top-query-response.ts"
)]
pub struct QrywellSearchTopQueryResponse {
    pub query: String,
    pub runs: i64,
    pub clicks: i64,
}

/// Rank click analytics row.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-rank-metric-response.ts"
)]
pub struct QrywellSearchRankMetricResponse {
    pub rank: i32,
    pub clicks: i64,
    pub click_share: f32,
}

/// Zero-click query analytics row.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-zero-click-query-response.ts"
)]
pub struct QrywellSearchZeroClickQueryResponse {
    pub query: String,
    pub runs: i64,
}

/// Low relevance clicked result analytics row.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-low-relevance-click-response.ts"
)]
pub struct QrywellSearchLowRelevanceClickResponse {
    pub query: String,
    pub title: String,
    pub avg_score: f32,
    pub clicks: i64,
}

/// Aggregated search analytics response.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-search-analytics-response.ts"
)]
pub struct QrywellSearchAnalyticsResponse {
    pub window_days: i32,
    pub total_queries: i64,
    pub total_clicks: i64,
    pub top_queries: Vec<QrywellSearchTopQueryResponse>,
    pub rank_metrics: Vec<QrywellSearchRankMetricResponse>,
    pub zero_click_queries: Vec<QrywellSearchZeroClickQueryResponse>,
    pub low_relevance_clicks: Vec<QrywellSearchLowRelevanceClickResponse>,
}

/// Manual sync request for pushing runtime records to Qrywell.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-sync-request.ts"
)]
pub struct QrywellSyncRequest {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Sync result summary for Qrywell indexing.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-sync-response.ts"
)]
pub struct QrywellSyncResponse {
    pub entity_logical_name: String,
    pub synced_records: usize,
    pub indexed_chunks: usize,
}

/// Sync-all response summary across entities.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-sync-all-response.ts"
)]
pub struct QrywellSyncAllResponse {
    pub entities: Vec<QrywellSyncResponse>,
    pub total_entities: usize,
    pub total_records: usize,
    pub total_chunks: usize,
}

/// One failed Qrywell sync job.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-sync-failed-job-response.ts"
)]
pub struct QrywellSyncFailedJobResponse {
    pub job_id: String,
    pub entity_logical_name: String,
    pub record_id: String,
    pub operation: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub next_attempt_at: String,
    pub updated_at: String,
    pub last_error: Option<String>,
}

/// Queue health summary for Qrywell sync.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/qrywell-sync-health-response.ts"
)]
pub struct QrywellSyncHealthResponse {
    pub pending_jobs: i64,
    pub processing_jobs: i64,
    pub failed_jobs: i64,
    pub total_succeeded: i64,
    pub total_failed: i64,
    pub last_attempt_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_failure_at: Option<String>,
    pub failed_recent: Vec<QrywellSyncFailedJobResponse>,
}
