//! Application services and ports.

#![forbid(unsafe_code)]

mod metadata_service;

pub use metadata_service::{MetadataRepository, MetadataService, TenantRepository};
