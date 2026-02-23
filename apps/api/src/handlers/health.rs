use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use redis::AsyncCommands;

use crate::dto::{HealthDependencyStatus, HealthResponse};
use crate::state::AppState;

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

async fn check_postgres(pool: sqlx::PgPool) -> HealthDependencyStatus {
    let check = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&pool)
        .await;

    match check {
        Ok(_) => HealthDependencyStatus {
            status: "ok",
            detail: None,
        },
        Err(error) => HealthDependencyStatus {
            status: "error",
            detail: Some(format!("postgres check failed: {error}")),
        },
    }
}

async fn check_redis(
    redis_client: Option<redis::Client>,
    redis_required: bool,
) -> HealthDependencyStatus {
    let Some(redis_client) = redis_client else {
        return if redis_required {
            HealthDependencyStatus {
                status: "error",
                detail: Some("redis client is not configured".to_owned()),
            }
        } else {
            HealthDependencyStatus {
                status: "disabled",
                detail: None,
            }
        };
    };

    let mut connection = match redis_client.get_multiplexed_async_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            return HealthDependencyStatus {
                status: "error",
                detail: Some(format!("redis connection failed: {error}")),
            };
        }
    };

    let ping_response = connection.ping::<String>().await;
    match ping_response {
        Ok(value) if value.eq_ignore_ascii_case("pong") => HealthDependencyStatus {
            status: "ok",
            detail: None,
        },
        Ok(value) => HealthDependencyStatus {
            status: "error",
            detail: Some(format!("unexpected redis ping response: {value}")),
        },
        Err(error) => HealthDependencyStatus {
            status: "error",
            detail: Some(format!("redis ping failed: {error}")),
        },
    }
}
