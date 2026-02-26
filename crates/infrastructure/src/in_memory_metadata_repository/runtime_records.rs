use super::*;

mod query;
mod read;
mod relations;
mod write;

fn runtime_record_storage_key(
    tenant_id: TenantId,
    entity_logical_name: &str,
    record_id: &str,
) -> (TenantId, String, String) {
    (
        tenant_id,
        entity_logical_name.to_owned(),
        record_id.to_owned(),
    )
}

fn unique_value_storage_key(
    tenant_id: TenantId,
    entity_logical_name: &str,
    unique_value: &UniqueFieldValue,
) -> (TenantId, String, String, String) {
    (
        tenant_id,
        entity_logical_name.to_owned(),
        unique_value.field_logical_name.clone(),
        unique_value.field_value_hash.clone(),
    )
}

fn runtime_record_conflict_error(field_logical_name: &str) -> AppError {
    AppError::Conflict(format!(
        "unique constraint violated for field '{field_logical_name}'"
    ))
}

fn remove_runtime_record_unique_values(
    unique_index: &mut HashMap<(TenantId, String, String, String), String>,
    entity_logical_name: &str,
    record_id: &str,
) {
    unique_index.retain(|(_, entity, _, _), existing_record_id| {
        !(entity == entity_logical_name && existing_record_id == record_id)
    });
}

fn collect_runtime_records_for_scope(
    records: &HashMap<(TenantId, String, String), RuntimeRecord>,
    record_owners: &HashMap<(TenantId, String, String), String>,
    tenant_id: TenantId,
    entity_logical_name: &str,
    owner_subject: Option<&str>,
) -> Vec<RuntimeRecord> {
    records
        .iter()
        .filter_map(
            |((stored_tenant_id, stored_entity_name, stored_record_id), record)| {
                let matches_owner = owner_subject.is_none_or(|subject| {
                    record_owners
                        .get(&(
                            *stored_tenant_id,
                            stored_entity_name.clone(),
                            stored_record_id.clone(),
                        ))
                        .map(|owner| owner == subject)
                        .unwrap_or(false)
                });

                (stored_tenant_id == &tenant_id
                    && stored_entity_name == entity_logical_name
                    && matches_owner)
                    .then_some(record.clone())
            },
        )
        .collect()
}
