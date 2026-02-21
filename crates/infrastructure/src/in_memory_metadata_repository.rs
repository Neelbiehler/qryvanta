use std::collections::HashMap;

use async_trait::async_trait;
use qryvanta_application::{MetadataRepository, RecordListQuery, UniqueFieldValue};
use qryvanta_core::TenantId;
use qryvanta_core::{AppError, AppResult};
use qryvanta_domain::{
    EntityDefinition, EntityFieldDefinition, FieldType, PublishedEntitySchema, RuntimeRecord,
};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory metadata repository implementation.
#[derive(Debug, Default)]
pub struct InMemoryMetadataRepository {
    entities: RwLock<HashMap<(TenantId, String), EntityDefinition>>,
    fields: RwLock<HashMap<(TenantId, String, String), EntityFieldDefinition>>,
    published_schemas: RwLock<HashMap<(TenantId, String), Vec<PublishedEntitySchema>>>,
    runtime_records: RwLock<HashMap<(TenantId, String, String), RuntimeRecord>>,
    unique_values: RwLock<HashMap<(TenantId, String, String, String), String>>,
}

impl InMemoryMetadataRepository {
    /// Creates an empty in-memory repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: RwLock::new(HashMap::new()),
            fields: RwLock::new(HashMap::new()),
            published_schemas: RwLock::new(HashMap::new()),
            runtime_records: RwLock::new(HashMap::new()),
            unique_values: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MetadataRepository for InMemoryMetadataRepository {
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()> {
        let key = (tenant_id, entity.logical_name().as_str().to_owned());
        let mut entities = self.entities.write().await;

        if entities.contains_key(&key) {
            return Err(AppError::Conflict(format!(
                "entity '{}' already exists for tenant '{}'",
                key.1, key.0
            )));
        }

        entities.insert(key, entity);
        Ok(())
    }

    async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
        let entities = self.entities.read().await;

        let mut values: Vec<EntityDefinition> = entities
            .iter()
            .filter_map(|((stored_tenant_id, _), entity)| {
                (stored_tenant_id == &tenant_id).then_some(entity.clone())
            })
            .collect();
        values.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });

        Ok(values)
    }

    async fn find_entity(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<EntityDefinition>> {
        Ok(self
            .entities
            .read()
            .await
            .get(&(tenant_id, logical_name.to_owned()))
            .cloned())
    }

    async fn save_field(&self, tenant_id: TenantId, field: EntityFieldDefinition) -> AppResult<()> {
        self.fields.write().await.insert(
            (
                tenant_id,
                field.entity_logical_name().as_str().to_owned(),
                field.logical_name().as_str().to_owned(),
            ),
            field,
        );

        Ok(())
    }

    async fn list_fields(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>> {
        let fields = self.fields.read().await;
        let mut listed: Vec<EntityFieldDefinition> = fields
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), field)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(field.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    async fn publish_entity_schema(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
        _published_by: &str,
    ) -> AppResult<PublishedEntitySchema> {
        let mut published_schemas = self.published_schemas.write().await;
        let versions = published_schemas
            .entry((tenant_id, entity.logical_name().as_str().to_owned()))
            .or_default();

        let version = versions
            .last()
            .map(|schema| schema.version() + 1)
            .unwrap_or(1);
        let schema = PublishedEntitySchema::new(entity, version, fields)?;
        versions.push(schema.clone());

        Ok(schema)
    }

    async fn latest_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        Ok(self
            .published_schemas
            .read()
            .await
            .get(&(tenant_id, entity_logical_name.to_owned()))
            .and_then(|versions| versions.last().cloned()))
    }

    async fn create_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        let record = RuntimeRecord::new(Uuid::new_v4().to_string(), entity_logical_name, data)?;

        let mut unique_index = self.unique_values.write().await;
        for unique_value in &unique_values {
            let key = (
                tenant_id,
                entity_logical_name.to_owned(),
                unique_value.field_logical_name.clone(),
                unique_value.field_value_hash.clone(),
            );
            if unique_index.contains_key(&key) {
                return Err(AppError::Conflict(format!(
                    "unique constraint violated for field '{}'",
                    unique_value.field_logical_name
                )));
            }
        }

        for unique_value in unique_values {
            unique_index.insert(
                (
                    tenant_id,
                    entity_logical_name.to_owned(),
                    unique_value.field_logical_name,
                    unique_value.field_value_hash,
                ),
                record.record_id().as_str().to_owned(),
            );
        }

        self.runtime_records.write().await.insert(
            (
                tenant_id,
                entity_logical_name.to_owned(),
                record.record_id().as_str().to_owned(),
            ),
            record.clone(),
        );

        Ok(record)
    }

    async fn update_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
        unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        let record_key = (
            tenant_id,
            entity_logical_name.to_owned(),
            record_id.to_owned(),
        );

        if !self.runtime_records.read().await.contains_key(&record_key) {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist",
                record_id
            )));
        }

        let mut unique_index = self.unique_values.write().await;
        unique_index.retain(|(_, entity, _, _), existing_record_id| {
            !(entity == entity_logical_name && existing_record_id == record_id)
        });

        for unique_value in &unique_values {
            let key = (
                tenant_id,
                entity_logical_name.to_owned(),
                unique_value.field_logical_name.clone(),
                unique_value.field_value_hash.clone(),
            );

            if unique_index
                .get(&key)
                .map(|existing_record_id| existing_record_id.as_str() != record_id)
                .unwrap_or(false)
            {
                return Err(AppError::Conflict(format!(
                    "unique constraint violated for field '{}'",
                    unique_value.field_logical_name
                )));
            }
        }

        for unique_value in unique_values {
            unique_index.insert(
                (
                    tenant_id,
                    entity_logical_name.to_owned(),
                    unique_value.field_logical_name,
                    unique_value.field_value_hash,
                ),
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

    async fn list_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let records = self.runtime_records.read().await;
        let mut listed: Vec<RuntimeRecord> = records
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), record)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(record.clone())
            })
            .collect();

        listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));

        Ok(listed
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect())
    }

    async fn find_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<Option<RuntimeRecord>> {
        Ok(self
            .runtime_records
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                record_id.to_owned(),
            ))
            .cloned())
    }

    async fn delete_runtime_record(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        let removed = self.runtime_records.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            record_id.to_owned(),
        ));

        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist for entity '{}'",
                record_id, entity_logical_name
            )));
        }

        self.unique_values
            .write()
            .await
            .retain(|(_, entity, _, _), existing_record_id| {
                !(entity == entity_logical_name && existing_record_id == record_id)
            });

        Ok(())
    }

    async fn runtime_record_exists(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<bool> {
        Ok(self.runtime_records.read().await.contains_key(&(
            tenant_id,
            entity_logical_name.to_owned(),
            record_id.to_owned(),
        )))
    }

    async fn has_relation_reference(
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

#[cfg(test)]
mod tests {
    use qryvanta_application::{MetadataRepository, RecordListQuery, UniqueFieldValue};
    use qryvanta_core::TenantId;
    use qryvanta_domain::{EntityDefinition, EntityFieldDefinition, FieldType};
    use serde_json::json;

    use super::InMemoryMetadataRepository;

    #[tokio::test]
    async fn save_and_list_entities() {
        let repository = InMemoryMetadataRepository::new();
        let tenant_id = TenantId::new();

        let entity = EntityDefinition::new("account", "Account");
        assert!(entity.is_ok());
        let save_result = repository
            .save_entity(tenant_id, entity.unwrap_or_else(|_| unreachable!()))
            .await;
        assert!(save_result.is_ok());

        let listed = repository.list_entities(tenant_id).await;
        assert!(listed.is_ok());
        assert_eq!(listed.unwrap_or_default().len(), 1);
    }

    #[tokio::test]
    async fn list_entities_does_not_leak_across_tenants() {
        let repository = InMemoryMetadataRepository::new();
        let left_tenant = TenantId::new();
        let right_tenant = TenantId::new();

        let left_entity = EntityDefinition::new("account", "Account");
        assert!(left_entity.is_ok());
        let right_entity = EntityDefinition::new("contact", "Contact");
        assert!(right_entity.is_ok());

        let left_save_result = repository
            .save_entity(left_tenant, left_entity.unwrap_or_else(|_| unreachable!()))
            .await;
        assert!(left_save_result.is_ok());

        let right_save_result = repository
            .save_entity(
                right_tenant,
                right_entity.unwrap_or_else(|_| unreachable!()),
            )
            .await;
        assert!(right_save_result.is_ok());

        let left_listed = repository.list_entities(left_tenant).await;
        assert!(left_listed.is_ok());

        let entities = left_listed.unwrap_or_default();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].logical_name().as_str(), "account");
    }

    #[tokio::test]
    async fn runtime_record_unique_constraint_conflict() {
        let repository = InMemoryMetadataRepository::new();
        let tenant_id = TenantId::new();

        let entity = EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
        let saved = repository.save_entity(tenant_id, entity.clone()).await;
        assert!(saved.is_ok());
        let field = EntityFieldDefinition::new(
            "contact",
            "email",
            "Email",
            FieldType::Text,
            true,
            true,
            None,
            None,
        )
        .unwrap_or_else(|_| unreachable!());
        let saved_field = repository.save_field(tenant_id, field).await;
        assert!(saved_field.is_ok());
        let published = repository
            .publish_entity_schema(
                tenant_id,
                entity,
                repository
                    .list_fields(tenant_id, "contact")
                    .await
                    .unwrap_or_default(),
                "alice",
            )
            .await;
        assert!(published.is_ok());

        let first = repository
            .create_runtime_record(
                tenant_id,
                "contact",
                json!({"email": "alice@example.com"}),
                vec![UniqueFieldValue {
                    field_logical_name: "email".to_owned(),
                    field_value_hash: "same".to_owned(),
                }],
            )
            .await;
        assert!(first.is_ok());

        let second = repository
            .create_runtime_record(
                tenant_id,
                "contact",
                json!({"email": "alice@example.com"}),
                vec![UniqueFieldValue {
                    field_logical_name: "email".to_owned(),
                    field_value_hash: "same".to_owned(),
                }],
            )
            .await;
        assert!(second.is_err());
    }

    #[tokio::test]
    async fn list_runtime_records_honors_offset_and_limit() {
        let repository = InMemoryMetadataRepository::new();
        let tenant_id = TenantId::new();

        let first = repository
            .create_runtime_record(tenant_id, "contact", json!({}), Vec::new())
            .await;
        assert!(first.is_ok());

        let second = repository
            .create_runtime_record(tenant_id, "contact", json!({}), Vec::new())
            .await;
        assert!(second.is_ok());

        let listed = repository
            .list_runtime_records(
                tenant_id,
                "contact",
                RecordListQuery {
                    limit: 1,
                    offset: 1,
                },
            )
            .await;
        assert!(listed.is_ok());
        assert_eq!(listed.unwrap_or_default().len(), 1);
    }

    #[tokio::test]
    async fn relation_reference_check_detects_incoming_reference() {
        let repository = InMemoryMetadataRepository::new();
        let tenant_id = TenantId::new();

        let contact =
            EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
        let deal = EntityDefinition::new("deal", "Deal").unwrap_or_else(|_| unreachable!());
        assert!(
            repository
                .save_entity(tenant_id, contact.clone())
                .await
                .is_ok()
        );
        assert!(
            repository
                .save_entity(tenant_id, deal.clone())
                .await
                .is_ok()
        );

        let contact_field = EntityFieldDefinition::new(
            "contact",
            "name",
            "Name",
            FieldType::Text,
            true,
            false,
            None,
            None,
        )
        .unwrap_or_else(|_| unreachable!());
        assert!(
            repository
                .save_field(tenant_id, contact_field)
                .await
                .is_ok()
        );

        let relation_field = EntityFieldDefinition::new(
            "deal",
            "owner_contact_id",
            "Owner",
            FieldType::Relation,
            true,
            false,
            None,
            Some("contact".to_owned()),
        )
        .unwrap_or_else(|_| unreachable!());
        assert!(
            repository
                .save_field(tenant_id, relation_field)
                .await
                .is_ok()
        );

        let contact_publish = repository
            .publish_entity_schema(
                tenant_id,
                contact,
                repository
                    .list_fields(tenant_id, "contact")
                    .await
                    .unwrap_or_default(),
                "alice",
            )
            .await;
        assert!(contact_publish.is_ok());

        let deal_publish = repository
            .publish_entity_schema(
                tenant_id,
                deal,
                repository
                    .list_fields(tenant_id, "deal")
                    .await
                    .unwrap_or_default(),
                "alice",
            )
            .await;
        assert!(deal_publish.is_ok());

        let contact_record = repository
            .create_runtime_record(tenant_id, "contact", json!({"name": "Alice"}), Vec::new())
            .await;
        assert!(contact_record.is_ok());
        let contact_record = contact_record.unwrap_or_else(|_| unreachable!());

        let deal_record = repository
            .create_runtime_record(
                tenant_id,
                "deal",
                json!({"owner_contact_id": contact_record.record_id().as_str()}),
                Vec::new(),
            )
            .await;
        assert!(deal_record.is_ok());

        let referenced = repository
            .has_relation_reference(tenant_id, "contact", contact_record.record_id().as_str())
            .await;
        assert!(referenced.is_ok());
        assert!(referenced.unwrap_or(false));
    }
}
