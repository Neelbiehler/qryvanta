//! Qryvanta API composition root.

#![forbid(unsafe_code)]

mod api_config;
mod api_router;
mod api_services;
mod auth;
mod dev_seed;
mod dto;
mod error;
mod handlers;
mod middleware;
mod qrywell_sync;
mod redis_session_store;
mod state;

use qryvanta_core::AppError;
use tracing::info;

use crate::api_config::SessionStoreBackend;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    api_config::init_tracing();
    let command = std::env::args().nth(1);

    let config = api_config::ApiConfig::load()?;

    let pool = api_services::connect_and_migrate(&config.database_url).await?;
    if config.migrate_only {
        info!("database migrations applied successfully");
        return Ok(());
    }

    if command.as_deref() == Some("seed-dev") {
        dev_seed::run(pool, &config).await?;
        return Ok(());
    }

    let app_state = api_services::build_app_state(pool.clone(), &config)?;
    qrywell_sync::spawn_qrywell_sync_worker(app_state.clone());
    let app = match config.session_store_backend {
        SessionStoreBackend::Postgres => {
            let session_layer =
                api_services::build_postgres_session_layer(pool.clone(), config.cookie_secure)
                    .await?;
            api_router::build_router(app_state, &config.frontend_url, session_layer)?
        }
        SessionStoreBackend::Redis => {
            let redis_url = config.redis_url.as_deref().ok_or_else(|| {
                AppError::Validation("REDIS_URL is required when SESSION_STORE=redis".to_owned())
            })?;
            let redis_client = api_services::build_redis_client(redis_url)?;
            let session_layer =
                api_services::build_redis_session_layer(redis_client, config.cookie_secure).await?;
            api_router::build_router(app_state, &config.frontend_url, session_layer)?
        }
    };
    let address = config.socket_address()?;

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| AppError::Internal(format!("failed to bind listener: {error}")))?;

    info!(%address, "qryvanta-api listening");

    axum::serve(listener, app)
        .await
        .map_err(|error| AppError::Internal(format!("api server error: {error}")))
}
