use super::*;

impl InMemoryMetadataRepository {
    pub(in super::super) async fn list_runtime_records_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let records = self.runtime_records.read().await;
        let record_owners = self.record_owners.read().await;
        let mut listed = collect_runtime_records_for_scope(
            &records,
            &record_owners,
            tenant_id,
            entity_logical_name,
            query.owner_subject.as_deref(),
        );

        listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));

        Ok(listed
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect())
    }

    pub(in super::super) async fn find_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<Option<RuntimeRecord>> {
        Ok(self
            .runtime_records
            .read()
            .await
            .get(&runtime_record_storage_key(
                tenant_id,
                entity_logical_name,
                record_id,
            ))
            .cloned())
    }

    pub(in super::super) async fn delete_runtime_record_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        let removed = self
            .runtime_records
            .write()
            .await
            .remove(&runtime_record_storage_key(
                tenant_id,
                entity_logical_name,
                record_id,
            ));

        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            )));
        }

        let mut unique_index = self.unique_values.write().await;
        remove_runtime_record_unique_values(&mut unique_index, entity_logical_name, record_id);

        self.record_owners
            .write()
            .await
            .remove(&runtime_record_storage_key(
                tenant_id,
                entity_logical_name,
                record_id,
            ));

        Ok(())
    }

    pub(in super::super) async fn runtime_record_exists_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<bool> {
        Ok(self
            .runtime_records
            .read()
            .await
            .contains_key(&runtime_record_storage_key(
                tenant_id,
                entity_logical_name,
                record_id,
            )))
    }

    pub(in super::super) async fn runtime_record_owned_by_subject_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        subject: &str,
    ) -> AppResult<bool> {
        Ok(self
            .record_owners
            .read()
            .await
            .get(&runtime_record_storage_key(
                tenant_id,
                entity_logical_name,
                record_id,
            ))
            .map(|owner| owner == subject)
            .unwrap_or(false))
    }
}
