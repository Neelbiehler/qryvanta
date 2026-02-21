use std::collections::BTreeMap;

use qryvanta_domain::RuntimeRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// Incoming runtime record create payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/create-runtime-record-request.ts"
)]
pub struct CreateRuntimeRecordRequest {
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}

/// Incoming runtime record update payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/update-runtime-record-request.ts"
)]
pub struct UpdateRuntimeRecordRequest {
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}

/// Incoming runtime record query payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/query-runtime-records-request.ts"
)]
pub struct QueryRuntimeRecordsRequest {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[ts(type = "Record<string, unknown> | null")]
    pub filters: Option<BTreeMap<String, Value>>,
}

/// API representation of a runtime record.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/runtime-record-response.ts"
)]
pub struct RuntimeRecordResponse {
    pub record_id: String,
    pub entity_logical_name: String,
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
}

impl From<RuntimeRecord> for RuntimeRecordResponse {
    fn from(value: RuntimeRecord) -> Self {
        Self {
            record_id: value.record_id().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            data: value.data().clone(),
        }
    }
}
