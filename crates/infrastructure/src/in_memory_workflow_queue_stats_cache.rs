use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use qryvanta_application::{WorkflowQueueStats, WorkflowQueueStatsCache, WorkflowQueueStatsQuery};
use qryvanta_core::AppResult;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy)]
struct QueueStatsCacheEntry {
    stats: WorkflowQueueStats,
    expires_at: Instant,
}

/// In-memory cache adapter for workflow queue stats.
#[derive(Default)]
pub struct InMemoryWorkflowQueueStatsCache {
    entries: RwLock<HashMap<WorkflowQueueStatsQuery, QueueStatsCacheEntry>>,
}

impl InMemoryWorkflowQueueStatsCache {
    /// Creates an empty in-memory queue stats cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl WorkflowQueueStatsCache for InMemoryWorkflowQueueStatsCache {
    async fn get_queue_stats(
        &self,
        query: WorkflowQueueStatsQuery,
    ) -> AppResult<Option<WorkflowQueueStats>> {
        {
            let entries = self.entries.read().await;
            if let Some(entry) = entries.get(&query) {
                if entry.expires_at > Instant::now() {
                    return Ok(Some(entry.stats));
                }
            } else {
                return Ok(None);
            }
        }

        let mut entries = self.entries.write().await;
        if entries
            .get(&query)
            .is_some_and(|entry| entry.expires_at <= Instant::now())
        {
            entries.remove(&query);
        }

        Ok(None)
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

        let now = Instant::now();
        let expires_at = now
            .checked_add(Duration::from_secs(u64::from(ttl_seconds)))
            .unwrap_or(now);

        self.entries
            .write()
            .await
            .insert(query, QueueStatsCacheEntry { stats, expires_at });

        Ok(())
    }
}
