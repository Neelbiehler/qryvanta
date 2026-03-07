use super::*;

impl PostgresMetadataRepository {
    pub(in super::super) async fn create_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        created_by_subject: &str,
        workflow_event: Option<RuntimeRecordWorkflowEventInput>,
    ) -> AppResult<RuntimeRecord> {
        let generated_record_id = Uuid::new_v4();
        self.create_runtime_record_with_id_uuid_impl(
            tenant_id,
            entity_logical_name,
            generated_record_id,
            data,
            unique_values,
            created_by_subject,
            workflow_event,
        )
        .await
    }

    pub(in super::super) async fn create_runtime_record_with_id_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        created_by_subject: &str,
        workflow_event: Option<RuntimeRecordWorkflowEventInput>,
    ) -> AppResult<RuntimeRecord> {
        let parsed_record_id = parse_runtime_record_uuid(record_id)?;
        self.create_runtime_record_with_id_uuid_impl(
            tenant_id,
            entity_logical_name,
            parsed_record_id,
            data,
            unique_values,
            created_by_subject,
            workflow_event,
        )
        .await
    }

    async fn create_runtime_record_with_id_uuid_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: Uuid,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        created_by_subject: &str,
        workflow_event: Option<RuntimeRecordWorkflowEventInput>,
    ) -> AppResult<RuntimeRecord> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

        let created = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            INSERT INTO runtime_records (id, tenant_id, entity_logical_name, data, created_by_subject)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, entity_logical_name, data
            "#,
        )
        .bind(record_id)
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(&data)
        .bind(created_by_subject)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            if let sqlx::Error::Database(database_error) = &error
                && database_error.code().as_deref() == Some("23505")
            {
                return AppError::Conflict(format!(
                    "runtime record '{}' already exists for entity '{}'",
                    record_id, entity_logical_name
                ));
            }
            AppError::Internal(format!(
                "failed to create runtime record for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        index_unique_values(
            &mut transaction,
            tenant_id,
            entity_logical_name,
            created.id,
            &unique_values,
        )
        .await?;
        let created_record_id = created.id.to_string();
        enqueue_runtime_record_workflow_event(
            &mut transaction,
            tenant_id,
            entity_logical_name,
            created_record_id.as_str(),
            workflow_event,
        )
        .await?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record create transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        runtime_record_from_row(created)
    }

    pub(in super::super) async fn update_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        workflow_event: Option<RuntimeRecordWorkflowEventInput>,
    ) -> AppResult<RuntimeRecord> {
        let record_uuid = parse_runtime_record_uuid(record_id)?;

        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

        let updated = sqlx::query_as::<_, RuntimeRecordRow>(
            r#"
            UPDATE runtime_records
            SET data = $4,
                updated_at = now()
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND id = $3
            RETURNING id, entity_logical_name, data
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .bind(&data)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to update runtime record '{}' for entity '{}' in tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            ))
        })?;

        sqlx::query(
            r#"
            DELETE FROM runtime_record_unique_values
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND record_id = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(record_uuid)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to clear unique field index for runtime record '{}' in entity '{}' and tenant '{}': {error}",
                record_id, entity_logical_name, tenant_id
            ))
        })?;

        index_unique_values(
            &mut transaction,
            tenant_id,
            entity_logical_name,
            record_uuid,
            &unique_values,
        )
        .await?;
        enqueue_runtime_record_workflow_event(
            &mut transaction,
            tenant_id,
            entity_logical_name,
            record_id,
            workflow_event,
        )
        .await?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit runtime record update transaction for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        runtime_record_from_row(updated)
    }
}

pub(super) async fn enqueue_runtime_record_workflow_event(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    tenant_id: TenantId,
    entity_logical_name: &str,
    record_id: &str,
    workflow_event: Option<RuntimeRecordWorkflowEventInput>,
) -> AppResult<()> {
    let Some(workflow_event) = workflow_event else {
        return Ok(());
    };

    let payload = normalized_runtime_record_workflow_payload(
        workflow_event.payload,
        entity_logical_name,
        record_id,
    );

    sqlx::query(
        r#"
        INSERT INTO workflow_runtime_trigger_events (
            tenant_id,
            trigger_type,
            entity_logical_name,
            record_id,
            emitted_by_subject,
            payload,
            status,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'pending', now(), now())
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(workflow_event.trigger.trigger_type())
    .bind(entity_logical_name)
    .bind(record_id)
    .bind(workflow_event.emitted_by_subject)
    .bind(payload)
    .execute(&mut **transaction)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to enqueue runtime workflow event for entity '{}' record '{}' in tenant '{}': {error}",
            entity_logical_name, record_id, tenant_id
        ))
    })?;

    Ok(())
}

pub(super) fn normalized_runtime_record_workflow_payload(
    mut payload: Value,
    entity_logical_name: &str,
    record_id: &str,
) -> Value {
    if let Some(payload_object) = payload.as_object_mut() {
        payload_object.insert(
            "entity_logical_name".to_owned(),
            Value::String(entity_logical_name.to_owned()),
        );
        payload_object.insert("record_id".to_owned(), Value::String(record_id.to_owned()));
        payload_object.insert("id".to_owned(), Value::String(record_id.to_owned()));
    }

    payload
}
