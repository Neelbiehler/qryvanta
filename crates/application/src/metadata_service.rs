use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::EntityDefinition;

/// Repository port for metadata persistence.
#[async_trait]
pub trait MetadataRepository: Send + Sync {
    /// Saves an entity definition.
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()>;

    /// Lists all entity definitions.
    async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>>;
}

/// Repository port for subject-to-tenant resolution.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Finds the tenant associated with the provided subject claim.
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>>;

    /// Adds a membership for the subject inside a tenant.
    async fn create_membership(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()>;
}

/// Application service for metadata operations.
#[derive(Clone)]
pub struct MetadataService {
    repository: Arc<dyn MetadataRepository>,
}

impl MetadataService {
    /// Creates a new metadata service from a repository implementation.
    #[must_use]
    pub fn new(repository: Arc<dyn MetadataRepository>) -> Self {
        Self { repository }
    }

    /// Registers a new entity definition.
    pub async fn register_entity(
        &self,
        tenant_id: TenantId,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> AppResult<EntityDefinition> {
        let entity = EntityDefinition::new(logical_name, display_name)?;
        self.repository
            .save_entity(tenant_id, entity.clone())
            .await?;
        Ok(entity)
    }

    /// Returns every known entity definition.
    pub async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
        self.repository.list_entities(tenant_id).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use qryvanta_core::{AppError, AppResult, TenantId};
    use qryvanta_domain::EntityDefinition;
    use tokio::sync::Mutex;

    use super::{MetadataRepository, MetadataService};

    struct FakeRepository {
        entities: Mutex<HashMap<(TenantId, String), EntityDefinition>>,
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                entities: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl MetadataRepository for FakeRepository {
        async fn save_entity(
            &self,
            tenant_id: TenantId,
            entity: EntityDefinition,
        ) -> AppResult<()> {
            let key = (tenant_id, entity.logical_name().as_str().to_owned());
            let mut entities = self.entities.lock().await;

            if entities.contains_key(&key) {
                return Err(AppError::Conflict(format!(
                    "entity '{}' already exists for tenant '{}'",
                    key.1, key.0
                )));
            }

            entities.insert(key, entity);
            Ok(())
        }

        async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
            let entities = self.entities.lock().await;
            let mut listed: Vec<EntityDefinition> = entities
                .iter()
                .filter_map(|((stored_tenant_id, _), entity)| {
                    (stored_tenant_id == &tenant_id).then_some(entity.clone())
                })
                .collect();
            listed.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });
            Ok(listed)
        }
    }

    #[tokio::test]
    async fn register_entity_persists_data() {
        let service = MetadataService::new(Arc::new(FakeRepository::new()));
        let tenant_id = TenantId::new();

        let created = service
            .register_entity(tenant_id, "contact", "Contact")
            .await;
        assert!(created.is_ok());

        let entities = service.list_entities(tenant_id).await;
        assert!(entities.is_ok());
        assert_eq!(entities.unwrap_or_default().len(), 1);
    }
}
