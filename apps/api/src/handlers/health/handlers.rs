use super::checks::{check_postgres, check_redis};
use super::*;
use crate::observability::render_metrics_prometheus;

pub async fn health_handler(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let postgres = check_postgres(state.postgres_pool.clone()).await;
    let redis = check_redis(state.redis_client.clone(), state.redis_required).await;

    let ready = is_healthy(postgres.status) && (is_healthy(redis.status) || !state.redis_required);
    let status = if ready { "ok" } else { "degraded" };
    let http_status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        http_status,
        Json(HealthResponse {
            status,
            ready,
            postgres,
            redis,
        }),
    )
}

fn is_healthy(status: &str) -> bool {
    status == "ok"
}

pub async fn metrics_handler(
    State(state): State<AppState>,
) -> (StatusCode, [(&'static str, &'static str); 1], String) {
    let queue_stats = state.workflow_service.queue_stats(60).await.ok();
    let metrics = render_metrics_prometheus(
        state.observability_metrics.snapshot(),
        queue_stats,
        state.slow_request_threshold_ms,
        state.slow_query_threshold_ms,
    );

    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics,
    )
}
