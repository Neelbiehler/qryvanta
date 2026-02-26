use super::*;

impl MetadataService {
    pub(super) async fn normalize_record_payload_with_entity_business_rules(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        schema: &PublishedEntitySchema,
        data: Value,
        existing_record_data: Option<&Value>,
    ) -> AppResult<Value> {
        let mut object = Self::normalize_record_payload_without_required(schema, data)?;
        Self::apply_calculated_field_values(schema, &mut object)?;

        let effects = self
            .evaluate_entity_business_rule_effects(
                tenant_id,
                entity_logical_name,
                &Value::Object(object.clone()),
            )
            .await?;

        Self::apply_entity_business_rule_value_patches(schema, &mut object, &effects)?;

        if let Some(existing_record_data) = existing_record_data {
            Self::preserve_hidden_or_locked_update_values(
                schema,
                existing_record_data,
                &mut object,
                &effects,
            )?;
            Self::enforce_locked_field_changes(schema, existing_record_data, &object, &effects)?;
        }

        Self::apply_calculated_field_values(schema, &mut object)?;
        Self::validate_record_values(schema, &object)?;
        Self::enforce_required_fields_with_business_rules(schema, &object, &effects)?;

        if !effects.error_messages.is_empty() {
            return Err(AppError::Validation(effects.error_messages.join(" ")));
        }

        Ok(Value::Object(object))
    }
}
