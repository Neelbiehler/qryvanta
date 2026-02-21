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
use axum::routing::{delete, get, post, put};
use qryvanta_application::{
    AppService, AuthEventService, AuthTokenService, AuthorizationService, EmailService,
    MetadataService, MfaService, RateLimitRule, RateLimitService, SecurityAdminService,
    TenantRepository, UserService,
};
use qryvanta_core::{AppError, TenantId};
use qryvanta_infrastructure::{
    AesSecretEncryptor, Argon2PasswordHasher, ConsoleEmailService, PostgresAppRepository,
    PostgresAuditLogRepository, PostgresAuditRepository, PostgresAuthEventRepository,
    PostgresAuthTokenRepository, PostgresAuthorizationRepository, PostgresMetadataRepository,
    PostgresPasskeyRepository, PostgresRateLimitRepository, PostgresSecurityAdminRepository,
    PostgresTenantRepository, PostgresUserRepository, SmtpEmailConfig, SmtpEmailService,
    TotpRsProvider,
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

    let migrate_only = env::args().nth(1).as_deref() == Some("migrate");

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

    let totp_encryption_key = env::var("TOTP_ENCRYPTION_KEY").unwrap_or_else(|_| "0".repeat(64));

    let email_provider = env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "console".to_owned());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .map_err(|error| AppError::Internal(format!("failed to connect to database: {error}")))?;

    sqlx::migrate!("../../crates/infrastructure/migrations")
        .run(&pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to run migrations: {error}")))?;

    if migrate_only {
        info!("database migrations applied successfully");
        return Ok(());
    }

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
    let app_repository = Arc::new(PostgresAppRepository::new(pool.clone()));
    let authorization_repository = Arc::new(PostgresAuthorizationRepository::new(pool.clone()));
    let authorization_service = AuthorizationService::new(authorization_repository);
    let audit_repository = Arc::new(PostgresAuditRepository::new(pool.clone()));
    let security_admin_repository = Arc::new(PostgresSecurityAdminRepository::new(pool.clone()));
    let audit_log_repository = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let security_admin_service = SecurityAdminService::new(
        authorization_service.clone(),
        security_admin_repository,
        audit_log_repository,
        audit_repository.clone(),
    );
    let auth_event_repository = Arc::new(PostgresAuthEventRepository::new(pool.clone()));
    let auth_event_service = AuthEventService::new(auth_event_repository);
    let tenant_repository: Arc<dyn TenantRepository> =
        Arc::new(PostgresTenantRepository::new(pool.clone()));
    let passkey_repository = PostgresPasskeyRepository::new(pool.clone());

    // User and auth services.
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let user_service = UserService::new(
        user_repository.clone(),
        password_hasher.clone(),
        tenant_repository.clone(),
        auth_event_service.clone(),
    );

    // Auth token and email services.
    let auth_token_repository = Arc::new(PostgresAuthTokenRepository::new(pool.clone()));
    let email_service: Arc<dyn EmailService> = match email_provider.as_str() {
        "smtp" => {
            let smtp_port = required_non_empty_env("SMTP_PORT")?
                .parse::<u16>()
                .map_err(|error| AppError::Validation(format!("invalid SMTP_PORT: {error}")))?;

            let smtp_config = SmtpEmailConfig {
                host: required_non_empty_env("SMTP_HOST")?,
                port: smtp_port,
                username: required_non_empty_env("SMTP_USERNAME")?,
                password: required_non_empty_env("SMTP_PASSWORD")?,
                from_address: required_non_empty_env("SMTP_FROM_ADDRESS")?,
            };
            Arc::new(SmtpEmailService::new(smtp_config)?)
        }
        "console" => Arc::new(ConsoleEmailService::new()),
        _ => {
            return Err(AppError::Validation(format!(
                "EMAIL_PROVIDER must be either 'console' or 'smtp', got '{email_provider}'"
            )));
        }
    };

    let auth_token_service =
        AuthTokenService::new(auth_token_repository, email_service, frontend_url.clone());

    // MFA services.
    let totp_provider = Arc::new(TotpRsProvider::new("Qryvanta"));
    let secret_encryptor = Arc::new(
        AesSecretEncryptor::from_hex(&totp_encryption_key).unwrap_or_else(|_| {
            AesSecretEncryptor::from_hex(&"0".repeat(64))
                .unwrap_or_else(|_| AesSecretEncryptor::new(&[0u8; 32]))
        }),
    );
    let mfa_service = MfaService::new(
        user_repository,
        password_hasher,
        totp_provider,
        secret_encryptor,
    );

    // Rate limiting service.
    let rate_limit_repository = Arc::new(PostgresRateLimitRepository::new(pool.clone()));
    let rate_limit_service = RateLimitService::new(rate_limit_repository);

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
        app_service: AppService::new(
            authorization_service.clone(),
            app_repository,
            Arc::new(MetadataService::new(
                metadata_repository.clone(),
                authorization_service.clone(),
                audit_repository.clone(),
            )),
            audit_repository.clone(),
        ),
        metadata_service: MetadataService::new(
            metadata_repository,
            authorization_service.clone(),
            audit_repository,
        ),
        security_admin_service,
        authorization_service,
        auth_event_service,
        user_service,
        auth_token_service,
        mfa_service,
        rate_limit_service,
        tenant_repository,
        passkey_repository,
        webauthn,
        frontend_url: frontend_url.clone(),
        bootstrap_token,
        bootstrap_tenant_id,
    };

    let protected_routes = Router::new()
        .route(
            "/api/apps",
            get(handlers::apps::list_apps_handler).post(handlers::apps::create_app_handler),
        )
        .route(
            "/api/apps/{app_logical_name}/entities",
            get(handlers::apps::list_app_entities_handler)
                .post(handlers::apps::bind_app_entity_handler),
        )
        .route(
            "/api/apps/{app_logical_name}/permissions",
            get(handlers::apps::list_app_role_permissions_handler)
                .put(handlers::apps::save_app_role_permission_handler),
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
            "/api/workspace/apps/{app_logical_name}/entities/{entity_logical_name}/records/{record_id}",
            get(handlers::apps::workspace_get_record_handler)
                .put(handlers::apps::workspace_update_record_handler)
                .delete(handlers::apps::workspace_delete_record_handler),
        )
        .route(
            "/api/entities",
            get(handlers::entities::list_entities_handler)
                .post(handlers::entities::create_entity_handler),
        )
        .route(
            "/api/entities/{entity_logical_name}/fields",
            get(handlers::entities::list_fields_handler)
                .post(handlers::entities::save_field_handler),
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
            get(handlers::security::list_roles_handler)
                .post(handlers::security::create_role_handler),
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
            "/api/security/registration-mode",
            get(handlers::security::registration_mode_handler)
                .put(handlers::security::update_registration_mode_handler),
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
        // Password change (requires auth).
        .route("/api/profile/password", put(auth::change_password_handler))
        // MFA management (requires auth).
        .route("/auth/mfa/totp/enroll", post(auth::mfa_enroll_handler))
        .route("/auth/mfa/totp/confirm", post(auth::mfa_confirm_handler))
        .route("/auth/mfa/totp", delete(auth::mfa_disable_handler))
        .route(
            "/auth/mfa/recovery-codes/regenerate",
            post(auth::mfa_regenerate_recovery_codes_handler),
        )
        // Email verification (resend, requires auth).
        .route(
            "/auth/resend-verification",
            post(auth::resend_verification_handler),
        )
        .route("/auth/invite", post(auth::send_invite_handler))
        .route_layer(from_fn(middleware::require_auth));

    let cors_layer = CorsLayer::new()
        .allow_origin(
            HeaderValue::from_str(&frontend_url)
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

    // Rate limit rules (OWASP Credential Stuffing Prevention).
    // Login: 10 attempts per IP per 15 minutes.
    let login_rate_rule = RateLimitRule::new("login", 10, 15 * 60);
    // Registration: 5 attempts per IP per hour.
    let register_rate_rule = RateLimitRule::new("register", 5, 60 * 60);
    // Password reset: 5 attempts per IP per hour.
    let forgot_password_rate_rule = RateLimitRule::new("forgot_password", 5, 60 * 60);
    // Invite acceptance: 10 attempts per IP per hour.
    let invite_accept_rate_rule = RateLimitRule::new("invite_accept", 10, 60 * 60);

    // Rate-limited auth routes: login.
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

    // Rate-limited auth routes: registration.
    let register_routes = Router::new()
        .route("/auth/register", post(auth::register_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit,
        ))
        .layer(axum::Extension(register_rate_rule));

    // Rate-limited auth routes: forgot password.
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

    let app = Router::new()
        .route("/health", get(handlers::health::health_handler))
        .route("/auth/bootstrap", post(auth::bootstrap_handler))
        .merge(login_routes)
        .merge(register_routes)
        .merge(forgot_password_routes)
        .merge(invite_accept_routes)
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

fn required_non_empty_env(name: &str) -> Result<String, AppError> {
    let value = required_env(name)?;
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{name} must not be empty")));
    }

    Ok(value)
}
