//! Infrastructure adapters for application ports.

#![forbid(unsafe_code)]

mod in_memory_metadata_repository;
mod postgres_metadata_repository;
mod postgres_passkey_repository;
mod postgres_tenant_repository;

pub use in_memory_metadata_repository::InMemoryMetadataRepository;
pub use postgres_metadata_repository::PostgresMetadataRepository;
pub use postgres_passkey_repository::PostgresPasskeyRepository;
pub use postgres_tenant_repository::PostgresTenantRepository;
