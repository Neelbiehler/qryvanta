use super::*;

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
                normalized_data,
                unique_values,
                actor.subject(),
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
                normalized_data,
                unique_values,
                actor.subject(),
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
                normalized_data,
                unique_values,
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
                normalized_data,
                unique_values,
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
            .delete_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
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
            .delete_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
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
}
