use std::collections::HashMap;

use async_trait::async_trait;
use qryvanta_application::ExtensionRepository;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::ExtensionDefinition;
use tokio::sync::RwLock;

/// In-memory extension definition repository.
#[derive(Debug, Default)]
pub struct InMemoryExtensionRepository {
    definitions: RwLock<HashMap<(TenantId, String), ExtensionDefinition>>,
}

impl InMemoryExtensionRepository {
    /// Creates an empty in-memory extension repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ExtensionRepository for InMemoryExtensionRepository {
    async fn save_extension(
        &self,
        tenant_id: TenantId,
        definition: ExtensionDefinition,
    ) -> AppResult<()> {
        self.definitions.write().await.insert(
            (
                tenant_id,
                definition.manifest().logical_name().as_str().to_owned(),
            ),
            definition,
        );
        Ok(())
    }

    async fn find_extension(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<ExtensionDefinition>> {
        Ok(self
            .definitions
            .read()
            .await
            .get(&(tenant_id, logical_name.to_owned()))
            .cloned())
    }

    async fn list_extensions(&self, tenant_id: TenantId) -> AppResult<Vec<ExtensionDefinition>> {
        let mut listed = self
            .definitions
            .read()
            .await
            .iter()
            .filter_map(|((stored_tenant_id, _), definition)| {
                (stored_tenant_id == &tenant_id).then_some(definition.clone())
            })
            .collect::<Vec<_>>();
        listed.sort_by(|left, right| {
            left.manifest()
                .logical_name()
                .as_str()
                .cmp(right.manifest().logical_name().as_str())
        });

        Ok(listed)
    }
}
