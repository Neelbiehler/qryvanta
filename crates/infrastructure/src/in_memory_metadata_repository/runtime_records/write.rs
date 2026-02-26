use super::*;

impl InMemoryMetadataRepository {
    pub(in super::super) async fn create_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
        created_by_subject: &str,
    ) -> AppResult<RuntimeRecord> {
        let record = RuntimeRecord::new(Uuid::new_v4().to_string(), entity_logical_name, data)?;

        let mut unique_index = self.unique_values.write().await;
        ensure_unique_values_available(
            &unique_index,
            tenant_id,
            entity_logical_name,
            &unique_values,
            None,
        )?;

        for unique_value in unique_values {
            unique_index.insert(
                unique_value_storage_key(tenant_id, entity_logical_name, &unique_value),
                record.record_id().as_str().to_owned(),
            );
        }

        let record_key =
            runtime_record_storage_key(tenant_id, entity_logical_name, record.record_id().as_str());

        self.runtime_records
            .write()
            .await
            .insert(record_key.clone(), record.clone());

        self.record_owners
            .write()
            .await
            .insert(record_key, created_by_subject.to_owned());

        Ok(record)
    }

    pub(in super::super) async fn update_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        let record_key = runtime_record_storage_key(tenant_id, entity_logical_name, record_id);

        if !self.runtime_records.read().await.contains_key(&record_key) {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist",
                record_id
            )));
        }

        let mut unique_index = self.unique_values.write().await;
        remove_runtime_record_unique_values(&mut unique_index, entity_logical_name, record_id);

        ensure_unique_values_available(
            &unique_index,
            tenant_id,
            entity_logical_name,
            &unique_values,
            Some(record_id),
        )?;

        for unique_value in unique_values {
            unique_index.insert(
                unique_value_storage_key(tenant_id, entity_logical_name, &unique_value),
                record_id.to_owned(),
            );
        }

        let updated = RuntimeRecord::new(record_id, entity_logical_name, data)?;
        self.runtime_records
            .write()
            .await
            .insert(record_key, updated.clone());

        Ok(updated)
    }
}

fn ensure_unique_values_available(
    unique_index: &HashMap<(TenantId, String, String, String), String>,
    tenant_id: TenantId,
    entity_logical_name: &str,
    unique_values: &[UniqueFieldValue],
    current_record_id: Option<&str>,
) -> AppResult<()> {
    for unique_value in unique_values {
        let key = unique_value_storage_key(tenant_id, entity_logical_name, unique_value);
        if unique_index
            .get(&key)
            .map(|existing_record_id| {
                current_record_id
                    .map(|record_id| existing_record_id.as_str() != record_id)
                    .unwrap_or(true)
            })
            .unwrap_or(false)
        {
            return Err(runtime_record_conflict_error(
                unique_value.field_logical_name.as_str(),
            ));
        }
    }

    Ok(())
}
