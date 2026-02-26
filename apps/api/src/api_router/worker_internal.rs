use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};

use crate::state::AppState;
use crate::{handlers, middleware};

pub(super) fn build_worker_internal_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/api/internal/worker/jobs/claim",
            post(handlers::worker::claim_workflow_jobs_handler),
        )
        .route(
            "/api/internal/worker/heartbeat",
            post(handlers::worker::worker_heartbeat_handler),
        )
        .route(
            "/api/internal/worker/jobs/stats",
            get(handlers::worker::workflow_queue_stats_handler),
        )
        .route_layer(from_fn_with_state(
            app_state,
            middleware::require_worker_auth,
        ))
}
