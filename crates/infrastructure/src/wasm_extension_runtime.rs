use async_trait::async_trait;
use qryvanta_application::{
    ExtensionActionResult, ExtensionRuntime, RuntimeExtensionActionRequest,
};
use qryvanta_core::AppResult;
use serde_json::json;
use uuid::Uuid;

/// Baseline runtime adapter for WASM extension execution boundaries.
#[derive(Debug, Default)]
pub struct WasmExtensionRuntime;

impl WasmExtensionRuntime {
    /// Creates a new WASM extension runtime adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ExtensionRuntime for WasmExtensionRuntime {
    async fn validate_compatibility(
        &self,
        definition: &qryvanta_domain::ExtensionDefinition,
        platform_api_version: &str,
    ) -> AppResult<bool> {
        Ok(definition.manifest().runtime_api_version().as_str() == platform_api_version)
    }

    async fn execute_action(
        &self,
        request: RuntimeExtensionActionRequest,
    ) -> AppResult<ExtensionActionResult> {
        Ok(ExtensionActionResult {
            execution_id: Uuid::new_v4().to_string(),
            status: "accepted".to_owned(),
            output: json!({
                "extension_logical_name": request.extension.manifest().logical_name().as_str(),
                "action_type": request.input.action_type.as_str(),
                "runtime_kind": request.extension.manifest().runtime_kind().as_str(),
                "tenant_id": request.tenant_id.to_string(),
            }),
        })
    }
}
