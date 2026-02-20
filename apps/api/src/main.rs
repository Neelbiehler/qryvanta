//! Qryvanta API composition root.

#![forbid(unsafe_code)]

mod auth;
mod dto;
mod error;
mod handlers;
mod middleware;
mod state;

use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::Router;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::{get, post};
use qryvanta_application::{MetadataService, TenantRepository};
use qryvanta_core::{AppError, TenantId};
use qryvanta_infrastructure::{
    PostgresMetadataRepository, PostgresPasskeyRepository, PostgresTenantRepository,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::info;
use tracing_subscriber::EnvFilter;
use url::Url;
use webauthn_rs::WebauthnBuilder;

use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    init_tracing();

    let database_url = required_env("DATABASE_URL")?;
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned());
    let bootstrap_token = required_env("AUTH_BOOTSTRAP_TOKEN")?;
    let session_secret = required_env("SESSION_SECRET")?;

    if session_secret.len() < 32 {
        return Err(AppError::Validation(
            "SESSION_SECRET must be at least 32 characters".to_owned(),
        ));
    }

    let api_host = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let api_port = env::var("API_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3001);

    let webauthn_rp_id = env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_owned());
    let webauthn_rp_origin =
        env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| frontend_url.clone());
    let cookie_secure = env::var("SESSION_COOKIE_SECURE")
        .unwrap_or_else(|_| "false".to_owned())
        .eq_ignore_ascii_case("true");
    let bootstrap_tenant_id = env::var("DEV_DEFAULT_TENANT_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            uuid::Uuid::parse_str(value.as_str())
                .map(TenantId::from_uuid)
                .map_err(|error| {
                    AppError::Validation(format!("invalid DEV_DEFAULT_TENANT_ID: {error}"))
                })
        })
        .transpose()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .map_err(|error| AppError::Internal(format!("failed to connect to database: {error}")))?;

    sqlx::migrate!("../../crates/infrastructure/migrations")
        .run(&pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to run migrations: {error}")))?;

    let session_store = PostgresStore::new(pool.clone())
        .with_table_name("tower_sessions")
        .map_err(|error| {
            AppError::Validation(format!("invalid session table name configuration: {error}"))
        })?;
    session_store.migrate().await.map_err(|error| {
        AppError::Internal(format!("failed to initialize session store: {error}"))
    })?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(cookie_secure)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(30)));

    let metadata_repository = Arc::new(PostgresMetadataRepository::new(pool.clone()));
    let tenant_repository: Arc<dyn TenantRepository> =
        Arc::new(PostgresTenantRepository::new(pool.clone()));
    let passkey_repository = PostgresPasskeyRepository::new(pool.clone());

    let webauthn_origin = Url::parse(&webauthn_rp_origin)
        .map_err(|error| AppError::Validation(format!("invalid WEBAUTHN_RP_ORIGIN: {error}")))?;
    let webauthn = Arc::new(
        WebauthnBuilder::new(&webauthn_rp_id, &webauthn_origin)
            .map_err(|error| {
                AppError::Validation(format!("invalid WebAuthn relying party config: {error}"))
            })?
            .rp_name("Qryvanta")
            .build()
            .map_err(|error| {
                AppError::Internal(format!("failed to initialize WebAuthn runtime: {error}"))
            })?,
    );

    let app_state = AppState {
        metadata_service: MetadataService::new(metadata_repository),
        tenant_repository,
        passkey_repository,
        webauthn,
        frontend_url: frontend_url.clone(),
        bootstrap_token,
        bootstrap_tenant_id,
    };

    let protected_routes = Router::new()
        .route(
            "/api/entities",
            get(handlers::entities::list_entities_handler)
                .post(handlers::entities::create_entity_handler),
        )
        .route("/auth/me", get(auth::me_handler))
        .route(
            "/auth/webauthn/register/start",
            post(auth::webauthn_registration_start_handler),
        )
        .route(
            "/auth/webauthn/register/finish",
            post(auth::webauthn_registration_finish_handler),
        )
        .route_layer(from_fn(middleware::require_auth));

    let cors_layer = CorsLayer::new()
        .allow_origin(
            HeaderValue::from_str(&frontend_url)
                .map_err(|error| AppError::Internal(format!("invalid FRONTEND_URL: {error}")))?,
        )
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(handlers::health::health_handler))
        .route("/auth/bootstrap", post(auth::bootstrap_handler))
        .route(
            "/auth/webauthn/login/start",
            get(auth::webauthn_login_start_handler),
        )
        .route(
            "/auth/webauthn/login/finish",
            post(auth::webauthn_login_finish_handler),
        )
        .route("/auth/logout", post(auth::logout_handler))
        .merge(protected_routes)
        .route_layer(from_fn_with_state(
            app_state.clone(),
            middleware::require_same_origin_for_mutations,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(session_layer)
        .with_state(app_state);

    let host = IpAddr::from_str(&api_host)
        .map_err(|error| AppError::Internal(format!("invalid API_HOST '{api_host}': {error}")))?;
    let address = SocketAddr::from((host, api_port));

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| AppError::Internal(format!("failed to bind listener: {error}")))?;

    info!(%address, "qryvanta-api listening");

    axum::serve(listener, app)
        .await
        .map_err(|error| AppError::Internal(format!("api server error: {error}")))
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn required_env(name: &str) -> Result<String, AppError> {
    env::var(name).map_err(|_| AppError::Validation(format!("{name} is required")))
}
