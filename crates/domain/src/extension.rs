use std::str::FromStr;

use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};

/// Extension runtime boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionRuntimeKind {
    /// WebAssembly component runtime.
    Wasm,
}

impl ExtensionRuntimeKind {
    /// Returns stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wasm => "wasm",
        }
    }
}

impl FromStr for ExtensionRuntimeKind {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "wasm" => Ok(Self::Wasm),
            _ => Err(AppError::Validation(format!(
                "unknown extension runtime '{}'",
                value
            ))),
        }
    }
}

/// Extension capability granted to a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionCapability {
    /// Read runtime records.
    RuntimeRecordRead,
    /// Write runtime records.
    RuntimeRecordWrite,
    /// Read metadata definitions.
    MetadataRead,
    /// Write metadata definitions.
    MetadataWrite,
    /// Execute outbound HTTP calls.
    OutboundHttp,
    /// Trigger workflow dispatch.
    WorkflowDispatch,
}

impl ExtensionCapability {
    /// Returns stable transport value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuntimeRecordRead => "runtime.record.read",
            Self::RuntimeRecordWrite => "runtime.record.write",
            Self::MetadataRead => "metadata.read",
            Self::MetadataWrite => "metadata.write",
            Self::OutboundHttp => "outbound.http",
            Self::WorkflowDispatch => "workflow.dispatch",
        }
    }
}

impl FromStr for ExtensionCapability {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "runtime.record.read" => Ok(Self::RuntimeRecordRead),
            "runtime.record.write" => Ok(Self::RuntimeRecordWrite),
            "metadata.read" => Ok(Self::MetadataRead),
            "metadata.write" => Ok(Self::MetadataWrite),
            "outbound.http" => Ok(Self::OutboundHttp),
            "workflow.dispatch" => Ok(Self::WorkflowDispatch),
            _ => Err(AppError::Validation(format!(
                "unknown extension capability '{}'",
                value
            ))),
        }
    }
}

/// Stable extension lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionLifecycleState {
    /// Created but not available for execution.
    Draft,
    /// Published and executable.
    Published,
    /// Disabled and blocked from execution.
    Disabled,
}

/// Sandboxing policy attached to one extension package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionIsolationPolicy {
    max_memory_mb: u32,
    max_cpu_time_ms: u64,
    max_storage_mb: u32,
    allow_network: bool,
    allowed_hosts: Vec<String>,
}

impl ExtensionIsolationPolicy {
    /// Creates a validated isolation policy.
    pub fn new(
        max_memory_mb: u32,
        max_cpu_time_ms: u64,
        max_storage_mb: u32,
        allow_network: bool,
        allowed_hosts: Vec<String>,
    ) -> AppResult<Self> {
        if max_memory_mb == 0 {
            return Err(AppError::Validation(
                "max_memory_mb must be greater than zero".to_owned(),
            ));
        }
        if max_memory_mb > 4096 {
            return Err(AppError::Validation(
                "max_memory_mb must be less than or equal to 4096".to_owned(),
            ));
        }
        if max_cpu_time_ms == 0 {
            return Err(AppError::Validation(
                "max_cpu_time_ms must be greater than zero".to_owned(),
            ));
        }
        if max_storage_mb == 0 {
            return Err(AppError::Validation(
                "max_storage_mb must be greater than zero".to_owned(),
            ));
        }

        let mut normalized_hosts = allowed_hosts
            .into_iter()
            .map(|host| host.trim().to_ascii_lowercase())
            .filter(|host| !host.is_empty())
            .collect::<Vec<_>>();
        normalized_hosts.sort();
        normalized_hosts.dedup();

        if !allow_network && !normalized_hosts.is_empty() {
            return Err(AppError::Validation(
                "allowed_hosts must be empty when allow_network is false".to_owned(),
            ));
        }

        Ok(Self {
            max_memory_mb,
            max_cpu_time_ms,
            max_storage_mb,
            allow_network,
            allowed_hosts: normalized_hosts,
        })
    }

    /// Returns maximum memory budget in MB.
    #[must_use]
    pub fn max_memory_mb(&self) -> u32 {
        self.max_memory_mb
    }

    /// Returns maximum CPU time budget in milliseconds.
    #[must_use]
    pub fn max_cpu_time_ms(&self) -> u64 {
        self.max_cpu_time_ms
    }

    /// Returns maximum storage budget in MB.
    #[must_use]
    pub fn max_storage_mb(&self) -> u32 {
        self.max_storage_mb
    }

    /// Returns whether network access is allowed.
    #[must_use]
    pub fn allow_network(&self) -> bool {
        self.allow_network
    }

    /// Returns normalized allowed host list.
    #[must_use]
    pub fn allowed_hosts(&self) -> &[String] {
        &self.allowed_hosts
    }
}

/// Immutable extension package manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionManifest {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    package_version: NonEmptyString,
    runtime_api_version: NonEmptyString,
    runtime_kind: ExtensionRuntimeKind,
    requested_capabilities: Vec<ExtensionCapability>,
    isolation_policy: ExtensionIsolationPolicy,
}

/// Input payload used to build a validated extension manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionManifestInput {
    /// Stable extension identifier.
    pub logical_name: String,
    /// User-facing extension name.
    pub display_name: String,
    /// Extension package version.
    pub package_version: String,
    /// Runtime API contract version expected by the extension.
    pub runtime_api_version: String,
    /// Runtime kind.
    pub runtime_kind: ExtensionRuntimeKind,
    /// Capabilities requested by this extension.
    pub requested_capabilities: Vec<ExtensionCapability>,
    /// Runtime isolation profile.
    pub isolation_policy: ExtensionIsolationPolicy,
}

impl ExtensionManifest {
    /// Creates a validated extension manifest.
    pub fn new(input: ExtensionManifestInput) -> AppResult<Self> {
        let ExtensionManifestInput {
            logical_name,
            display_name,
            package_version,
            runtime_api_version,
            runtime_kind,
            requested_capabilities,
            isolation_policy,
        } = input;

        let mut capabilities = requested_capabilities;
        capabilities.sort_by_key(ExtensionCapability::as_str);
        capabilities.dedup();

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            package_version: NonEmptyString::new(package_version)?,
            runtime_api_version: NonEmptyString::new(runtime_api_version)?,
            runtime_kind,
            requested_capabilities: capabilities,
            isolation_policy,
        })
    }

    /// Returns extension logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns extension display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns package version string.
    #[must_use]
    pub fn package_version(&self) -> &NonEmptyString {
        &self.package_version
    }

    /// Returns runtime API version.
    #[must_use]
    pub fn runtime_api_version(&self) -> &NonEmptyString {
        &self.runtime_api_version
    }

    /// Returns runtime kind.
    #[must_use]
    pub fn runtime_kind(&self) -> ExtensionRuntimeKind {
        self.runtime_kind
    }

    /// Returns requested capabilities.
    #[must_use]
    pub fn requested_capabilities(&self) -> &[ExtensionCapability] {
        &self.requested_capabilities
    }

    /// Returns isolation policy.
    #[must_use]
    pub fn isolation_policy(&self) -> &ExtensionIsolationPolicy {
        &self.isolation_policy
    }
}

/// Tenant-scoped extension definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDefinition {
    manifest: ExtensionManifest,
    package_sha256: NonEmptyString,
    lifecycle_state: ExtensionLifecycleState,
}

impl ExtensionDefinition {
    /// Creates a draft extension definition.
    pub fn new(manifest: ExtensionManifest, package_sha256: impl Into<String>) -> AppResult<Self> {
        Ok(Self {
            manifest,
            package_sha256: NonEmptyString::new(package_sha256)?,
            lifecycle_state: ExtensionLifecycleState::Draft,
        })
    }

    /// Returns extension manifest.
    #[must_use]
    pub fn manifest(&self) -> &ExtensionManifest {
        &self.manifest
    }

    /// Returns immutable package sha256 fingerprint.
    #[must_use]
    pub fn package_sha256(&self) -> &NonEmptyString {
        &self.package_sha256
    }

    /// Returns lifecycle state.
    #[must_use]
    pub fn lifecycle_state(&self) -> ExtensionLifecycleState {
        self.lifecycle_state
    }

    /// Returns whether extension is executable.
    #[must_use]
    pub fn is_published(&self) -> bool {
        self.lifecycle_state == ExtensionLifecycleState::Published
    }

    /// Publishes the extension definition.
    pub fn publish(&self) -> Self {
        let mut next = self.clone();
        next.lifecycle_state = ExtensionLifecycleState::Published;
        next
    }

    /// Disables the extension definition.
    pub fn disable(&self) -> Self {
        let mut next = self.clone();
        next.lifecycle_state = ExtensionLifecycleState::Disabled;
        next
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExtensionCapability, ExtensionDefinition, ExtensionIsolationPolicy, ExtensionManifest,
        ExtensionManifestInput, ExtensionRuntimeKind,
    };

    #[test]
    fn isolation_policy_rejects_hosts_without_network() {
        let result =
            ExtensionIsolationPolicy::new(128, 1000, 16, false, vec!["api.test".to_owned()]);
        assert!(result.is_err());
    }

    #[test]
    fn manifest_normalizes_capabilities() {
        let policy =
            ExtensionIsolationPolicy::new(128, 1000, 16, true, vec!["api.test".to_owned()])
                .unwrap_or_else(|_| unreachable!());
        let manifest = ExtensionManifest::new(ExtensionManifestInput {
            logical_name: "sample_extension".to_owned(),
            display_name: "Sample Extension".to_owned(),
            package_version: "1.0.0".to_owned(),
            runtime_api_version: "1.0".to_owned(),
            runtime_kind: ExtensionRuntimeKind::Wasm,
            requested_capabilities: vec![
                ExtensionCapability::OutboundHttp,
                ExtensionCapability::RuntimeRecordRead,
                ExtensionCapability::OutboundHttp,
            ],
            isolation_policy: policy,
        })
        .unwrap_or_else(|_| unreachable!());

        assert_eq!(manifest.requested_capabilities().len(), 2);
        assert_eq!(
            manifest.requested_capabilities()[0],
            ExtensionCapability::OutboundHttp
        );
        assert_eq!(
            manifest.requested_capabilities()[1],
            ExtensionCapability::RuntimeRecordRead
        );
    }

    #[test]
    fn definition_defaults_to_draft_and_can_publish() {
        let policy = ExtensionIsolationPolicy::new(128, 1000, 16, false, Vec::new())
            .unwrap_or_else(|_| unreachable!());
        let manifest = ExtensionManifest::new(ExtensionManifestInput {
            logical_name: "sample_extension".to_owned(),
            display_name: "Sample Extension".to_owned(),
            package_version: "1.0.0".to_owned(),
            runtime_api_version: "1.0".to_owned(),
            runtime_kind: ExtensionRuntimeKind::Wasm,
            requested_capabilities: vec![ExtensionCapability::RuntimeRecordRead],
            isolation_policy: policy,
        })
        .unwrap_or_else(|_| unreachable!());

        let definition =
            ExtensionDefinition::new(manifest, "abc123").unwrap_or_else(|_| unreachable!());
        assert!(!definition.is_published());

        let published = definition.publish();
        assert!(published.is_published());
    }
}
