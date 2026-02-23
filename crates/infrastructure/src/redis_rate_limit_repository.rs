//! Redis-backed rate limit repository.

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use qryvanta_application::{AttemptInfo, RateLimitRepository};
use qryvanta_core::{AppError, AppResult};
use redis::Script;

const RECORD_ATTEMPT_SCRIPT: &str = r#"
local key = KEYS[1]
local window = tonumber(ARGV[1])
local now_epoch = tonumber(ARGV[2])

local count = redis.call('INCR', key)
local ttl = redis.call('TTL', key)

if ttl < 0 then
  redis.call('EXPIRE', key, window)
  ttl = window
end

local window_started = now_epoch - (window - ttl)
return {count, window_started}
"#;

/// Redis implementation of the rate limit repository port.
#[derive(Clone)]
pub struct RedisRateLimitRepository {
    client: redis::Client,
    key_prefix: String,
}

impl RedisRateLimitRepository {
    /// Creates a repository with a configured Redis client and key prefix.
    #[must_use]
    pub fn new(client: redis::Client, key_prefix: impl Into<String>) -> Self {
        Self {
            client,
            key_prefix: key_prefix.into(),
        }
    }

    fn key_for(&self, key: &str) -> String {
        format!("{}:{key}", self.key_prefix)
    }
}

#[async_trait]
impl RateLimitRepository for RedisRateLimitRepository {
    async fn record_attempt(
        &self,
        key: &str,
        window_duration_seconds: i64,
    ) -> AppResult<AttemptInfo> {
        if window_duration_seconds <= 0 {
            return Err(AppError::Validation(
                "window_duration_seconds must be greater than zero".to_owned(),
            ));
        }

        let redis_key = self.key_for(key);
        let window_duration = i32::try_from(window_duration_seconds).map_err(|error| {
            AppError::Validation(format!("invalid rate limit window duration: {error}"))
        })?;
        let now = Utc::now();

        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| AppError::Internal(format!("failed to connect to redis: {error}")))?;

        let script = Script::new(RECORD_ATTEMPT_SCRIPT);
        let (attempt_count, window_started_epoch): (i64, i64) = script
            .key(redis_key)
            .arg(window_duration)
            .arg(now.timestamp())
            .invoke_async(&mut connection)
            .await
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to record redis rate limit attempt: {error}"
                ))
            })?;

        let attempt_count = i32::try_from(attempt_count)
            .map_err(|error| AppError::Internal(format!("invalid redis attempt count: {error}")))?;
        let window_started_at = Utc
            .timestamp_opt(window_started_epoch, 0)
            .single()
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "invalid redis window start timestamp: {window_started_epoch}"
                ))
            })?;

        Ok(AttemptInfo {
            attempt_count,
            window_started_at,
        })
    }

    async fn cleanup_expired(&self, _before: DateTime<Utc>) -> AppResult<u64> {
        // Redis rate limit keys expire automatically via TTL.
        Ok(0)
    }
}
