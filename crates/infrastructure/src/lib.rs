//! Infrastructure adapters for application ports.

#![forbid(unsafe_code)]

mod in_memory_metadata_repository;
mod postgres_audit_log_repository;
mod postgres_audit_repository;
mod postgres_auth_event_repository;
mod postgres_authorization_repository;
mod postgres_metadata_repository;
mod postgres_passkey_repository;
mod postgres_security_admin_repository;
mod postgres_tenant_repository;

pub use in_memory_metadata_repository::InMemoryMetadataRepository;
pub use postgres_audit_log_repository::PostgresAuditLogRepository;
pub use postgres_audit_repository::PostgresAuditRepository;
pub use postgres_auth_event_repository::PostgresAuthEventRepository;
pub use postgres_authorization_repository::PostgresAuthorizationRepository;
pub use postgres_metadata_repository::PostgresMetadataRepository;
pub use postgres_passkey_repository::PostgresPasskeyRepository;
pub use postgres_security_admin_repository::PostgresSecurityAdminRepository;
pub use postgres_tenant_repository::PostgresTenantRepository;
