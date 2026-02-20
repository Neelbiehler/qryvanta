use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::Permission;

/// Repository port for permission lookups.
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    /// Lists effective permissions for a subject in a tenant.
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>>;
}

/// Application service for tenant-scoped authorization checks.
#[derive(Clone)]
pub struct AuthorizationService {
    repository: Arc<dyn AuthorizationRepository>,
}

impl AuthorizationService {
    /// Creates a new authorization service from a repository implementation.
    #[must_use]
    pub fn new(repository: Arc<dyn AuthorizationRepository>) -> Self {
        Self { repository }
    }

    /// Ensures a subject has the required permission in the tenant scope.
    pub async fn require_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        permission: Permission,
    ) -> AppResult<()> {
        let permissions = self
            .repository
            .list_permissions_for_subject(tenant_id, subject)
            .await?;

        if permissions.iter().any(|value| value == &permission) {
            return Ok(());
        }

        Err(AppError::Forbidden(format!(
            "subject '{subject}' is missing permission '{}' in tenant '{tenant_id}'",
            permission.as_str()
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use qryvanta_core::{AppResult, TenantId};
    use qryvanta_domain::Permission;

    use super::{AuthorizationRepository, AuthorizationService};

    struct FakeAuthorizationRepository {
        map: HashMap<(TenantId, String), Vec<Permission>>,
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
    }

    #[tokio::test]
    async fn require_permission_allows_granted_subject() {
        let tenant_id = TenantId::new();
        let repository = FakeAuthorizationRepository {
            map: HashMap::from([(
                (tenant_id, "alice".to_owned()),
                vec![Permission::MetadataEntityRead],
            )]),
        };
        let service = AuthorizationService::new(Arc::new(repository));

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
        };
        let service = AuthorizationService::new(Arc::new(repository));

        let result = service
            .require_permission(tenant_id, "alice", Permission::MetadataEntityCreate)
            .await;
        assert!(result.is_err());
    }
}
