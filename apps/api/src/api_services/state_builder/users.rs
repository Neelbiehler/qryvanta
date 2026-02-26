use std::sync::Arc;

use qryvanta_application::{
    AuthEventService, AuthTokenService, MfaService, TenantRepository, UserService,
};
use qryvanta_core::AppError;
use qryvanta_infrastructure::{
    AesSecretEncryptor, Argon2PasswordHasher, PostgresAuthTokenRepository, PostgresUserRepository,
    TotpRsProvider,
};
use sqlx::PgPool;

use crate::api_config::ApiConfig;

use super::super::email::build_email_service;

pub(super) struct UserServices {
    pub(super) user_service: UserService,
    pub(super) auth_token_service: AuthTokenService,
    pub(super) mfa_service: MfaService,
}

pub(super) fn build_user_services(
    pool: &PgPool,
    config: &ApiConfig,
    tenant_repository: Arc<dyn TenantRepository>,
    user_repository: Arc<PostgresUserRepository>,
    auth_event_service: AuthEventService,
) -> Result<UserServices, AppError> {
    let password_hasher = Arc::new(Argon2PasswordHasher::new());

    let user_service = UserService::new(
        user_repository.clone(),
        password_hasher.clone(),
        tenant_repository,
        auth_event_service,
    );

    let auth_token_repository = Arc::new(PostgresAuthTokenRepository::new(pool.clone()));
    let email_service = build_email_service(config)?;
    let auth_token_service = AuthTokenService::new(
        auth_token_repository,
        email_service,
        config.frontend_url.clone(),
    );

    let totp_provider = Arc::new(TotpRsProvider::new("Qryvanta"));
    let secret_encryptor = Arc::new(build_secret_encryptor(config));
    let mfa_service = MfaService::new(
        user_repository,
        password_hasher,
        totp_provider,
        secret_encryptor,
    );

    Ok(UserServices {
        user_service,
        auth_token_service,
        mfa_service,
    })
}

fn build_secret_encryptor(config: &ApiConfig) -> AesSecretEncryptor {
    AesSecretEncryptor::from_hex(&config.totp_encryption_key).unwrap_or_else(|_| {
        AesSecretEncryptor::from_hex(&"0".repeat(64))
            .unwrap_or_else(|_| AesSecretEncryptor::new(&[0_u8; 32]))
    })
}
