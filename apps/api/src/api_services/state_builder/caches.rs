use std::sync::Arc;

use qryvanta_application::{RateLimitRepository, RateLimitService, WorkflowQueueStatsCache};
use qryvanta_core::{AppError, AppResult};
use qryvanta_infrastructure::{
    InMemoryWorkflowQueueStatsCache, PostgresRateLimitRepository, RedisRateLimitRepository,
    RedisWorkflowQueueStatsCache,
};
use sqlx::PgPool;

use crate::api_config::{ApiConfig, RateLimitStoreConfig, WorkflowQueueStatsCacheBackend};

pub(super) fn build_workflow_queue_stats_cache(
    config: &ApiConfig,
    redis_client: Option<redis::Client>,
) -> AppResult<Arc<dyn WorkflowQueueStatsCache>> {
    match config.workflow_queue_stats_cache_backend {
        WorkflowQueueStatsCacheBackend::InMemory => {
            Ok(Arc::new(InMemoryWorkflowQueueStatsCache::new()))
        }
        WorkflowQueueStatsCacheBackend::Redis => {
            let redis_client = redis_client.ok_or_else(|| {
                AppError::Validation(
                    "REDIS_URL is required when WORKFLOW_QUEUE_STATS_CACHE_BACKEND=redis"
                        .to_owned(),
                )
            })?;
            Ok(Arc::new(RedisWorkflowQueueStatsCache::new(
                redis_client,
                "qryvanta:workflow_queue_stats",
            )))
        }
    }
}

pub(super) fn build_rate_limit_service(
    pool: &PgPool,
    config: &ApiConfig,
    redis_client: Option<redis::Client>,
) -> AppResult<RateLimitService> {
    let rate_limit_repository: Arc<dyn RateLimitRepository> = match config.rate_limit_store {
        RateLimitStoreConfig::Postgres => Arc::new(PostgresRateLimitRepository::new(pool.clone())),
        RateLimitStoreConfig::Redis => {
            let redis_client = redis_client.ok_or_else(|| {
                AppError::Validation("REDIS_URL is required when RATE_LIMIT_STORE=redis".to_owned())
            })?;
            Arc::new(RedisRateLimitRepository::new(
                redis_client,
                "qryvanta:rate_limit",
            ))
        }
    };

    Ok(RateLimitService::new(rate_limit_repository))
}
