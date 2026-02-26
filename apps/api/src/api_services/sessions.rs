use qryvanta_core::AppError;
use sqlx::PgPool;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

use crate::redis_session_store::RedisSessionStore;

pub async fn build_postgres_session_layer(
    pool: PgPool,
    cookie_secure: bool,
) -> Result<SessionManagerLayer<PostgresStore>, AppError> {
    let session_store = PostgresStore::new(pool)
        .with_table_name("tower_sessions")
        .map_err(|error| {
            AppError::Validation(format!("invalid session table name configuration: {error}"))
        })?;

    session_store.migrate().await.map_err(|error| {
        AppError::Internal(format!("failed to initialize session store: {error}"))
    })?;

    Ok(SessionManagerLayer::new(session_store)
        .with_secure(cookie_secure)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(30))))
}

pub async fn build_redis_session_layer(
    redis_client: redis::Client,
    cookie_secure: bool,
) -> Result<SessionManagerLayer<RedisSessionStore>, AppError> {
    let session_store = RedisSessionStore::new(redis_client, "qryvanta:session");

    Ok(SessionManagerLayer::new(session_store)
        .with_secure(cookie_secure)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(30))))
}
