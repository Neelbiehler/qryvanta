use std::sync::Arc;

use qryvanta_application::{
    AppService, AuthEventService, AuthTokenService, AuthorizationService, EmailService,
    MetadataService, MfaService, TenantRepository, UserService,
};
use qryvanta_core::AppError;
use qryvanta_infrastructure::{
    AesSecretEncryptor, Argon2PasswordHasher, ConsoleEmailService, PostgresAppRepository,
    PostgresAuditLogRepository, PostgresAuditRepository, PostgresAuthEventRepository,
    PostgresAuthTokenRepository, PostgresAuthorizationRepository, PostgresMetadataRepository,
    PostgresPasskeyRepository, PostgresRateLimitRepository, PostgresSecurityAdminRepository,
    PostgresTenantRepository, PostgresUserRepository, SmtpEmailConfig, SmtpEmailService,
    TotpRsProvider,
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use url::Url;
use webauthn_rs::WebauthnBuilder;

use crate::api_config::{ApiConfig, EmailProviderConfig};
use crate::state::AppState;

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|error| AppError::Internal(format!("failed to connect to database: {error}")))?;

    sqlx::migrate!("../../crates/infrastructure/migrations")
        .run(&pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to run migrations: {error}")))?;

    Ok(pool)
}

pub async fn build_session_layer(
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

pub fn build_app_state(pool: PgPool, config: &ApiConfig) -> Result<AppState, AppError> {
    let metadata_repository = Arc::new(PostgresMetadataRepository::new(pool.clone()));
    let app_repository = Arc::new(PostgresAppRepository::new(pool.clone()));
    let authorization_repository = Arc::new(PostgresAuthorizationRepository::new(pool.clone()));
    let authorization_service = AuthorizationService::new(authorization_repository);
    let audit_repository = Arc::new(PostgresAuditRepository::new(pool.clone()));
    let security_admin_repository = Arc::new(PostgresSecurityAdminRepository::new(pool.clone()));
    let audit_log_repository = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let security_admin_service = qryvanta_application::SecurityAdminService::new(
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

    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let user_service = UserService::new(
        user_repository.clone(),
        password_hasher.clone(),
        tenant_repository.clone(),
        auth_event_service.clone(),
    );

    let auth_token_repository = Arc::new(PostgresAuthTokenRepository::new(pool.clone()));
    let email_service = build_email_service(config)?;
    let auth_token_service = AuthTokenService::new(
        auth_token_repository,
        email_service,
        config.frontend_url.clone(),
    );

    let totp_provider = Arc::new(TotpRsProvider::new("Qryvanta"));
    let secret_encryptor = Arc::new(
        AesSecretEncryptor::from_hex(&config.totp_encryption_key).unwrap_or_else(|_| {
            AesSecretEncryptor::from_hex(&"0".repeat(64))
                .unwrap_or_else(|_| AesSecretEncryptor::new(&[0_u8; 32]))
        }),
    );
    let mfa_service = MfaService::new(
        user_repository,
        password_hasher,
        totp_provider,
        secret_encryptor,
    );

    let rate_limit_repository = Arc::new(PostgresRateLimitRepository::new(pool.clone()));
    let rate_limit_service = qryvanta_application::RateLimitService::new(rate_limit_repository);

    let webauthn_origin = Url::parse(&config.webauthn_rp_origin)
        .map_err(|error| AppError::Validation(format!("invalid WEBAUTHN_RP_ORIGIN: {error}")))?;
    let webauthn = Arc::new(
        WebauthnBuilder::new(&config.webauthn_rp_id, &webauthn_origin)
            .map_err(|error| {
                AppError::Validation(format!("invalid WebAuthn relying party config: {error}"))
            })?
            .rp_name("Qryvanta")
            .build()
            .map_err(|error| {
                AppError::Internal(format!("failed to initialize WebAuthn runtime: {error}"))
            })?,
    );

    Ok(AppState {
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
        frontend_url: config.frontend_url.clone(),
        bootstrap_token: config.bootstrap_token.clone(),
        bootstrap_tenant_id: config.bootstrap_tenant_id,
    })
}

fn build_email_service(config: &ApiConfig) -> Result<Arc<dyn EmailService>, AppError> {
    let service: Arc<dyn EmailService> = match &config.email_provider {
        EmailProviderConfig::Console => Arc::new(ConsoleEmailService::new()),
        EmailProviderConfig::Smtp(smtp) => {
            let smtp_config = SmtpEmailConfig {
                host: smtp.host.clone(),
                port: smtp.port,
                username: smtp.username.clone(),
                password: smtp.password.clone(),
                from_address: smtp.from_address.clone(),
            };
            Arc::new(SmtpEmailService::new(smtp_config)?)
        }
    };

    Ok(service)
}
