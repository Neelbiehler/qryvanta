use std::sync::Arc;

use qryvanta_core::{AppError, AppResult, UserIdentity};
use qryvanta_domain::{
    ExtensionDefinition, ExtensionLifecycleState, ExtensionManifest, Permission,
};

use crate::AuthorizationService;
use crate::extension_ports::{
    ExecuteExtensionActionInput, ExtensionActionResult, ExtensionRepository, ExtensionRuntime,
    RuntimeExtensionActionRequest,
};

/// Supported platform extension API versions.
const SUPPORTED_EXTENSION_API_VERSIONS: &[&str] = &["1.0"];

/// Input payload for extension registration.
#[derive(Debug, Clone)]
pub struct RegisterExtensionInput {
    /// Immutable extension manifest.
    pub manifest: ExtensionManifest,
    /// SHA-256 fingerprint of package bytes.
    pub package_sha256: String,
}

/// Compatibility report across platform API versions.
#[derive(Debug, Clone)]
pub struct ExtensionCompatibilityReport {
    /// Extension logical name.
    pub extension_logical_name: String,
    /// Checked platform API versions.
    pub checked_platform_api_versions: Vec<String>,
    /// Platform versions accepted by runtime compatibility checks.
    pub compatible_platform_api_versions: Vec<String>,
    /// Platform versions rejected by runtime compatibility checks.
    pub incompatible_platform_api_versions: Vec<String>,
    /// Indicates whether all checked versions are compatible.
    pub is_compatible: bool,
}

/// Application service for extension lifecycle, compatibility, and execution policy enforcement.
#[derive(Clone)]
pub struct ExtensionService {
    authorization_service: AuthorizationService,
    repository: Arc<dyn ExtensionRepository>,
    runtime: Arc<dyn ExtensionRuntime>,
}

impl ExtensionService {
    /// Creates a new extension service.
    #[must_use]
    pub fn new(
        authorization_service: AuthorizationService,
        repository: Arc<dyn ExtensionRepository>,
        runtime: Arc<dyn ExtensionRuntime>,
    ) -> Self {
        Self {
            authorization_service,
            repository,
            runtime,
        }
    }

    /// Returns supported platform extension API versions.
    #[must_use]
    pub fn supported_api_versions() -> &'static [&'static str] {
        SUPPORTED_EXTENSION_API_VERSIONS
    }

    /// Registers a draft extension definition.
    pub async fn register_extension(
        &self,
        actor: &UserIdentity,
        input: RegisterExtensionInput,
    ) -> AppResult<ExtensionDefinition> {
        self.require_extension_manage_permission(actor).await?;

        if self
            .repository
            .find_extension(actor.tenant_id(), input.manifest.logical_name().as_str())
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(format!(
                "extension '{}' already exists",
                input.manifest.logical_name().as_str()
            )));
        }

        let definition = ExtensionDefinition::new(input.manifest, input.package_sha256)?;
        self.repository
            .save_extension(actor.tenant_id(), definition.clone())
            .await?;

        Ok(definition)
    }

    /// Lists extension definitions for one tenant.
    pub async fn list_extensions(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<Vec<ExtensionDefinition>> {
        self.require_extension_manage_permission(actor).await?;
        self.repository.list_extensions(actor.tenant_id()).await
    }

    /// Publishes one draft extension.
    pub async fn publish_extension(
        &self,
        actor: &UserIdentity,
        extension_logical_name: &str,
    ) -> AppResult<ExtensionDefinition> {
        self.require_extension_manage_permission(actor).await?;

        let definition = self
            .repository
            .find_extension(actor.tenant_id(), extension_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "extension '{}' does not exist",
                    extension_logical_name
                ))
            })?;

        let published = definition.publish();
        self.repository
            .save_extension(actor.tenant_id(), published.clone())
            .await?;

        Ok(published)
    }

    /// Disables one extension.
    pub async fn disable_extension(
        &self,
        actor: &UserIdentity,
        extension_logical_name: &str,
    ) -> AppResult<ExtensionDefinition> {
        self.require_extension_manage_permission(actor).await?;

        let definition = self
            .repository
            .find_extension(actor.tenant_id(), extension_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "extension '{}' does not exist",
                    extension_logical_name
                ))
            })?;

        let disabled = definition.disable();
        self.repository
            .save_extension(actor.tenant_id(), disabled.clone())
            .await?;

        Ok(disabled)
    }

    /// Executes compatibility checks across provided or default platform API versions.
    pub async fn extension_compatibility_report(
        &self,
        actor: &UserIdentity,
        extension_logical_name: &str,
        platform_api_versions: Vec<String>,
    ) -> AppResult<ExtensionCompatibilityReport> {
        self.require_extension_manage_permission(actor).await?;

        let definition = self
            .repository
            .find_extension(actor.tenant_id(), extension_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "extension '{}' does not exist",
                    extension_logical_name
                ))
            })?;

        let mut checked_versions = platform_api_versions
            .into_iter()
            .map(|version| version.trim().to_owned())
            .filter(|version| !version.is_empty())
            .collect::<Vec<_>>();
        if checked_versions.is_empty() {
            checked_versions = Self::supported_api_versions()
                .iter()
                .map(ToString::to_string)
                .collect();
        }
        checked_versions.sort();
        checked_versions.dedup();

        let mut compatible_versions = Vec::new();
        let mut incompatible_versions = Vec::new();

        for version in &checked_versions {
            let runtime_compatible = self
                .runtime
                .validate_compatibility(&definition, version.as_str())
                .await?;
            if runtime_compatible {
                compatible_versions.push(version.clone());
            } else {
                incompatible_versions.push(version.clone());
            }
        }

        Ok(ExtensionCompatibilityReport {
            extension_logical_name: extension_logical_name.to_owned(),
            checked_platform_api_versions: checked_versions,
            compatible_platform_api_versions: compatible_versions,
            incompatible_platform_api_versions: incompatible_versions.clone(),
            is_compatible: incompatible_versions.is_empty(),
        })
    }

    /// Executes one extension action after capability and isolation policy enforcement.
    pub async fn execute_extension_action(
        &self,
        actor: &UserIdentity,
        input: ExecuteExtensionActionInput,
    ) -> AppResult<ExtensionActionResult> {
        self.require_extension_manage_permission(actor).await?;

        let definition = self
            .repository
            .find_extension(actor.tenant_id(), input.extension_logical_name.as_str())
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "extension '{}' does not exist",
                    input.extension_logical_name
                ))
            })?;

        if definition.lifecycle_state() != ExtensionLifecycleState::Published {
            return Err(AppError::Forbidden(format!(
                "extension '{}' is not published",
                input.extension_logical_name
            )));
        }

        let required_capability = input.action_type.required_capability();
        if !definition
            .manifest()
            .requested_capabilities()
            .contains(&required_capability)
        {
            return Err(AppError::Forbidden(format!(
                "extension '{}' is missing required capability '{}' for action '{}'",
                input.extension_logical_name,
                required_capability.as_str(),
                input.action_type.as_str()
            )));
        }

        let policy = definition.manifest().isolation_policy();

        if input.requested_memory_mb > policy.max_memory_mb() {
            return Err(AppError::Forbidden(format!(
                "extension '{}' requested memory {}MB above limit {}MB",
                input.extension_logical_name,
                input.requested_memory_mb,
                policy.max_memory_mb()
            )));
        }

        if input.requested_cpu_time_ms > policy.max_cpu_time_ms() {
            return Err(AppError::Forbidden(format!(
                "extension '{}' requested cpu time {}ms above limit {}ms",
                input.extension_logical_name,
                input.requested_cpu_time_ms,
                policy.max_cpu_time_ms()
            )));
        }

        if input.requested_storage_mb > policy.max_storage_mb() {
            return Err(AppError::Forbidden(format!(
                "extension '{}' requested storage {}MB above limit {}MB",
                input.extension_logical_name,
                input.requested_storage_mb,
                policy.max_storage_mb()
            )));
        }

        if input.target_host.is_some() && !input.requested_network_access {
            return Err(AppError::Validation(
                "target_host requires requested_network_access=true".to_owned(),
            ));
        }

        if input.requested_network_access {
            if !policy.allow_network() {
                return Err(AppError::Forbidden(format!(
                    "extension '{}' requested network access but policy forbids it",
                    input.extension_logical_name
                )));
            }

            let normalized_target_host = input
                .target_host
                .as_ref()
                .map(|host| host.trim().to_ascii_lowercase());

            if !policy.allowed_hosts().is_empty() {
                let Some(target_host) = normalized_target_host else {
                    return Err(AppError::Validation(
                        "target_host is required when allowed_hosts is configured".to_owned(),
                    ));
                };

                if !policy
                    .allowed_hosts()
                    .iter()
                    .any(|host| host == &target_host)
                {
                    return Err(AppError::Forbidden(format!(
                        "extension '{}' target host '{}' is not allowed",
                        input.extension_logical_name, target_host
                    )));
                }
            }
        }

        self.runtime
            .execute_action(RuntimeExtensionActionRequest {
                tenant_id: actor.tenant_id(),
                extension: definition,
                input,
            })
            .await
    }

    async fn require_extension_manage_permission(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use qryvanta_core::{AppResult, TenantId, UserIdentity};
    use qryvanta_domain::{
        ExtensionCapability, ExtensionDefinition, ExtensionIsolationPolicy, ExtensionManifest,
        ExtensionManifestInput, ExtensionRuntimeKind, Permission,
    };
    use serde_json::json;
    use tokio::sync::{Mutex, RwLock};
    use uuid::Uuid;

    use crate::authorization_service::AuthorizationRepository;
    use crate::extension_ports::{
        ExtensionRepository, ExtensionRuntime, RuntimeExtensionActionRequest,
    };
    use crate::{
        AuditEvent, AuditRepository, AuthorizationService, RuntimeFieldGrant,
        TemporaryPermissionGrant,
    };

    use super::{
        ExecuteExtensionActionInput, ExtensionActionResult, ExtensionCompatibilityReport,
        ExtensionService, RegisterExtensionInput,
    };
    use crate::ExtensionActionType;

    struct FakeAuthorizationRepository {
        grants: HashMap<(TenantId, String), Vec<Permission>>,
    }

    #[async_trait]
    impl AuthorizationRepository for FakeAuthorizationRepository {
        async fn list_permissions_for_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
        ) -> AppResult<Vec<Permission>> {
            Ok(self
                .grants
                .get(&(tenant_id, subject.to_owned()))
                .cloned()
                .unwrap_or_default())
        }

        async fn list_runtime_field_grants_for_subject(
            &self,
            _tenant_id: TenantId,
            _subject: &str,
            _entity_logical_name: &str,
        ) -> AppResult<Vec<RuntimeFieldGrant>> {
            Ok(Vec::new())
        }

        async fn find_active_temporary_permission_grant(
            &self,
            _tenant_id: TenantId,
            _subject: &str,
            _permission: Permission,
        ) -> AppResult<Option<TemporaryPermissionGrant>> {
            Ok(None)
        }
    }

    #[derive(Default)]
    struct FakeExtensionRepository {
        definitions: RwLock<HashMap<(TenantId, String), ExtensionDefinition>>,
    }

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

    #[async_trait]
    impl ExtensionRepository for FakeExtensionRepository {
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

        async fn list_extensions(
            &self,
            tenant_id: TenantId,
        ) -> AppResult<Vec<ExtensionDefinition>> {
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

    #[derive(Default)]
    struct FakeExtensionRuntime;

    #[async_trait]
    impl ExtensionRuntime for FakeExtensionRuntime {
        async fn validate_compatibility(
            &self,
            definition: &ExtensionDefinition,
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
                status: "ok".to_owned(),
                output: json!({
                    "extension": request.extension.manifest().logical_name().as_str(),
                    "action": request.input.action_type.as_str(),
                }),
            })
        }
    }

    fn actor(tenant_id: TenantId, subject: &str) -> UserIdentity {
        UserIdentity::new(subject, subject, None, tenant_id)
    }

    fn make_manifest(
        logical_name: &str,
        runtime_api_version: &str,
        capabilities: Vec<ExtensionCapability>,
        allow_network: bool,
        allowed_hosts: Vec<String>,
    ) -> ExtensionManifest {
        ExtensionManifest::new(ExtensionManifestInput {
            logical_name: logical_name.to_owned(),
            display_name: logical_name.to_owned(),
            package_version: "1.0.0".to_owned(),
            runtime_api_version: runtime_api_version.to_owned(),
            runtime_kind: ExtensionRuntimeKind::Wasm,
            requested_capabilities: capabilities,
            isolation_policy: ExtensionIsolationPolicy::new(
                128,
                2_000,
                64,
                allow_network,
                allowed_hosts,
            )
            .unwrap_or_else(|_| unreachable!()),
        })
        .unwrap_or_else(|_| unreachable!())
    }

    fn build_service(tenant_id: TenantId, subject: &str) -> ExtensionService {
        let authorization_service = AuthorizationService::new(
            Arc::new(FakeAuthorizationRepository {
                grants: HashMap::from([(
                    (tenant_id, subject.to_owned()),
                    vec![Permission::SecurityRoleManage],
                )]),
            }),
            Arc::new(FakeAuditRepository::default()),
        );

        ExtensionService::new(
            authorization_service,
            Arc::new(FakeExtensionRepository::default()),
            Arc::new(FakeExtensionRuntime),
        )
    }

    async fn register_and_publish(
        service: &ExtensionService,
        actor: &UserIdentity,
        manifest: ExtensionManifest,
    ) {
        let created = service
            .register_extension(
                actor,
                RegisterExtensionInput {
                    manifest,
                    package_sha256: "abc123".to_owned(),
                },
            )
            .await;
        assert!(created.is_ok());

        let published = service
            .publish_extension(
                actor,
                created
                    .unwrap_or_else(|_| unreachable!())
                    .manifest()
                    .logical_name()
                    .as_str(),
            )
            .await;
        assert!(published.is_ok());
    }

    #[tokio::test]
    async fn execute_extension_action_rejects_missing_capability() {
        let tenant_id = TenantId::new();
        let subject = "owner";
        let actor = actor(tenant_id, subject);
        let service = build_service(tenant_id, subject);

        register_and_publish(
            &service,
            &actor,
            make_manifest(
                "read_only_extension",
                "1.0",
                vec![ExtensionCapability::RuntimeRecordRead],
                false,
                Vec::new(),
            ),
        )
        .await;

        let result = service
            .execute_extension_action(
                &actor,
                ExecuteExtensionActionInput {
                    extension_logical_name: "read_only_extension".to_owned(),
                    action_type: ExtensionActionType::RuntimeRecordWriteHook,
                    payload: json!({"record_id": "1"}),
                    requested_memory_mb: 64,
                    requested_cpu_time_ms: 1_000,
                    requested_storage_mb: 16,
                    requested_network_access: false,
                    target_host: None,
                },
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_extension_action_rejects_isolation_policy_violation() {
        let tenant_id = TenantId::new();
        let subject = "owner";
        let actor = actor(tenant_id, subject);
        let service = build_service(tenant_id, subject);

        register_and_publish(
            &service,
            &actor,
            make_manifest(
                "network_extension",
                "1.0",
                vec![ExtensionCapability::OutboundHttp],
                true,
                vec!["api.example.com".to_owned()],
            ),
        )
        .await;

        let result = service
            .execute_extension_action(
                &actor,
                ExecuteExtensionActionInput {
                    extension_logical_name: "network_extension".to_owned(),
                    action_type: ExtensionActionType::OutboundHttp,
                    payload: json!({"method": "GET"}),
                    requested_memory_mb: 129,
                    requested_cpu_time_ms: 1_000,
                    requested_storage_mb: 16,
                    requested_network_access: true,
                    target_host: Some("api.example.com".to_owned()),
                },
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn extension_compatibility_report_splits_versions() {
        let tenant_id = TenantId::new();
        let subject = "owner";
        let actor = actor(tenant_id, subject);
        let service = build_service(tenant_id, subject);

        register_and_publish(
            &service,
            &actor,
            make_manifest(
                "versioned_extension",
                "1.0",
                vec![ExtensionCapability::RuntimeRecordRead],
                false,
                Vec::new(),
            ),
        )
        .await;

        let report = service
            .extension_compatibility_report(
                &actor,
                "versioned_extension",
                vec!["1.0".to_owned(), "2.0".to_owned()],
            )
            .await;

        assert!(report.is_ok());
        let ExtensionCompatibilityReport {
            compatible_platform_api_versions,
            incompatible_platform_api_versions,
            is_compatible,
            ..
        } = report.unwrap_or_else(|_| unreachable!());

        assert_eq!(compatible_platform_api_versions, vec!["1.0".to_owned()]);
        assert_eq!(incompatible_platform_api_versions, vec!["2.0".to_owned()]);
        assert!(!is_compatible);
    }
}
