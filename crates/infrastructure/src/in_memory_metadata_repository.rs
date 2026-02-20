use std::collections::HashMap;

use async_trait::async_trait;
use qryvanta_application::MetadataRepository;
use qryvanta_core::TenantId;
use qryvanta_core::{AppError, AppResult};
use qryvanta_domain::EntityDefinition;
use tokio::sync::RwLock;

/// In-memory metadata repository implementation.
#[derive(Debug, Default)]
pub struct InMemoryMetadataRepository {
    entities: RwLock<HashMap<(TenantId, String), EntityDefinition>>,
}

impl InMemoryMetadataRepository {
    /// Creates an empty in-memory repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MetadataRepository for InMemoryMetadataRepository {
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()> {
        let key = (tenant_id, entity.logical_name().as_str().to_owned());
        let mut entities = self.entities.write().await;

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
        let entities = self.entities.read().await;

        let mut values: Vec<EntityDefinition> = entities
            .iter()
            .filter_map(|((stored_tenant_id, _), entity)| {
                (stored_tenant_id == &tenant_id).then_some(entity.clone())
            })
            .collect();
        values.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use qryvanta_application::MetadataRepository;
    use qryvanta_core::TenantId;
    use qryvanta_domain::EntityDefinition;

    use super::InMemoryMetadataRepository;

    #[tokio::test]
    async fn save_and_list_entities() {
        let repository = InMemoryMetadataRepository::new();
        let tenant_id = TenantId::new();

        let entity = EntityDefinition::new("account", "Account");
        assert!(entity.is_ok());
        let save_result = repository
            .save_entity(tenant_id, entity.unwrap_or_else(|_| unreachable!()))
            .await;
        assert!(save_result.is_ok());

        let listed = repository.list_entities(tenant_id).await;
        assert!(listed.is_ok());
        assert_eq!(listed.unwrap_or_default().len(), 1);
    }
}
