use super::*;

impl InMemoryMetadataRepository {
    pub(in super::super) async fn has_relation_reference_impl(
        &self,
        tenant_id: TenantId,
        target_entity_logical_name: &str,
        target_record_id: &str,
    ) -> AppResult<bool> {
        let published_schemas = self.published_schemas.read().await;
        let runtime_records = self.runtime_records.read().await;

        for ((schema_tenant_id, _), versions) in published_schemas.iter() {
            if schema_tenant_id != &tenant_id {
                continue;
            }

            let Some(schema) = versions.last() else {
                continue;
            };

            let relation_fields: Vec<&EntityFieldDefinition> = schema
                .fields()
                .iter()
                .filter(|field| {
                    field.field_type() == FieldType::Relation
                        && field
                            .relation_target_entity()
                            .map(|target| target.as_str() == target_entity_logical_name)
                            .unwrap_or(false)
                })
                .collect();

            if relation_fields.is_empty() {
                continue;
            }

            for ((record_tenant_id, record_entity, _), record) in runtime_records.iter() {
                if record_tenant_id != &tenant_id
                    || record_entity != schema.entity().logical_name().as_str()
                {
                    continue;
                }

                let Some(data) = record.data().as_object() else {
                    continue;
                };

                if relation_fields.iter().any(|field| {
                    data.get(field.logical_name().as_str())
                        .and_then(Value::as_str)
                        .map(|value| value == target_record_id)
                        .unwrap_or(false)
                }) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
