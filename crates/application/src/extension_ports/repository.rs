use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::ExtensionDefinition;

/// Extension definition persistence port.
#[async_trait]
pub trait ExtensionRepository: Send + Sync {
    /// Saves or updates an extension definition.
    async fn save_extension(
        &self,
        tenant_id: TenantId,
        definition: ExtensionDefinition,
    ) -> AppResult<()>;

    /// Finds one extension definition.
    async fn find_extension(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<ExtensionDefinition>>;

    /// Lists all extension definitions for one tenant.
    async fn list_extensions(&self, tenant_id: TenantId) -> AppResult<Vec<ExtensionDefinition>>;
}
