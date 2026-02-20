//! Rate limiting ports and application service.
//!
//! Implements a sliding-window rate limiter backed by the `auth_rate_limits`
//! database table. Follows OWASP Credential Stuffing Prevention cheat sheet
//! recommendations for per-IP and per-endpoint throttling.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use qryvanta_core::AppResult;

// ---------------------------------------------------------------------------
// Ports
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for a rate limit rule.
#[derive(Debug, Clone)]
pub struct RateLimitRule {
    /// The route prefix or category name (e.g., "login", "forgot_password").
    pub category: String,
    /// Maximum number of attempts allowed in the window.
    pub max_attempts: i32,
    /// Window duration in seconds.
    pub window_seconds: i64,
}

impl RateLimitRule {
    /// Creates a new rate limit rule.
    #[must_use]
    pub fn new(category: impl Into<String>, max_attempts: i32, window_seconds: i64) -> Self {
        Self {
            category: category.into(),
            max_attempts,
            window_seconds,
        }
    }
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

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
            return Err(qryvanta_core::AppError::RateLimited(
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
