use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::Permission;

use crate::AuditRepository;

/// Runtime field-level grant row resolved for one subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldGrant {
    /// Field logical name.
    pub field_logical_name: String,
    /// Read access for the field.
    pub can_read: bool,
    /// Write access for the field.
    pub can_write: bool,
}

/// Effective runtime field access resolved for one subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldAccess {
    /// Fields readable by the subject.
    pub readable_fields: std::collections::BTreeSet<String>,
    /// Fields writable by the subject.
    pub writable_fields: std::collections::BTreeSet<String>,
}

/// Active temporary permission grant projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryPermissionGrant {
    /// Stable grant id.
    pub grant_id: String,
    /// Human-readable reason captured during grant.
    pub reason: String,
    /// Grant expiry timestamp in RFC3339.
    pub expires_at: String,
}

/// Repository port for permission lookups.
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    /// Lists effective permissions for a subject in a tenant.
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>>;

    /// Lists explicit field-level runtime grants for a subject and entity.
    async fn list_runtime_field_grants_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>>;

    /// Finds an active temporary grant for a specific permission.
    async fn find_active_temporary_permission_grant(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>>;
}

/// Application service for tenant-scoped authorization checks.
#[derive(Clone)]
pub struct AuthorizationService {
    repository: Arc<dyn AuthorizationRepository>,
    audit_repository: Arc<dyn AuditRepository>,
}

impl AuthorizationService {
    /// Creates a new authorization service from a repository implementation.
    #[must_use]
    pub fn new(
        repository: Arc<dyn AuthorizationRepository>,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            repository,
            audit_repository,
        }
    }
}

enum PermissionGrantResolution {
    Granted,
    Temporary(TemporaryPermissionGrant),
    Missing,
}

mod permissions;
mod runtime_fields;
mod surfaces;

#[cfg(test)]
mod tests;
