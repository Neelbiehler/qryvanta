use serde::Serialize;
use ts_rs::TS;

/// API error payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/error-response.ts"
)]
pub struct ErrorResponse {
    message: String,
}

impl ErrorResponse {
    pub(super) fn new(message: String) -> Self {
        Self { message }
    }
}
