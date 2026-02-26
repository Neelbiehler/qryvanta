use super::checks::{check_postgres, check_redis};
use super::*;

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
