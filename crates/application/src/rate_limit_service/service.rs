use std::sync::Arc;

use chrono::Utc;

use qryvanta_core::{AppError, AppResult};

use super::config::RateLimitRule;
use super::ports::RateLimitRepository;

/// Application service for rate limiting.
#[derive(Clone)]
pub struct RateLimitService {
    repository: Arc<dyn RateLimitRepository>,
}

impl RateLimitService {
    /// Creates a new rate limit service.
    #[must_use]
    pub fn new(repository: Arc<dyn RateLimitRepository>) -> Self {
        Self { repository }
    }

    /// Checks whether the given key is within the rate limit.
    ///
    /// Records the attempt and returns `Ok(())` if allowed, or
    /// `Err(AppError::RateLimited)` if the limit has been exceeded.
    ///
    /// The key should be formatted as `"{category}:{identifier}"` where
    /// identifier is typically an IP address or email.
    pub async fn check_rate_limit(&self, rule: &RateLimitRule, key: &str) -> AppResult<()> {
        let composite_key = format!("{}:{key}", rule.category);
        let info = self
            .repository
            .record_attempt(&composite_key, rule.window_seconds)
            .await?;

        if info.attempt_count > rule.max_attempts {
            return Err(AppError::RateLimited(
                "too many requests, please try again later".to_owned(),
            ));
        }

        Ok(())
    }

    /// Removes expired rate limit entries. Intended for periodic cleanup.
    pub async fn cleanup(&self) -> AppResult<u64> {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        self.repository.cleanup_expired(cutoff).await
    }
}
