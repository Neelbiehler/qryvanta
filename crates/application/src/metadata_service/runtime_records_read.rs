use super::*;

impl MetadataService {
    /// Lists runtime records for an entity.
    pub async fn list_runtime_records(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let read_scope = self.runtime_read_scope_for_actor(actor).await?;
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own {
            query.owner_subject = Some(actor.subject().to_owned());
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let records = self
            .repository
            .list_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await?;

        Self::redact_runtime_records_if_needed(records, field_access.as_ref())
    }

    /// Queries runtime records with exact-match field filters.
    pub async fn query_runtime_records(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let read_scope = self.runtime_read_scope_for_actor(actor).await?;
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own {
            query.owner_subject = Some(actor.subject().to_owned());
        }

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        self.validate_runtime_query(
            actor,
            entity_logical_name,
            &schema,
            &mut query,
            field_access.as_ref(),
        )
        .await?;

        let records = self
            .repository
            .query_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await?;

        Self::redact_runtime_records_if_needed(records, field_access.as_ref())
    }

    /// Lists runtime records without global permission checks.
    pub async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let read_scope = self
            .runtime_read_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own {
            query.owner_subject = Some(actor.subject().to_owned());
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let records = self
            .repository
            .list_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await?;

        Self::redact_runtime_records_if_needed(records, field_access.as_ref())
    }

    /// Queries runtime records without global permission checks.
    pub async fn query_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        mut query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let read_scope = self
            .runtime_read_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own {
            query.owner_subject = Some(actor.subject().to_owned());
        }

        let schema = self
            .published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;
        self.validate_runtime_query(
            actor,
            entity_logical_name,
            &schema,
            &mut query,
            field_access.as_ref(),
        )
        .await?;

        let records = self
            .repository
            .query_runtime_records(actor.tenant_id(), entity_logical_name, query)
            .await?;

        Self::redact_runtime_records_if_needed(records, field_access.as_ref())
    }

    /// Gets a runtime record by identifier.
    pub async fn get_runtime_record(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        let read_scope = self.runtime_read_scope_for_actor(actor).await?;
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own
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
                "subject '{}' can only read owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Returns whether the runtime record owner subject matches.
    pub async fn runtime_record_owned_by_subject(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool> {
        self.runtime_read_scope_for_actor(actor).await?;

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .runtime_record_owned_by_subject(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                subject,
            )
            .await
    }

    /// Gets a runtime record without global permission checks.
    pub async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        let read_scope = self
            .runtime_read_scope_for_actor_optional(actor)
            .await?
            .unwrap_or(RuntimeAccessScope::All);
        let field_access = self
            .runtime_field_access_for_actor(actor, entity_logical_name)
            .await?;

        if read_scope == RuntimeAccessScope::Own
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
                "subject '{}' can only read owned runtime records for entity '{}'",
                actor.subject(),
                entity_logical_name
            )));
        }

        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        let record = self
            .repository
            .find_runtime_record(actor.tenant_id(), entity_logical_name, record_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "runtime record '{}' does not exist for entity '{}'",
                    record_id, entity_logical_name
                ))
            })?;

        Self::redact_runtime_record_if_needed(record, field_access.as_ref())
    }

    /// Returns whether the runtime record owner subject matches without global checks.
    pub async fn runtime_record_owned_by_subject_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool> {
        self.published_schema_for_runtime(actor.tenant_id(), entity_logical_name)
            .await?;

        self.repository
            .runtime_record_owned_by_subject(
                actor.tenant_id(),
                entity_logical_name,
                record_id,
                subject,
            )
            .await
    }
}
