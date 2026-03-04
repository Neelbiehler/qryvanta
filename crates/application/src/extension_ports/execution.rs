use std::str::FromStr;

use async_trait::async_trait;
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{ExtensionCapability, ExtensionDefinition};
use serde_json::Value;

/// Stable extension action categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionActionType {
    /// Runtime read-hook action.
    RuntimeRecordReadHook,
    /// Runtime write-hook action.
    RuntimeRecordWriteHook,
    /// Metadata write-hook action.
    MetadataWriteHook,
    /// Outbound HTTP action.
    OutboundHttp,
    /// Workflow dispatch action.
    WorkflowDispatch,
}

impl ExtensionActionType {
    /// Returns stable action type value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuntimeRecordReadHook => "runtime_record_read_hook",
            Self::RuntimeRecordWriteHook => "runtime_record_write_hook",
            Self::MetadataWriteHook => "metadata_write_hook",
            Self::OutboundHttp => "outbound_http",
            Self::WorkflowDispatch => "workflow_dispatch",
        }
    }

    /// Returns capability required for this action type.
    #[must_use]
    pub fn required_capability(&self) -> ExtensionCapability {
        match self {
            Self::RuntimeRecordReadHook => ExtensionCapability::RuntimeRecordRead,
            Self::RuntimeRecordWriteHook => ExtensionCapability::RuntimeRecordWrite,
            Self::MetadataWriteHook => ExtensionCapability::MetadataWrite,
            Self::OutboundHttp => ExtensionCapability::OutboundHttp,
            Self::WorkflowDispatch => ExtensionCapability::WorkflowDispatch,
        }
    }
}

impl FromStr for ExtensionActionType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "runtime_record_read_hook" => Ok(Self::RuntimeRecordReadHook),
            "runtime_record_write_hook" => Ok(Self::RuntimeRecordWriteHook),
            "metadata_write_hook" => Ok(Self::MetadataWriteHook),
            "outbound_http" => Ok(Self::OutboundHttp),
            "workflow_dispatch" => Ok(Self::WorkflowDispatch),
            _ => Err(AppError::Validation(format!(
                "unknown extension action type '{}'",
                value
            ))),
        }
    }
}

/// Normalized input used by extension execution entrypoints.
#[derive(Debug, Clone)]
pub struct ExecuteExtensionActionInput {
    /// Extension logical name.
    pub extension_logical_name: String,
    /// Action type.
    pub action_type: ExtensionActionType,
    /// JSON payload for extension action execution.
    pub payload: Value,
    /// Requested memory budget in MB.
    pub requested_memory_mb: u32,
    /// Requested CPU execution time in milliseconds.
    pub requested_cpu_time_ms: u64,
    /// Requested storage budget in MB.
    pub requested_storage_mb: u32,
    /// Whether the action requires network access.
    pub requested_network_access: bool,
    /// Optional target host for network actions.
    pub target_host: Option<String>,
}

/// Runtime action request payload passed to adapter implementations.
#[derive(Debug, Clone)]
pub struct RuntimeExtensionActionRequest {
    /// Tenant scope.
    pub tenant_id: TenantId,
    /// Extension definition snapshot.
    pub extension: ExtensionDefinition,
    /// Requested action envelope.
    pub input: ExecuteExtensionActionInput,
}

/// Runtime action execution result.
#[derive(Debug, Clone)]
pub struct ExtensionActionResult {
    /// Runtime-assigned execution identifier.
    pub execution_id: String,
    /// Stable status value.
    pub status: String,
    /// Action output payload.
    pub output: Value,
}

/// Extension runtime execution and compatibility port.
#[async_trait]
pub trait ExtensionRuntime: Send + Sync {
    /// Validates extension package compatibility with one platform API version.
    async fn validate_compatibility(
        &self,
        definition: &ExtensionDefinition,
        platform_api_version: &str,
    ) -> AppResult<bool>;

    /// Executes one extension action under already validated policy controls.
    async fn execute_action(
        &self,
        request: RuntimeExtensionActionRequest,
    ) -> AppResult<ExtensionActionResult>;
}
