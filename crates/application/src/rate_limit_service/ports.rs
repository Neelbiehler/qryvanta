use async_trait::async_trait;
use chrono::{DateTime, Utc};

use qryvanta_core::AppResult;

/// Repository port for rate limit persistence.
#[async_trait]
pub trait RateLimitRepository: Send + Sync {
    /// Records an attempt for the given key.
    ///
    /// Uses an UPSERT pattern: if the current window has expired, resets the
    /// counter. Returns the updated attempt count within the active window.
    async fn record_attempt(
        &self,
        key: &str,
        window_duration_seconds: i64,
    ) -> AppResult<AttemptInfo>;

    /// Removes expired entries older than the given cutoff.
    async fn cleanup_expired(&self, before: DateTime<Utc>) -> AppResult<u64>;
}

/// Information about the current rate limit window for a key.
#[derive(Debug, Clone)]
pub struct AttemptInfo {
    /// Number of attempts in the current window (including this one).
    pub attempt_count: i32,
    /// When the current window started.
    pub window_started_at: DateTime<Utc>,
}
