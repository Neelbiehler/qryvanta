use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;

use qryvanta_application::{WorkflowClaimPartition, WorkflowWorkerHeartbeatInput};
use qryvanta_core::AppError;
use qryvanta_domain::{WorkflowAction, WorkflowStep, WorkflowTrigger};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ApiResult;
use crate::middleware::WorkerIdentity;
use crate::state::AppState;

mod claim;
mod heartbeat;
mod stats;

pub use claim::claim_workflow_jobs_handler;
pub use heartbeat::worker_heartbeat_handler;
pub use stats::workflow_queue_stats_handler;

fn parse_worker_partition(
    partition_count: Option<u32>,
    partition_index: Option<u32>,
    max_partition_count: u32,
) -> Result<Option<WorkflowClaimPartition>, AppError> {
    match (partition_count, partition_index) {
        (None, None) => Ok(None),
        (Some(requested_partition_count), Some(requested_partition_index)) => {
            let effective_partition_count = requested_partition_count.clamp(1, max_partition_count);

            WorkflowClaimPartition::new(effective_partition_count, requested_partition_index)
                .map(Some)
        }
        _ => Err(AppError::Validation(
            "partition_count and partition_index must be provided together".to_owned(),
        )),
    }
}
