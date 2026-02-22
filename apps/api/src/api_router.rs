use axum::Router;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::{delete, get, post, put};
use qryvanta_application::RateLimitRule;
use qryvanta_core::AppError;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use crate::state::AppState;
use crate::{auth, handlers, middleware};

pub fn build_router(
    app_state: AppState,
    frontend_url: &str,
    session_layer: SessionManagerLayer<PostgresStore>,
) -> Result<Router, AppError> {
    let protected_routes = Router::new()
        .route(
            "/api/apps",
            get(handlers::apps::list_apps_handler).post(handlers::apps::create_app_handler),
        )
        .route(
            "/api/apps/{app_logical_name}/entities",
            get(handlers::apps::list_app_entities_handler).post(handlers::apps::bind_app_entity_handler),
        )
        .route(
            "/api/apps/{app_logical_name}/permissions",
            get(handlers::apps::list_app_role_permissions_handler)
                .put(handlers::apps::save_app_role_permission_handler),
        )
        .route(
            "/api/workflows",
            get(handlers::workflows::list_workflows_handler)
                .post(handlers::workflows::save_workflow_handler),
        )
        .route(
            "/api/workflows/runs",
            get(handlers::workflows::list_workflow_runs_handler),
        )
        .route(
            "/api/workflows/runs/{run_id}/attempts",
            get(handlers::workflows::list_workflow_run_attempts_handler),
        )
        .route(
            "/api/workflows/{workflow_logical_name}/execute",
            post(handlers::workflows::execute_workflow_handler),
        )
        .route(
            "/api/workspace/apps",
            get(handlers::apps::list_workspace_apps_handler),
        )
        .route(
            "/api/workspace/apps/{app_logical_name}/navigation",
            get(handlers::apps::app_navigation_handler),
        )
        .route(
            "/api/workspace/apps/{app_logical_name}/entities/{entity_logical_name}/schema",
            get(handlers::apps::workspace_entity_schema_handler),
        )
        .route(
            "/api/workspace/apps/{app_logical_name}/entities/{entity_logical_name}/capabilities",
            get(handlers::apps::workspace_entity_capabilities_handler),
        )
        .route(
            "/api/workspace/apps/{app_logical_name}/entities/{entity_logical_name}/records",
            get(handlers::apps::workspace_list_records_handler)
                .post(handlers::apps::workspace_create_record_handler),
        )
        .route(
            "/api/workspace/apps/{app_logical_name}/entities/{entity_logical_name}/records/query",
            post(handlers::apps::workspace_query_records_handler),
        )
        .route(
            "/api/workspace/apps/{app_logical_name}/entities/{entity_logical_name}/records/{record_id}",
            get(handlers::apps::workspace_get_record_handler)
                .put(handlers::apps::workspace_update_record_handler)
                .delete(handlers::apps::workspace_delete_record_handler),
        )
        .route(
            "/api/entities",
            get(handlers::entities::list_entities_handler).post(handlers::entities::create_entity_handler),
        )
        .route(
            "/api/entities/{entity_logical_name}/fields",
            get(handlers::entities::list_fields_handler).post(handlers::entities::save_field_handler),
        )
        .route(
            "/api/entities/{entity_logical_name}/publish",
            post(handlers::entities::publish_entity_handler),
        )
        .route(
            "/api/entities/{entity_logical_name}/published",
            get(handlers::entities::latest_published_schema_handler),
        )
        .route(
            "/api/runtime/{entity_logical_name}/records",
            get(handlers::runtime::list_runtime_records_handler)
                .post(handlers::runtime::create_runtime_record_handler),
        )
        .route(
            "/api/runtime/{entity_logical_name}/records/query",
            post(handlers::runtime::query_runtime_records_handler),
        )
        .route(
            "/api/runtime/{entity_logical_name}/records/{record_id}",
            get(handlers::runtime::get_runtime_record_handler)
                .put(handlers::runtime::update_runtime_record_handler)
                .delete(handlers::runtime::delete_runtime_record_handler),
        )
        .route(
            "/api/security/roles",
            get(handlers::security::list_roles_handler).post(handlers::security::create_role_handler),
        )
        .route(
            "/api/security/role-assignments",
            get(handlers::security::list_role_assignments_handler)
                .post(handlers::security::assign_role_handler),
        )
        .route(
            "/api/security/role-unassignments",
            post(handlers::security::unassign_role_handler),
        )
        .route(
            "/api/security/audit-log",
            get(handlers::security::list_audit_log_handler),
        )
        .route(
            "/api/security/audit-log/export",
            get(handlers::security::export_audit_log_handler),
        )
        .route(
            "/api/security/audit-log/purge",
            post(handlers::security::purge_audit_log_handler),
        )
        .route(
            "/api/security/registration-mode",
            get(handlers::security::registration_mode_handler)
                .put(handlers::security::update_registration_mode_handler),
        )
        .route(
            "/api/security/audit-retention-policy",
            get(handlers::security::audit_retention_policy_handler)
                .put(handlers::security::update_audit_retention_policy_handler),
        )
        .route(
            "/api/security/runtime-field-permissions",
            get(handlers::security::list_runtime_field_permissions_handler)
                .put(handlers::security::save_runtime_field_permissions_handler),
        )
        .route(
            "/api/security/temporary-access-grants",
            get(handlers::security::list_temporary_access_grants_handler)
                .post(handlers::security::create_temporary_access_grant_handler),
        )
        .route(
            "/api/security/temporary-access-grants/{grant_id}/revoke",
            post(handlers::security::revoke_temporary_access_grant_handler),
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
        .route("/api/profile/password", put(auth::change_password_handler))
        .route("/auth/mfa/totp/enroll", post(auth::mfa_enroll_handler))
        .route("/auth/mfa/totp/confirm", post(auth::mfa_confirm_handler))
        .route("/auth/mfa/totp", delete(auth::mfa_disable_handler))
        .route(
            "/auth/mfa/recovery-codes/regenerate",
            post(auth::mfa_regenerate_recovery_codes_handler),
        )
        .route(
            "/auth/resend-verification",
            post(auth::resend_verification_handler),
        )
        .route("/auth/invite", post(auth::send_invite_handler))
        .route_layer(from_fn(middleware::require_auth));

    let cors_layer = CorsLayer::new()
        .allow_origin(
            HeaderValue::from_str(frontend_url)
                .map_err(|error| AppError::Internal(format!("invalid FRONTEND_URL: {error}")))?,
        )
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE]);

    let login_rate_rule = RateLimitRule::new("login", 10, 15 * 60);
    let register_rate_rule = RateLimitRule::new("register", 5, 60 * 60);
    let forgot_password_rate_rule = RateLimitRule::new("forgot_password", 5, 60 * 60);
    let invite_accept_rate_rule = RateLimitRule::new("invite_accept", 10, 60 * 60);

    let login_routes = Router::new()
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
        .route_layer(from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit,
        ))
        .layer(axum::Extension(login_rate_rule));

    let register_routes = Router::new()
        .route("/auth/register", post(auth::register_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit,
        ))
        .layer(axum::Extension(register_rate_rule));

    let forgot_password_routes = Router::new()
        .route("/auth/forgot-password", post(auth::forgot_password_handler))
        .route("/auth/reset-password", post(auth::reset_password_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit,
        ))
        .layer(axum::Extension(forgot_password_rate_rule));

    let invite_accept_routes = Router::new()
        .route("/auth/invite/accept", post(auth::accept_invite_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit,
        ))
        .layer(axum::Extension(invite_accept_rate_rule));

    let worker_internal_routes = Router::new()
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
            app_state.clone(),
            middleware::require_worker_auth,
        ));

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
