use async_trait::async_trait;
use qryvanta_core::AppResult;

use super::execution::{WorkflowQueueStats, WorkflowQueueStatsQuery};

/// Optional cache port for queue stats.
#[async_trait]
pub trait WorkflowQueueStatsCache: Send + Sync {
    /// Returns cached queue stats for one query.
    async fn get_queue_stats(
        &self,
        query: WorkflowQueueStatsQuery,
    ) -> AppResult<Option<WorkflowQueueStats>>;

    /// Stores queue stats for one query with ttl.
    async fn set_queue_stats(
        &self,
        query: WorkflowQueueStatsQuery,
        stats: WorkflowQueueStats,
        ttl_seconds: u32,
    ) -> AppResult<()>;
}
