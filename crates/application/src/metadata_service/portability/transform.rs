use std::collections::HashMap;

use serde_json::Map;
use uuid::Uuid;

use super::*;

impl MetadataService {
    pub(super) fn rewrite_relation_values(
        value: Value,
        relation_fields: &HashMap<String, String>,
        record_id_map: &HashMap<(String, String), String>,
    ) -> AppResult<Value> {
        let mut object = value.as_object().cloned().ok_or_else(|| {
            AppError::Validation("runtime record payload must be a JSON object".to_owned())
        })?;

        for (field_logical_name, target_entity_logical_name) in relation_fields {
            let Some(existing_value) = object.get(field_logical_name).cloned() else {
                continue;
            };
            let Some(existing_record_id) = existing_value.as_str() else {
                continue;
            };

            if let Some(remapped_record_id) = record_id_map.get(&(
                target_entity_logical_name.clone(),
                existing_record_id.to_owned(),
            )) {
                object.insert(
                    field_logical_name.clone(),
                    Value::String(remapped_record_id.clone()),
                );
            }
        }

        Ok(Value::Object(object))
    }

    pub(super) fn payload_sha256(payload: &WorkspacePortablePayload) -> AppResult<String> {
        let payload_value = serde_json::to_value(payload).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize workspace portability payload: {error}"
            ))
        })?;
        let canonical_payload = Self::canonicalize_json_value(payload_value);
        Self::hash_json_value(&canonical_payload)
    }

    pub(super) fn canonicalize_json_value(value: Value) -> Value {
        match value {
            Value::Object(object) => {
                let mut keys = object.keys().cloned().collect::<Vec<_>>();
                keys.sort();

                let mut canonical_object = Map::new();
                for key in keys {
                    if let Some(entry_value) = object.get(key.as_str()).cloned() {
                        canonical_object.insert(key, Self::canonicalize_json_value(entry_value));
                    }
                }

                Value::Object(canonical_object)
            }
            Value::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(Self::canonicalize_json_value)
                    .collect(),
            ),
            other => other,
        }
    }

    pub(super) fn deterministic_record_id(
        tenant_id: TenantId,
        entity_logical_name: &str,
        source_record_id: &str,
    ) -> String {
        let digest = Sha256::digest(format!(
            "{}:{}:{}",
            tenant_id, entity_logical_name, source_record_id
        ));

        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&digest[..16]);

        // Set RFC4122 variant + v5 UUID bits to produce a stable UUID-shaped identifier.
        bytes[6] = (bytes[6] & 0x0f) | 0x50;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Uuid::from_bytes(bytes).to_string()
    }
}
