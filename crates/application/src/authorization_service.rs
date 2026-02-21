use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{AuditAction, Permission};

use crate::{AuditEvent, AuditRepository};

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

    /// Ensures a subject has the required permission in the tenant scope.
    pub async fn require_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<()> {
        match self
            .resolve_permission_grant(tenant_id, subject, permission)
            .await?
        {
            PermissionGrantResolution::Granted => Ok(()),
            PermissionGrantResolution::Temporary(grant) => {
                self.append_temporary_access_use_event(tenant_id, subject, permission, &grant)
                    .await
            }
            PermissionGrantResolution::Missing => Err(AppError::Forbidden(format!(
                "subject '{subject}' is missing permission '{}' in tenant '{tenant_id}'",
                permission.as_str()
            ))),
        }
    }

    /// Returns whether the subject currently has the permission.
    pub async fn has_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<bool> {
        match self
            .resolve_permission_grant(tenant_id, subject, permission)
            .await?
        {
            PermissionGrantResolution::Granted => Ok(true),
            PermissionGrantResolution::Temporary(grant) => {
                self.append_temporary_access_use_event(tenant_id, subject, permission, &grant)
                    .await?;
                Ok(true)
            }
            PermissionGrantResolution::Missing => Ok(false),
        }
    }

    /// Returns effective field-level runtime access for a subject and entity.
    pub async fn runtime_field_access(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Option<RuntimeFieldAccess>> {
        let grants = self
            .repository
            .list_runtime_field_grants_for_subject(tenant_id, subject, entity_logical_name)
            .await?;

        if grants.is_empty() {
            return Ok(None);
        }

        let mut readable_fields = std::collections::BTreeSet::new();
        let mut writable_fields = std::collections::BTreeSet::new();

        for grant in grants {
            if grant.can_read {
                readable_fields.insert(grant.field_logical_name.clone());
            }
            if grant.can_write {
                writable_fields.insert(grant.field_logical_name);
            }
        }

        Ok(Some(RuntimeFieldAccess {
            readable_fields,
            writable_fields,
        }))
    }

    async fn resolve_permission_grant(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<PermissionGrantResolution> {
        let permissions = self
            .repository
            .list_permissions_for_subject(tenant_id, subject)
            .await?;

        if permissions.iter().any(|value| value == &permission) {
            return Ok(PermissionGrantResolution::Granted);
        }

        let temporary_grant = self
            .repository
            .find_active_temporary_permission_grant(tenant_id, subject, permission)
            .await?;

        Ok(temporary_grant
            .map(PermissionGrantResolution::Temporary)
            .unwrap_or(PermissionGrantResolution::Missing))
    }

    async fn append_temporary_access_use_event(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
        grant: &TemporaryPermissionGrant,
    ) -> AppResult<()> {
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id,
                subject: subject.to_owned(),
                action: AuditAction::SecurityTemporaryAccessUsed,
                resource_type: "security_temporary_access_grant".to_owned(),
                resource_id: grant.grant_id.clone(),
                detail: Some(format!(
                    "used temporary grant '{}' for permission '{}' (expires_at='{}', reason='{}')",
                    grant.grant_id,
                    permission.as_str(),
                    grant.expires_at,
                    grant.reason
                )),
            })
            .await
    }
}

enum PermissionGrantResolution {
    Granted,
    Temporary(TemporaryPermissionGrant),
    Missing,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use qryvanta_core::{AppResult, TenantId};
    use qryvanta_domain::Permission;
    use tokio::sync::Mutex;

    use crate::{AuditEvent, AuditRepository};

    use super::{
        AuthorizationRepository, AuthorizationService, RuntimeFieldGrant, TemporaryPermissionGrant,
    };

    #[derive(Default)]
    struct FakeAuditRepository {
        events: Mutex<Vec<AuditEvent>>,
    }

    #[async_trait]
    impl AuditRepository for FakeAuditRepository {
        async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
            self.events.lock().await.push(event);
            Ok(())
        }
    }

    struct FakeAuthorizationRepository {
        map: HashMap<(TenantId, String), Vec<Permission>>,
        runtime_field_grants: HashMap<(TenantId, String, String), Vec<RuntimeFieldGrant>>,
        temporary_permission_grants:
            HashMap<(TenantId, String, Permission), TemporaryPermissionGrant>,
    }

    #[async_trait]
    impl AuthorizationRepository for FakeAuthorizationRepository {
        async fn list_permissions_for_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
        ) -> AppResult<Vec<Permission>> {
            Ok(self
                .map
                .get(&(tenant_id, subject.to_owned()))
                .cloned()
                .unwrap_or_default())
        }

        async fn list_runtime_field_grants_for_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
            entity_logical_name: &str,
        ) -> AppResult<Vec<RuntimeFieldGrant>> {
            Ok(self
                .runtime_field_grants
                .get(&(
                    tenant_id,
                    subject.to_owned(),
                    entity_logical_name.to_owned(),
                ))
                .cloned()
                .unwrap_or_default())
        }

        async fn find_active_temporary_permission_grant(
            &self,
            tenant_id: TenantId,
            subject: &str,
            permission: Permission,
        ) -> AppResult<Option<TemporaryPermissionGrant>> {
            Ok(self
                .temporary_permission_grants
                .get(&(tenant_id, subject.to_owned(), permission))
                .cloned())
        }
    }

    #[tokio::test]
    async fn require_permission_allows_granted_subject() {
        let tenant_id = TenantId::new();
        let repository = FakeAuthorizationRepository {
            map: HashMap::from([(
                (tenant_id, "alice".to_owned()),
                vec![Permission::MetadataEntityRead],
            )]),
            runtime_field_grants: HashMap::new(),
            temporary_permission_grants: HashMap::new(),
        };
        let service = AuthorizationService::new(
            Arc::new(repository),
            Arc::new(FakeAuditRepository::default()),
        );

        let result = service
            .require_permission(tenant_id, "alice", Permission::MetadataEntityRead)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn require_permission_denies_missing_grant() {
        let tenant_id = TenantId::new();
        let repository = FakeAuthorizationRepository {
            map: HashMap::new(),
            runtime_field_grants: HashMap::new(),
            temporary_permission_grants: HashMap::new(),
        };
        let service = AuthorizationService::new(
            Arc::new(repository),
            Arc::new(FakeAuditRepository::default()),
        );

        let result = service
            .require_permission(tenant_id, "alice", Permission::MetadataEntityCreate)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn require_permission_allows_active_temporary_grant() {
        let tenant_id = TenantId::new();
        let repository = FakeAuthorizationRepository {
            map: HashMap::new(),
            runtime_field_grants: HashMap::new(),
            temporary_permission_grants: HashMap::from([(
                (
                    tenant_id,
                    "alice".to_owned(),
                    Permission::RuntimeRecordWrite,
                ),
                TemporaryPermissionGrant {
                    grant_id: "grant-1".to_owned(),
                    reason: "incident response".to_owned(),
                    expires_at: "2099-01-01T00:00:00Z".to_owned(),
                },
            )]),
        };
        let audit_repository = Arc::new(FakeAuditRepository::default());
        let service = AuthorizationService::new(Arc::new(repository), audit_repository.clone());

        let result = service
            .require_permission(tenant_id, "alice", Permission::RuntimeRecordWrite)
            .await;
        assert!(result.is_ok());

        let events = audit_repository.events.lock().await;
        assert_eq!(events.len(), 1);
    }
}
