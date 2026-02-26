use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use qryvanta_core::AppError;
use tower_http::trace::TraceLayer;
use tower_sessions::{SessionManagerLayer, SessionStore};

use crate::state::AppState;
use crate::{auth, handlers, middleware};

mod cors;
mod protected;
mod public_auth;
mod worker_internal;

use cors::build_cors_layer;
use protected::build_protected_routes;
use public_auth::{
    build_forgot_password_routes, build_invite_accept_routes, build_login_routes,
    build_register_routes,
};
use worker_internal::build_worker_internal_routes;

pub fn build_router<S>(
    app_state: AppState,
    frontend_url: &str,
    session_layer: SessionManagerLayer<S>,
) -> Result<Router, AppError>
where
    S: SessionStore + Clone + Send + Sync + 'static,
{
    let protected_routes = build_protected_routes();
    let cors_layer = build_cors_layer(frontend_url)?;

    let login_routes = build_login_routes(app_state.clone());
    let register_routes = build_register_routes(app_state.clone());
    let forgot_password_routes = build_forgot_password_routes(app_state.clone());
    let invite_accept_routes = build_invite_accept_routes(app_state.clone());
    let worker_internal_routes = build_worker_internal_routes(app_state.clone());

    Ok(Router::new()
        .route("/health", get(handlers::health::health_handler))
        .route("/auth/bootstrap", post(auth::bootstrap_handler))
        .merge(login_routes)
        .merge(register_routes)
        .merge(forgot_password_routes)
        .merge(invite_accept_routes)
        .merge(worker_internal_routes)
        .route("/auth/verify-email", post(auth::verify_email_handler))
        .route("/auth/logout", post(auth::logout_handler))
        .merge(protected_routes)
        .route_layer(from_fn_with_state(
            app_state.clone(),
            middleware::require_same_origin_for_mutations,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(session_layer)
        .with_state(app_state))
}
