use super::*;

impl MetadataService {
    pub(super) async fn runtime_read_scope_for_actor_optional(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<Option<RuntimeAccessScope>> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordRead,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::All));
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordReadOwn,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::Own));
        }

        Ok(None)
    }

    pub(super) async fn runtime_write_scope_for_actor_optional(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<Option<RuntimeAccessScope>> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWrite,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::All));
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWriteOwn,
            )
            .await?
        {
            return Ok(Some(RuntimeAccessScope::Own));
        }

        Ok(None)
    }

    pub(super) async fn runtime_read_scope_for_actor(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<RuntimeAccessScope> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordRead,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::All);
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordReadOwn,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::Own);
        }

        Err(AppError::Forbidden(format!(
            "subject '{}' is missing runtime record read permissions in tenant '{}'",
            actor.subject(),
            actor.tenant_id()
        )))
    }

    pub(super) async fn runtime_write_scope_for_actor(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<RuntimeAccessScope> {
        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWrite,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::All);
        }

        if self
            .authorization_service
            .has_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::RuntimeRecordWriteOwn,
            )
            .await?
        {
            return Ok(RuntimeAccessScope::Own);
        }

        Err(AppError::Forbidden(format!(
            "subject '{}' is missing runtime record write permissions in tenant '{}'",
            actor.subject(),
            actor.tenant_id()
        )))
    }

    pub(super) async fn runtime_field_access_for_actor(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<crate::RuntimeFieldAccess>> {
        self.authorization_service
            .runtime_field_access(actor.tenant_id(), actor.subject(), entity_logical_name)
            .await
    }

    pub(super) fn enforce_writable_fields(
        data: &Value,
        field_access: &crate::RuntimeFieldAccess,
    ) -> AppResult<()> {
        let object = data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;

        for key in object.keys() {
            if !field_access.writable_fields.contains(key.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "field '{}' is not writable for this subject",
                    key
                )));
            }
        }

        Ok(())
    }

    pub(super) fn redact_runtime_records_if_needed(
        records: Vec<RuntimeRecord>,
        field_access: Option<&crate::RuntimeFieldAccess>,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let Some(field_access) = field_access else {
            return Ok(records);
        };

        records
            .into_iter()
            .map(|record| Self::redact_runtime_record(record, field_access))
            .collect()
    }

    pub(super) fn redact_runtime_record_if_needed(
        record: RuntimeRecord,
        field_access: Option<&crate::RuntimeFieldAccess>,
    ) -> AppResult<RuntimeRecord> {
        let Some(field_access) = field_access else {
            return Ok(record);
        };

        Self::redact_runtime_record(record, field_access)
    }

    pub(super) fn redact_runtime_record(
        record: RuntimeRecord,
        field_access: &crate::RuntimeFieldAccess,
    ) -> AppResult<RuntimeRecord> {
        let mut redacted = serde_json::Map::new();

        if let Some(object) = record.data().as_object() {
            for (key, value) in object {
                if field_access.readable_fields.contains(key.as_str()) {
                    redacted.insert(key.clone(), value.clone());
                }
            }
        }

        RuntimeRecord::new(
            record.record_id().as_str(),
            record.entity_logical_name().as_str(),
            Value::Object(redacted),
        )
    }
}
