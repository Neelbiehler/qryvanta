use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// Incoming payload for extension registration.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/create-extension-request.ts"
)]
pub struct CreateExtensionRequest {
    pub logical_name: String,
    pub display_name: String,
    pub package_version: String,
    pub runtime_api_version: String,
    pub runtime_kind: String,
    pub package_sha256: String,
    pub requested_capabilities: Vec<String>,
    pub isolation_policy: ExtensionIsolationPolicyDto,
}

/// Shared isolation policy transport model.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/extension-isolation-policy-dto.ts"
)]
pub struct ExtensionIsolationPolicyDto {
    pub max_memory_mb: u32,
    pub max_cpu_time_ms: u64,
    pub max_storage_mb: u32,
    pub allow_network: bool,
    pub allowed_hosts: Vec<String>,
}

/// Extension definition API response.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/extension-response.ts"
)]
pub struct ExtensionResponse {
    pub logical_name: String,
    pub display_name: String,
    pub package_version: String,
    pub runtime_api_version: String,
    pub runtime_kind: String,
    pub package_sha256: String,
    pub lifecycle_state: String,
    pub requested_capabilities: Vec<String>,
    pub isolation_policy: ExtensionIsolationPolicyDto,
}

/// Compatibility check request payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/extension-compatibility-request.ts"
)]
pub struct ExtensionCompatibilityRequest {
    #[serde(default)]
    pub platform_api_versions: Vec<String>,
}

/// Compatibility check response payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/extension-compatibility-response.ts"
)]
pub struct ExtensionCompatibilityResponse {
    pub extension_logical_name: String,
    pub checked_platform_api_versions: Vec<String>,
    pub compatible_platform_api_versions: Vec<String>,
    pub incompatible_platform_api_versions: Vec<String>,
    pub is_compatible: bool,
}

/// Extension execution request payload.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/execute-extension-action-request.ts"
)]
pub struct ExecuteExtensionActionRequest {
    pub action_type: String,
    #[ts(type = "unknown")]
    pub payload: Value,
    pub requested_memory_mb: u32,
    pub requested_cpu_time_ms: u64,
    pub requested_storage_mb: u32,
    pub requested_network_access: bool,
    pub target_host: Option<String>,
}

/// Extension execution response payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/execute-extension-action-response.ts"
)]
pub struct ExecuteExtensionActionResponse {
    pub execution_id: String,
    pub status: String,
    #[ts(type = "unknown")]
    pub output: Value,
}
