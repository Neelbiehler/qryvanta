//! Domain entities and invariants.

#![forbid(unsafe_code)]

mod metadata;
mod security;

pub use metadata::EntityDefinition;
pub use security::{AuditAction, Permission};
