use super::*;

impl MetadataService {
    pub(super) fn unique_values_for_record(
        schema: &PublishedEntitySchema,
        data: &Value,
    ) -> AppResult<Vec<UniqueFieldValue>> {
        let object = data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;
        let mut values = Vec::new();

        for field in schema.fields() {
            if !field.is_unique() {
                continue;
            }

            let Some(value) = object.get(field.logical_name().as_str()) else {
                continue;
            };

            values.push(UniqueFieldValue {
                field_logical_name: field.logical_name().as_str().to_owned(),
                field_value_hash: Self::hash_json_value(value)?,
            });
        }

        values.sort_by(|left, right| {
            left.field_logical_name
                .as_str()
                .cmp(right.field_logical_name.as_str())
        });

        Ok(values)
    }

    pub(super) fn hash_json_value(value: &Value) -> AppResult<String> {
        let encoded = serde_json::to_vec(value).map_err(|error| {
            AppError::Internal(format!(
                "failed to encode unique field value hash input: {error}"
            ))
        })?;

        let digest = Sha256::digest(encoded);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub(super) async fn validate_relation_values(
        &self,
        schema: &PublishedEntitySchema,
        tenant_id: TenantId,
        data: &Value,
    ) -> AppResult<()> {
        let object = data.as_object().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;

        for field in schema.fields() {
            if field.field_type() != FieldType::Relation {
                continue;
            }

            let Some(relation_target) = field.relation_target_entity() else {
                continue;
            };
            let Some(value) = object.get(field.logical_name().as_str()) else {
                continue;
            };
            let Some(record_id) = value.as_str() else {
                continue;
            };

            let exists = self
                .repository
                .runtime_record_exists(tenant_id, relation_target.as_str(), record_id)
                .await?;

            if !exists {
                return Err(AppError::Validation(format!(
                    "relation field '{}' references missing record '{}' in entity '{}'",
                    field.logical_name().as_str(),
                    record_id,
                    relation_target.as_str()
                )));
            }
        }

        Ok(())
    }
}
