use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AuditAction, BusinessRuleActionType, BusinessRuleCondition, BusinessRuleDefinition,
    BusinessRuleDefinitionInput, BusinessRuleOperator, BusinessRuleScope, EntityDefinition,
    EntityFieldDefinition, EntityFieldMutableUpdateInput, FieldType, FormDefinition,
    FormFieldPlacement, FormSection, FormTab, FormType, OptionSetDefinition, Permission,
    PublishedEntitySchema, RuntimeRecord, SortDirection, ViewColumn, ViewDefinition, ViewSort,
    ViewType,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::AuthorizationService;
use crate::metadata_ports::{
    AuditEvent, AuditRepository, MetadataRepositoryByConcern, RecordListQuery,
    RuntimeRecordConditionGroup, RuntimeRecordConditionNode, RuntimeRecordFilter,
    RuntimeRecordOperator, RuntimeRecordQuery, RuntimeRecordSort, SaveBusinessRuleInput,
    SaveFieldInput, SaveFormInput, SaveOptionSetInput, SaveViewInput, UniqueFieldValue,
    UpdateEntityInput, UpdateFieldInput,
};

/// Application service for metadata and runtime record operations.
#[derive(Clone)]
pub struct MetadataService {
    repository: Arc<dyn MetadataRepositoryByConcern>,
    authorization_service: AuthorizationService,
    audit_repository: Arc<dyn AuditRepository>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeAccessScope {
    All,
    Own,
}

#[derive(Debug, Default)]
struct EntityBusinessRuleEffects {
    required_overrides: BTreeMap<String, bool>,
    visibility_overrides: BTreeMap<String, bool>,
    lock_overrides: BTreeMap<String, bool>,
    value_patches: BTreeMap<String, Value>,
    error_messages: Vec<String>,
}

impl EntityBusinessRuleEffects {
    fn is_field_hidden(&self, field_logical_name: &str) -> bool {
        matches!(
            self.visibility_overrides.get(field_logical_name),
            Some(false)
        )
    }
}

mod definitions_business_rules;
mod definitions_components;
mod definitions_entities;
mod portability;
mod publish;
mod publish_access;
mod publish_defaults;
mod publish_validation;
mod runtime_access;
mod runtime_payload;
mod runtime_payload_calculation;
mod runtime_payload_normalization;
mod runtime_payload_option_sets;
mod runtime_payload_rules;
mod runtime_query;
mod runtime_query_links;
mod runtime_query_validation;
mod runtime_records_read;
mod runtime_records_write;
mod runtime_write;

pub use portability::{
    ExportWorkspaceBundleOptions, ImportWorkspaceBundleOptions, ImportWorkspaceBundleResult,
    PortableEntityBundle, PortableRuntimeRecord, WorkspacePortableBundle, WorkspacePortablePayload,
};

impl MetadataService {
    /// Creates a new metadata service from a repository implementation.
    #[must_use]
    pub fn new(
        repository: Arc<dyn MetadataRepositoryByConcern>,
        authorization_service: AuthorizationService,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            repository,
            authorization_service,
            audit_repository,
        }
    }

    pub(super) async fn require_entity_exists(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<()> {
        let entity = self
            .repository
            .find_entity(tenant_id, entity_logical_name)
            .await?;

        if entity.is_none() {
            return Err(AppError::NotFound(format!(
                "entity '{}' does not exist for tenant '{}'",
                entity_logical_name, tenant_id
            )));
        }

        Ok(())
    }

    pub(super) async fn published_schema_for_runtime(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<PublishedEntitySchema> {
        self.repository
            .latest_published_schema(tenant_id, entity_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "entity '{}' must be published before runtime records can be used",
                    entity_logical_name
                ))
            })
    }

    pub(crate) async fn claim_runtime_record_workflow_events(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<crate::ClaimedRuntimeRecordWorkflowEvent>> {
        self.repository
            .claim_runtime_record_workflow_events(worker_id, limit, lease_seconds, tenant_filter)
            .await
    }

    pub(crate) async fn complete_runtime_record_workflow_event(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        self.repository
            .complete_runtime_record_workflow_event(tenant_id, event_id, worker_id, lease_token)
            .await
    }

    pub(crate) async fn release_runtime_record_workflow_event(
        &self,
        tenant_id: TenantId,
        event_id: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        self.repository
            .release_runtime_record_workflow_event(
                tenant_id,
                event_id,
                worker_id,
                lease_token,
                error_message,
            )
            .await
    }
}

#[cfg(test)]
mod tests;
