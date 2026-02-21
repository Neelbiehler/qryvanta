//! Qryvanta API composition root.

#![forbid(unsafe_code)]

mod api_config;
mod api_router;
mod api_services;
mod auth;
mod dto;
mod error;
mod handlers;
mod middleware;
mod state;

use qryvanta_core::AppError;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    api_config::init_tracing();

    let config = api_config::ApiConfig::load()?;

    let pool = api_services::connect_and_migrate(&config.database_url).await?;
    if config.migrate_only {
        info!("database migrations applied successfully");
        return Ok(());
    }

    let session_layer =
        api_services::build_session_layer(pool.clone(), config.cookie_secure).await?;
    let app_state = api_services::build_app_state(pool, &config)?;
    let app = api_router::build_router(app_state, &config.frontend_url, session_layer)?;
    let address = config.socket_address()?;

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| AppError::Internal(format!("failed to bind listener: {error}")))?;

    info!(%address, "qryvanta-api listening");

    axum::serve(listener, app)
        .await
        .map_err(|error| AppError::Internal(format!("api server error: {error}")))
}
