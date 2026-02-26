use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// Incoming runtime record create payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-runtime-record-request.ts"
)]
pub struct CreateRuntimeRecordRequest {
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}

/// Incoming runtime record update payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/update-runtime-record-request.ts"
)]
pub struct UpdateRuntimeRecordRequest {
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}

/// Incoming runtime record query payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-record-query-filter-request.ts"
)]
pub struct RuntimeRecordQueryFilterRequest {
    #[ts(type = "string | null")]
    pub scope_alias: Option<String>,
    pub field_logical_name: String,
    #[ts(type = "\"eq\" | \"neq\" | \"gt\" | \"gte\" | \"lt\" | \"lte\" | \"contains\" | \"in\"")]
    pub operator: String,
    #[ts(type = "unknown")]
    pub field_value: Value,
}

/// Incoming runtime query where-clause group payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-record-query-group-request.ts"
)]
pub struct RuntimeRecordQueryGroupRequest {
    #[ts(type = "\"and\" | \"or\" | null")]
    pub logical_mode: Option<String>,
    pub conditions: Option<Vec<RuntimeRecordQueryFilterRequest>>,
    pub groups: Option<Vec<RuntimeRecordQueryGroupRequest>>,
}

/// Incoming runtime query link-entity payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-record-query-link-entity-request.ts"
)]
pub struct RuntimeRecordQueryLinkEntityRequest {
    pub alias: String,
    #[ts(type = "string | null")]
    pub parent_alias: Option<String>,
    pub relation_field_logical_name: String,
    #[ts(type = "\"inner\" | \"left\" | null")]
    pub join_type: Option<String>,
}

/// Incoming runtime record query sort payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-record-query-sort-request.ts"
)]
pub struct RuntimeRecordQuerySortRequest {
    #[ts(type = "string | null")]
    pub scope_alias: Option<String>,
    pub field_logical_name: String,
    #[ts(type = "\"asc\" | \"desc\" | null")]
    pub direction: Option<String>,
}

/// Incoming runtime record query payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/query-runtime-records-request.ts"
)]
pub struct QueryRuntimeRecordsRequest {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[ts(type = "\"and\" | \"or\" | null")]
    pub logical_mode: Option<String>,
    #[serde(rename = "where")]
    pub where_clause: Option<RuntimeRecordQueryGroupRequest>,
    pub conditions: Option<Vec<RuntimeRecordQueryFilterRequest>>,
    pub link_entities: Option<Vec<RuntimeRecordQueryLinkEntityRequest>>,
    pub sort: Option<Vec<RuntimeRecordQuerySortRequest>>,
    /// Legacy exact-match map; converted to `eq` conditions when present.
    #[ts(type = "Record<string, unknown> | null")]
    pub filters: Option<BTreeMap<String, Value>>,
}

/// API representation of a runtime record.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/runtime-record-response.ts"
)]
pub struct RuntimeRecordResponse {
    pub record_id: String,
    pub entity_logical_name: String,
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}
