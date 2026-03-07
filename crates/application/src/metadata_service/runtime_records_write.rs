use super::*;
use crate::RuntimeRecordWorkflowEventInput;
use qryvanta_domain::WorkflowTrigger;

impl MetadataService {
    /// Creates a runtime record using the latest published entity schema.
    pub async fn create_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.runtime_write_scope_for_actor(actor).await?;

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let normalized_data = self
            .normalize_record_payload_with_entity_business_rules(
                actor.tenant_id(),
                entity_logical_name,
                &schema,
                data,
                None,
            )
            .await?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .create_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                normalized_data.clone(),
                unique_values,
                actor.subject(),
                Self::runtime_record_workflow_event_input(
                    actor,
                    WorkflowTrigger::RuntimeRecordCreated {
                        entity_logical_name: entity_logical_name.to_owned(),
                    },
                    record_payload_for_created(entity_logical_name, &normalized_data, None),
                ),
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordCreated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "created runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Creates a runtime record without global permission checks.
    pub async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.runtime_write_scope_for_actor_optional(actor).await?;

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let normalized_data = self
            .normalize_record_payload_with_entity_business_rules(
                actor.tenant_id(),
                entity_logical_name,
                &schema,
                data,
                None,
            )
            .await?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .create_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                normalized_data.clone(),
                unique_values,
                actor.subject(),
                Self::runtime_record_workflow_event_input(
                    actor,
                    WorkflowTrigger::RuntimeRecordCreated {
                        entity_logical_name: entity_logical_name.to_owned(),
                    },
                    record_payload_for_created(entity_logical_name, &normalized_data, None),
                ),
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordCreated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "created runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Updates a runtime record using the latest published entity schema.
    pub async fn update_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        let write_scope = self.runtime_write_scope_for_actor(actor).await?;

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only update owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let existing_record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })?;
        let normalized_data = self
            .normalize_record_payload_with_entity_business_rules(
                actor.tenant_id(),
                entity_logical_name,
                &schema,
                data,
                Some(existing_record.data()),
            )
            .await?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .update_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                normalized_data.clone(),
                unique_values,
                Self::runtime_record_workflow_event_input(
                    actor,
                    WorkflowTrigger::RuntimeRecordUpdated {
                        entity_logical_name: entity_logical_name.to_owned(),
                    },
                    record_payload_for_updated(
                        entity_logical_name,
                        record_id,
                        Some(existing_record.data()),
                        &normalized_data,
                    ),
                ),
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordUpdated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "updated runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Updates a runtime record without global permission checks.
    pub async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        let write_scope = self
            .runtime_write_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only update owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;
        if let Some(access) = &field_access {
            Self::enforce_writable_fields(&data, access)?;
        }

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        let existing_record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })?;
        let normalized_data = self
            .normalize_record_payload_with_entity_business_rules(
                actor.tenant_id(),
                entity_logical_name,
                &schema,
                data,
                Some(existing_record.data()),
            )
            .await?;
        self.validate_relation_values(&schema, actor.tenant_id(), &normalized_data)
            .await?;
        let unique_values = Self::unique_values_for_record(&schema, &normalized_data)?;

        let record = self
            .repository
            .update_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                normalized_data.clone(),
                unique_values,
                Self::runtime_record_workflow_event_input(
                    actor,
                    WorkflowTrigger::RuntimeRecordUpdated {
                        entity_logical_name: entity_logical_name.to_owned(),
                    },
                    record_payload_for_updated(
                        entity_logical_name,
                        record_id,
                        Some(existing_record.data()),
                        &normalized_data,
                    ),
                ),
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordUpdated,
                resource_type: "runtime_record".to_owned(),
                resource_id: record.record_id().as_str().to_owned(),
                detail: Some(format!(
                    "updated runtime record '{}' for entity '{}'",
                    record.record_id().as_str(),
                    entity_logical_name
                )),
            })
            .await?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Deletes a runtime record after enforcing relation-reference safeguards.
    pub async fn delete_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        let write_scope = self.runtime_write_scope_for_actor(actor).await?;

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only delete owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let existing_record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?;
        let Some(existing_record) = existing_record else {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            )));
        };

        if self
            .repository
            .has_relation_reference(actor.tenant_id(), entity_logical_name, record_id)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "runtime record '{}' in entity '{}' cannot be deleted because it is still referenced by relation fields",
                record_id, entity_logical_name
            )));
        }

        self.repository
            .delete_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                Self::runtime_record_workflow_event_input(
                    actor,
                    WorkflowTrigger::RuntimeRecordDeleted {
                        entity_logical_name: entity_logical_name.to_owned(),
                    },
                    record_payload_for_deleted(
                        entity_logical_name,
                        record_id,
                        Some(existing_record.data()),
                    ),
                ),
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordDeleted,
                resource_type: "runtime_record".to_owned(),
                resource_id: record_id.to_owned(),
                detail: Some(format!(
                    "deleted runtime record '{}' for entity '{}'",
                    record_id, entity_logical_name
                )),
            })
            .await?;

        Ok(())
    }

    /// Deletes a runtime record without global permission checks.
    pub async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        let write_scope = self
            .runtime_write_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);

        if write_scope == RuntimeAccessScope::Own
            && !self
                .repository
                .runtime_record_owned_by_subject(
                    actor.tenant_id(),
                    entity_logical_name,
                    record_id,
                    actor.subject(),
                )
                .await?
        {
            return Err(AppError::Forbidden(format!(
                "subject '{}' can only delete owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let existing_record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?;
        let Some(existing_record) = existing_record else {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            )));
        };

        if self
            .repository
            .has_relation_reference(actor.tenant_id(), entity_logical_name, record_id)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "runtime record '{}' in entity '{}' cannot be deleted because it is still referenced by relation fields",
                record_id, entity_logical_name
            )));
        }

        self.repository
            .delete_runtime_record(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                Self::runtime_record_workflow_event_input(
                    actor,
                    WorkflowTrigger::RuntimeRecordDeleted {
                        entity_logical_name: entity_logical_name.to_owned(),
                    },
                    record_payload_for_deleted(
                        entity_logical_name,
                        record_id,
                        Some(existing_record.data()),
                    ),
                ),
            )
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::RuntimeRecordDeleted,
                resource_type: "runtime_record".to_owned(),
                resource_id: record_id.to_owned(),
                detail: Some(format!(
                    "deleted runtime record '{}' for entity '{}'",
                    record_id, entity_logical_name
                )),
            })
            .await?;

        Ok(())
    }

    fn runtime_record_workflow_event_input(
        actor: &UserIdentity,
        trigger: WorkflowTrigger,
        payload: Value,
    ) -> Option<RuntimeRecordWorkflowEventInput> {
        if is_internal_workflow_subject(actor.subject()) {
            return None;
        }

        let record_id = payload
            .get("record_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        Some(RuntimeRecordWorkflowEventInput {
            trigger,
            record_id,
            payload,
            emitted_by_subject: actor.subject().to_owned(),
        })
    }
}

fn is_internal_workflow_subject(subject: &str) -> bool {
    subject == "workflow-runtime" || subject.starts_with("workflow-worker:")
}

fn record_payload_for_created(
    entity_logical_name: &str,
    record_data: &Value,
    record_id_override: Option<&str>,
) -> Value {
    let record_id = record_id_override
        .map(ToOwned::to_owned)
        .or_else(|| {
            record_data
                .get("id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .unwrap_or_default();
    let mut payload = serde_json::json!({
        "entity_logical_name": entity_logical_name,
        "record_id": record_id,
        "id": record_id,
        "record": record_data,
        "data": record_data,
        "event": "created",
    });

    if let Some(payload_object) = payload.as_object_mut()
        && let Some(record_object) = record_data.as_object()
    {
        for (key, value) in record_object {
            payload_object
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
    }

    payload
}

fn record_payload_for_updated(
    entity_logical_name: &str,
    record_id: &str,
    previous_data: Option<&Value>,
    current_data: &Value,
) -> Value {
    serde_json::json!({
        "entity_logical_name": entity_logical_name,
        "record_id": record_id,
        "id": record_id,
        "event": "updated",
        "previous": previous_data,
        "record": current_data,
        "data": current_data,
    })
}

fn record_payload_for_deleted(
    entity_logical_name: &str,
    record_id: &str,
    deleted_data: Option<&Value>,
) -> Value {
    serde_json::json!({
        "entity_logical_name": entity_logical_name,
        "record_id": record_id,
        "id": record_id,
        "event": "deleted",
        "record": deleted_data,
        "data": deleted_data,
    })
}
