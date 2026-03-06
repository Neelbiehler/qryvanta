use std::str::FromStr;

use crate::begin_tenant_transaction;
use async_trait::async_trait;
use qryvanta_application::ExtensionRepository;
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    ExtensionCapability, ExtensionDefinition, ExtensionIsolationPolicy, ExtensionLifecycleState,
    ExtensionManifest, ExtensionManifestInput, ExtensionRuntimeKind,
};
use sqlx::{FromRow, PgPool};

/// PostgreSQL-backed extension repository.
#[derive(Clone)]
pub struct PostgresExtensionRepository {
    pool: PgPool,
}

impl PostgresExtensionRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ExtensionRow {
    logical_name: String,
    display_name: String,
    package_version: String,
    runtime_api_version: String,
    runtime_kind: String,
    package_sha256: String,
    lifecycle_state: String,
    requested_capabilities: Vec<String>,
    isolation_policy_json: serde_json::Value,
}

#[async_trait]
impl ExtensionRepository for PostgresExtensionRepository {
    async fn save_extension(
        &self,
        tenant_id: TenantId,
        definition: ExtensionDefinition,
    ) -> AppResult<()> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let requested_capabilities = definition
            .manifest()
            .requested_capabilities()
            .iter()
            .map(|capability| capability.as_str().to_owned())
            .collect::<Vec<_>>();

        let isolation_policy_json = serde_json::to_value(definition.manifest().isolation_policy())
            .map_err(|error| {
                AppError::Internal(format!(
                    "failed to serialize isolation policy for extension '{}': {error}",
                    definition.manifest().logical_name().as_str()
                ))
            })?;

        sqlx::query(
            r#"
            INSERT INTO extension_definitions (
                tenant_id,
                logical_name,
                display_name,
                package_version,
                runtime_api_version,
                runtime_kind,
                package_sha256,
                lifecycle_state,
                requested_capabilities,
                isolation_policy_json,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (tenant_id, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                package_version = EXCLUDED.package_version,
                runtime_api_version = EXCLUDED.runtime_api_version,
                runtime_kind = EXCLUDED.runtime_kind,
                package_sha256 = EXCLUDED.package_sha256,
                lifecycle_state = EXCLUDED.lifecycle_state,
                requested_capabilities = EXCLUDED.requested_capabilities,
                isolation_policy_json = EXCLUDED.isolation_policy_json,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(definition.manifest().logical_name().as_str())
        .bind(definition.manifest().display_name().as_str())
        .bind(definition.manifest().package_version().as_str())
        .bind(definition.manifest().runtime_api_version().as_str())
        .bind(definition.manifest().runtime_kind().as_str())
        .bind(definition.package_sha256().as_str())
        .bind(extension_lifecycle_state_to_str(
            definition.lifecycle_state(),
        ))
        .bind(requested_capabilities)
        .bind(isolation_policy_json)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save extension '{}' in tenant '{}': {error}",
                definition.manifest().logical_name().as_str(),
                tenant_id
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped extension save transaction: {error}"
            ))
        })?;

        Ok(())
    }

    async fn find_extension(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<ExtensionDefinition>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let row = sqlx::query_as::<_, ExtensionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                package_version,
                runtime_api_version,
                runtime_kind,
                package_sha256,
                lifecycle_state,
                requested_capabilities,
                isolation_policy_json
            FROM extension_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find extension '{}' in tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped extension lookup transaction: {error}"
            ))
        })?;

        row.map(extension_definition_from_row).transpose()
    }

    async fn list_extensions(&self, tenant_id: TenantId) -> AppResult<Vec<ExtensionDefinition>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let rows = sqlx::query_as::<_, ExtensionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                package_version,
                runtime_api_version,
                runtime_kind,
                package_sha256,
                lifecycle_state,
                requested_capabilities,
                isolation_policy_json
            FROM extension_definitions
            WHERE tenant_id = $1
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list extensions in tenant '{}': {error}",
                tenant_id
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped extension list transaction: {error}"
            ))
        })?;

        rows.into_iter()
            .map(extension_definition_from_row)
            .collect()
    }
}

fn extension_definition_from_row(row: ExtensionRow) -> AppResult<ExtensionDefinition> {
    let runtime_kind = ExtensionRuntimeKind::from_str(row.runtime_kind.as_str())?;

    let requested_capabilities = row
        .requested_capabilities
        .iter()
        .map(|value| ExtensionCapability::from_str(value.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let isolation_policy: ExtensionIsolationPolicy =
        serde_json::from_value(row.isolation_policy_json).map_err(|error| {
            AppError::Internal(format!(
                "persisted isolation policy for extension '{}' is invalid: {error}",
                row.logical_name
            ))
        })?;

    let manifest = ExtensionManifest::new(ExtensionManifestInput {
        logical_name: row.logical_name,
        display_name: row.display_name,
        package_version: row.package_version,
        runtime_api_version: row.runtime_api_version,
        runtime_kind,
        requested_capabilities,
        isolation_policy,
    })?;

    let definition = ExtensionDefinition::new(manifest, row.package_sha256)?;

    let lifecycle_state = extension_lifecycle_state_from_str(row.lifecycle_state.as_str())?;
    Ok(match lifecycle_state {
        ExtensionLifecycleState::Draft => definition,
        ExtensionLifecycleState::Published => definition.publish(),
        ExtensionLifecycleState::Disabled => definition.disable(),
    })
}

fn extension_lifecycle_state_to_str(state: ExtensionLifecycleState) -> &'static str {
    match state {
        ExtensionLifecycleState::Draft => "draft",
        ExtensionLifecycleState::Published => "published",
        ExtensionLifecycleState::Disabled => "disabled",
    }
}

fn extension_lifecycle_state_from_str(value: &str) -> AppResult<ExtensionLifecycleState> {
    match value {
        "draft" => Ok(ExtensionLifecycleState::Draft),
        "published" => Ok(ExtensionLifecycleState::Published),
        "disabled" => Ok(ExtensionLifecycleState::Disabled),
        _ => Err(AppError::Internal(format!(
            "unknown persisted extension lifecycle state '{}'",
            value
        ))),
    }
}

#[cfg(test)]
mod tests;
