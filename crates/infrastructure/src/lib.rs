//! Infrastructure adapters for application ports.

#![forbid(unsafe_code)]

mod aes_secret_encryptor;
mod argon2_password_hasher;
mod console_email_service;
mod in_memory_metadata_repository;
mod postgres_audit_log_repository;
mod postgres_audit_repository;
mod postgres_auth_event_repository;
mod postgres_auth_token_repository;
mod postgres_authorization_repository;
mod postgres_metadata_repository;
mod postgres_passkey_repository;
mod postgres_rate_limit_repository;
mod postgres_security_admin_repository;
mod postgres_tenant_repository;
mod postgres_user_repository;
mod smtp_email_service;
mod totp_provider;

pub use aes_secret_encryptor::AesSecretEncryptor;
pub use argon2_password_hasher::Argon2PasswordHasher;
pub use console_email_service::ConsoleEmailService;
pub use in_memory_metadata_repository::InMemoryMetadataRepository;
pub use postgres_audit_log_repository::PostgresAuditLogRepository;
pub use postgres_audit_repository::PostgresAuditRepository;
pub use postgres_auth_event_repository::PostgresAuthEventRepository;
pub use postgres_auth_token_repository::PostgresAuthTokenRepository;
pub use postgres_authorization_repository::PostgresAuthorizationRepository;
pub use postgres_metadata_repository::PostgresMetadataRepository;
pub use postgres_passkey_repository::PostgresPasskeyRepository;
pub use postgres_rate_limit_repository::PostgresRateLimitRepository;
pub use postgres_security_admin_repository::PostgresSecurityAdminRepository;
pub use postgres_tenant_repository::PostgresTenantRepository;
pub use postgres_user_repository::PostgresUserRepository;
pub use smtp_email_service::{SmtpEmailConfig, SmtpEmailService};
pub use totp_provider::TotpRsProvider;
