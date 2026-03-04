use std::str::FromStr;

use axum::Json;
use axum::extract::{Extension, Path, State};

use qryvanta_application::{
    ExecuteExtensionActionInput, ExtensionActionType, RegisterExtensionInput,
};
use qryvanta_core::{AppError, UserIdentity};
use qryvanta_domain::{
    ExtensionCapability, ExtensionDefinition, ExtensionIsolationPolicy, ExtensionLifecycleState,
    ExtensionManifest, ExtensionManifestInput, ExtensionRuntimeKind,
};

use crate::dto::{
    CreateExtensionRequest, ExecuteExtensionActionRequest, ExecuteExtensionActionResponse,
    ExtensionCompatibilityRequest, ExtensionCompatibilityResponse, ExtensionIsolationPolicyDto,
    ExtensionResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn list_extensions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
) -> ApiResult<Json<Vec<ExtensionResponse>>> {
    let extensions = state
        .extension_service
        .list_extensions(&user)
        .await?
        .into_iter()
        .map(extension_response_from_definition)
        .collect();

    Ok(Json(extensions))
}

pub async fn create_extension_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<CreateExtensionRequest>,
) -> ApiResult<Json<ExtensionResponse>> {
    let runtime_kind = ExtensionRuntimeKind::from_str(payload.runtime_kind.as_str())?;
    let requested_capabilities = parse_extension_capabilities(&payload.requested_capabilities)?;

    let isolation_policy = ExtensionIsolationPolicy::new(
        payload.isolation_policy.max_memory_mb,
        payload.isolation_policy.max_cpu_time_ms,
        payload.isolation_policy.max_storage_mb,
        payload.isolation_policy.allow_network,
        payload.isolation_policy.allowed_hosts,
    )?;

    let manifest = ExtensionManifest::new(ExtensionManifestInput {
        logical_name: payload.logical_name,
        display_name: payload.display_name,
        package_version: payload.package_version,
        runtime_api_version: payload.runtime_api_version,
        runtime_kind,
        requested_capabilities,
        isolation_policy,
    })?;

    let definition = state
        .extension_service
        .register_extension(
            &user,
            RegisterExtensionInput {
                manifest,
                package_sha256: payload.package_sha256,
            },
        )
        .await?;

    Ok(Json(extension_response_from_definition(definition)))
}

pub async fn publish_extension_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(extension_logical_name): Path<String>,
) -> ApiResult<Json<ExtensionResponse>> {
    let definition = state
        .extension_service
        .publish_extension(&user, extension_logical_name.as_str())
        .await?;

    Ok(Json(extension_response_from_definition(definition)))
}

pub async fn disable_extension_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(extension_logical_name): Path<String>,
) -> ApiResult<Json<ExtensionResponse>> {
    let definition = state
        .extension_service
        .disable_extension(&user, extension_logical_name.as_str())
        .await?;

    Ok(Json(extension_response_from_definition(definition)))
}

pub async fn extension_compatibility_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(extension_logical_name): Path<String>,
    Json(payload): Json<ExtensionCompatibilityRequest>,
) -> ApiResult<Json<ExtensionCompatibilityResponse>> {
    let report = state
        .extension_service
        .extension_compatibility_report(
            &user,
            extension_logical_name.as_str(),
            payload.platform_api_versions,
        )
        .await?;

    Ok(Json(ExtensionCompatibilityResponse {
        extension_logical_name: report.extension_logical_name,
        checked_platform_api_versions: report.checked_platform_api_versions,
        compatible_platform_api_versions: report.compatible_platform_api_versions,
        incompatible_platform_api_versions: report.incompatible_platform_api_versions,
        is_compatible: report.is_compatible,
    }))
}

pub async fn execute_extension_action_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(extension_logical_name): Path<String>,
    Json(payload): Json<ExecuteExtensionActionRequest>,
) -> ApiResult<Json<ExecuteExtensionActionResponse>> {
    let action_type = ExtensionActionType::from_str(payload.action_type.as_str())?;

    let result = state
        .extension_service
        .execute_extension_action(
            &user,
            ExecuteExtensionActionInput {
                extension_logical_name,
                action_type,
                payload: payload.payload,
                requested_memory_mb: payload.requested_memory_mb,
                requested_cpu_time_ms: payload.requested_cpu_time_ms,
                requested_storage_mb: payload.requested_storage_mb,
                requested_network_access: payload.requested_network_access,
                target_host: payload.target_host,
            },
        )
        .await?;

    Ok(Json(ExecuteExtensionActionResponse {
        execution_id: result.execution_id,
        status: result.status,
        output: result.output,
    }))
}

fn parse_extension_capabilities(values: &[String]) -> Result<Vec<ExtensionCapability>, AppError> {
    values
        .iter()
        .map(|value| ExtensionCapability::from_str(value.as_str()))
        .collect()
}

fn extension_response_from_definition(definition: ExtensionDefinition) -> ExtensionResponse {
    ExtensionResponse {
        logical_name: definition.manifest().logical_name().as_str().to_owned(),
        display_name: definition.manifest().display_name().as_str().to_owned(),
        package_version: definition.manifest().package_version().as_str().to_owned(),
        runtime_api_version: definition
            .manifest()
            .runtime_api_version()
            .as_str()
            .to_owned(),
        runtime_kind: definition.manifest().runtime_kind().as_str().to_owned(),
        package_sha256: definition.package_sha256().as_str().to_owned(),
        lifecycle_state: extension_lifecycle_state_str(definition.lifecycle_state()).to_owned(),
        requested_capabilities: definition
            .manifest()
            .requested_capabilities()
            .iter()
            .map(|capability| capability.as_str().to_owned())
            .collect(),
        isolation_policy: ExtensionIsolationPolicyDto {
            max_memory_mb: definition.manifest().isolation_policy().max_memory_mb(),
            max_cpu_time_ms: definition.manifest().isolation_policy().max_cpu_time_ms(),
            max_storage_mb: definition.manifest().isolation_policy().max_storage_mb(),
            allow_network: definition.manifest().isolation_policy().allow_network(),
            allowed_hosts: definition
                .manifest()
                .isolation_policy()
                .allowed_hosts()
                .to_vec(),
        },
    }
}

fn extension_lifecycle_state_str(state: ExtensionLifecycleState) -> &'static str {
    match state {
        ExtensionLifecycleState::Draft => "draft",
        ExtensionLifecycleState::Published => "published",
        ExtensionLifecycleState::Disabled => "disabled",
    }
}
