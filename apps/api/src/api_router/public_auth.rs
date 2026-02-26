use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use qryvanta_application::RateLimitRule;

use crate::state::AppState;
use crate::{auth, middleware};

pub(super) fn build_login_routes(app_state: AppState) -> Router<AppState> {
    let login_rate_rule = RateLimitRule::new("login", 10, 15 * 60);

    Router::new()
        .route("/auth/login", post(auth::login_handler))
        .route("/auth/login/mfa", post(auth::mfa_verify_handler))
        .route(
            "/auth/webauthn/login/start",
            get(auth::webauthn_login_start_handler),
        )
        .route(
            "/auth/webauthn/login/finish",
            post(auth::webauthn_login_finish_handler),
        )
        .route_layer(from_fn_with_state(app_state, middleware::rate_limit))
        .layer(axum::Extension(login_rate_rule))
}

pub(super) fn build_register_routes(app_state: AppState) -> Router<AppState> {
    let register_rate_rule = RateLimitRule::new("register", 5, 60 * 60);

    Router::new()
        .route("/auth/register", post(auth::register_handler))
        .route_layer(from_fn_with_state(app_state, middleware::rate_limit))
        .layer(axum::Extension(register_rate_rule))
}

pub(super) fn build_forgot_password_routes(app_state: AppState) -> Router<AppState> {
    let forgot_password_rate_rule = RateLimitRule::new("forgot_password", 5, 60 * 60);

    Router::new()
        .route("/auth/forgot-password", post(auth::forgot_password_handler))
        .route("/auth/reset-password", post(auth::reset_password_handler))
        .route_layer(from_fn_with_state(app_state, middleware::rate_limit))
        .layer(axum::Extension(forgot_password_rate_rule))
}

pub(super) fn build_invite_accept_routes(app_state: AppState) -> Router<AppState> {
    let invite_accept_rate_rule = RateLimitRule::new("invite_accept", 10, 60 * 60);

    Router::new()
        .route("/auth/invite/accept", post(auth::accept_invite_handler))
        .route_layer(from_fn_with_state(app_state, middleware::rate_limit))
        .layer(axum::Extension(invite_accept_rate_rule))
}
