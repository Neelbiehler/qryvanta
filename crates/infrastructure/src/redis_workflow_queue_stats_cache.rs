//! Redis-backed workflow queue stats cache.

use async_trait::async_trait;
use qryvanta_application::{WorkflowQueueStats, WorkflowQueueStatsCache, WorkflowQueueStatsQuery};
use qryvanta_core::{AppError, AppResult};
use redis::AsyncCommands;

/// Redis implementation of the workflow queue stats cache port.
#[derive(Clone)]
pub struct RedisWorkflowQueueStatsCache {
    client: redis::Client,
    key_prefix: String,
}

impl RedisWorkflowQueueStatsCache {
    /// Creates a cache adapter with a configured Redis client and key prefix.
    #[must_use]
    pub fn new(client: redis::Client, key_prefix: impl Into<String>) -> Self {
        Self {
            client,
            key_prefix: key_prefix.into(),
        }
    }

    fn key_for(&self, query: WorkflowQueueStatsQuery) -> String {
        match query.partition {
            Some(partition) => format!(
                "{}:window={}:partition={}:{}",
                self.key_prefix,
                query.active_window_seconds,
                partition.partition_count(),
                partition.partition_index()
            ),
            None => format!(
                "{}:window={}:partition=none",
                self.key_prefix, query.active_window_seconds
            ),
        }
    }

    fn encode_stats(stats: WorkflowQueueStats) -> String {
        format!(
            "{},{},{},{},{},{}",
            stats.pending_jobs,
            stats.leased_jobs,
            stats.completed_jobs,
            stats.failed_jobs,
            stats.expired_leases,
            stats.active_workers
        )
    }

    fn decode_stats(value: &str) -> AppResult<WorkflowQueueStats> {
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() != 6 {
            return Err(AppError::Internal(format!(
                "invalid workflow queue stats cache value '{value}'"
            )));
        }

        Ok(WorkflowQueueStats {
            pending_jobs: parse_metric(parts[0], "pending_jobs")?,
            leased_jobs: parse_metric(parts[1], "leased_jobs")?,
            completed_jobs: parse_metric(parts[2], "completed_jobs")?,
            failed_jobs: parse_metric(parts[3], "failed_jobs")?,
            expired_leases: parse_metric(parts[4], "expired_leases")?,
            active_workers: parse_metric(parts[5], "active_workers")?,
        })
    }
}

#[async_trait]
impl WorkflowQueueStatsCache for RedisWorkflowQueueStatsCache {
    async fn get_queue_stats(
        &self,
        query: WorkflowQueueStatsQuery,
    ) -> AppResult<Option<WorkflowQueueStats>> {
        let key = self.key_for(query);
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| AppError::Internal(format!("failed to connect to redis: {error}")))?;

        let encoded: Option<String> = connection.get(key).await.map_err(|error| {
            AppError::Internal(format!(
                "failed to read workflow queue stats cache entry: {error}"
            ))
        })?;

        encoded.as_deref().map(Self::decode_stats).transpose()
    }

    async fn set_queue_stats(
        &self,
        query: WorkflowQueueStatsQuery,
        stats: WorkflowQueueStats,
        ttl_seconds: u32,
    ) -> AppResult<()> {
        if ttl_seconds == 0 {
            return Ok(());
        }

        let key = self.key_for(query);
        let value = Self::encode_stats(stats);
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| AppError::Internal(format!("failed to connect to redis: {error}")))?;

        connection
            .set_ex(key, value, u64::from(ttl_seconds))
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to write workflow queue stats cache entry: {error}"
                ))
            })
    }
}

fn parse_metric(value: &str, metric_name: &str) -> AppResult<i64> {
    value.parse::<i64>().map_err(|error| {
        AppError::Internal(format!(
            "invalid workflow queue stats cache field '{metric_name}' value '{value}': {error}"
        ))
    })
}
