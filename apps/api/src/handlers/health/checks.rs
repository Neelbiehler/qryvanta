use redis::AsyncCommands;

use super::*;

pub(super) async fn check_postgres(pool: sqlx::PgPool) -> HealthDependencyStatus {
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

pub(super) async fn check_redis(
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
