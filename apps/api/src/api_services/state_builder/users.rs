use std::sync::Arc;

use qryvanta_application::{
    AuthEventService, AuthTokenService, AuthorizationService, MfaService, TenantAccessService,
    TenantRepository, UserService,
};
use qryvanta_core::AppError;
use qryvanta_infrastructure::{
    AesSecretEncryptor, Argon2PasswordHasher, AwsKmsEnvelopeSecretEncryptor,
    PostgresAuthTokenRepository, PostgresUserRepository, TotpRsProvider,
};
use sqlx::PgPool;

use crate::api_config::{ApiConfig, TotpEncryptionConfig};

use super::super::email::build_email_service;

pub(super) struct UserServices {
    pub(super) user_service: UserService,
    pub(super) tenant_access_service: TenantAccessService,
    pub(super) auth_token_service: AuthTokenService,
    pub(super) mfa_service: MfaService,
}

pub(super) fn build_user_services(
    pool: &PgPool,
    config: &ApiConfig,
    tenant_repository: Arc<dyn TenantRepository>,
    user_repository: Arc<PostgresUserRepository>,
    authorization_service: AuthorizationService,
    auth_event_service: AuthEventService,
) -> Result<UserServices, AppError> {
    let password_hasher = Arc::new(Argon2PasswordHasher::new());

    let user_service = UserService::new(
        user_repository.clone(),
        password_hasher.clone(),
        tenant_repository.clone(),
        auth_event_service,
    );
    let tenant_access_service = TenantAccessService::new(
        tenant_repository,
        user_repository.clone(),
        authorization_service,
    );

    let auth_token_repository = Arc::new(PostgresAuthTokenRepository::new(pool.clone()));
    let email_service = build_email_service(config)?;
    let auth_token_service = AuthTokenService::new(
        auth_token_repository,
        email_service,
        config.frontend_url.clone(),
    );

    let totp_provider = Arc::new(TotpRsProvider::new("Qryvanta"));
    let secret_encryptor: Arc<dyn qryvanta_application::SecretEncryptor> =
        match &config.totp_encryption {
            TotpEncryptionConfig::StaticKey { key_hex } => {
                Arc::new(AesSecretEncryptor::from_hex(key_hex)?)
            }
            TotpEncryptionConfig::AwsKmsEnvelope {
                kms_key_id,
                legacy_static_key_hex,
            } => Arc::new(AwsKmsEnvelopeSecretEncryptor::new(
                kms_key_id,
                legacy_static_key_hex.as_deref(),
            )?),
        };
    let mfa_service = MfaService::new(
        user_repository,
        password_hasher,
        totp_provider,
        secret_encryptor,
    );

    Ok(UserServices {
        user_service,
        tenant_access_service,
        auth_token_service,
        mfa_service,
    })
}
