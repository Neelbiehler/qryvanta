use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use qryvanta_core::{AppError, UserIdentity};
use tracing::warn;

use crate::dto::{
    BusinessRuleResponse, CreateRuntimeRecordRequest, QueryRuntimeRecordsRequest,
    RuntimeRecordResponse, UpdateRuntimeRecordRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

mod handlers;
mod query;

pub use handlers::{
    create_runtime_record_handler, delete_runtime_record_handler, get_runtime_record_handler,
    list_runtime_business_rules_handler, list_runtime_records_handler,
    query_runtime_records_handler, update_runtime_record_handler,
};
pub(crate) use query::runtime_record_query_from_request;

#[cfg(test)]
mod tests;
