use std::cmp::Ordering;
use std::collections::HashMap;

use async_trait::async_trait;
use qryvanta_application::{
    MetadataRepository, RecordListQuery, RuntimeRecordConditionGroup, RuntimeRecordConditionNode,
    RuntimeRecordFilter, RuntimeRecordJoinType, RuntimeRecordLogicalMode, RuntimeRecordOperator,
    RuntimeRecordQuery, RuntimeRecordSort, RuntimeRecordSortDirection, UniqueFieldValue,
};
use qryvanta_core::TenantId;
use qryvanta_core::{AppError, AppResult};
use qryvanta_domain::{
    EntityDefinition, EntityFieldDefinition, FieldType, FormDefinition, OptionSetDefinition,
    PublishedEntitySchema, RuntimeRecord, ViewDefinition,
};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory metadata repository implementation.
#[derive(Debug, Default)]
pub struct InMemoryMetadataRepository {
    entities: RwLock<HashMap<(TenantId, String), EntityDefinition>>,
    fields: RwLock<HashMap<(TenantId, String, String), EntityFieldDefinition>>,
    option_sets: RwLock<HashMap<(TenantId, String, String), OptionSetDefinition>>,
    forms: RwLock<HashMap<(TenantId, String, String), FormDefinition>>,
    views: RwLock<HashMap<(TenantId, String, String), ViewDefinition>>,
    published_schemas: RwLock<HashMap<(TenantId, String), Vec<PublishedEntitySchema>>>,
    runtime_records: RwLock<HashMap<(TenantId, String, String), RuntimeRecord>>,
    record_owners: RwLock<HashMap<(TenantId, String, String), String>>,
    unique_values: RwLock<HashMap<(TenantId, String, String, String), String>>,
}

impl InMemoryMetadataRepository {
    /// Creates an empty in-memory repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: RwLock::new(HashMap::new()),
            fields: RwLock::new(HashMap::new()),
            option_sets: RwLock::new(HashMap::new()),
            forms: RwLock::new(HashMap::new()),
            views: RwLock::new(HashMap::new()),
            published_schemas: RwLock::new(HashMap::new()),
            runtime_records: RwLock::new(HashMap::new()),
            record_owners: RwLock::new(HashMap::new()),
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

    async fn find_field(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<Option<EntityFieldDefinition>> {
        Ok(self
            .fields
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                field_logical_name.to_owned(),
            ))
            .cloned())
    }

    async fn delete_field(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.fields.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            field_logical_name.to_owned(),
        ));

        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "field '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, field_logical_name, tenant_id
            )));
        }

        Ok(())
    }

    async fn field_exists_in_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<bool> {
        let published = self.published_schemas.read().await;
        let Some(versions) = published.get(&(tenant_id, entity_logical_name.to_owned())) else {
            return Ok(false);
        };

        Ok(versions.iter().any(|schema| {
            schema
                .fields()
                .iter()
                .any(|field| field.logical_name().as_str() == field_logical_name)
        }))
    }

    async fn save_option_set(
        &self,
        tenant_id: TenantId,
        option_set: OptionSetDefinition,
    ) -> AppResult<()> {
        self.option_sets.write().await.insert(
            (
                tenant_id,
                option_set.entity_logical_name().as_str().to_owned(),
                option_set.logical_name().as_str().to_owned(),
            ),
            option_set,
        );
        Ok(())
    }

    async fn list_option_sets(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<OptionSetDefinition>> {
        let option_sets = self.option_sets.read().await;
        let mut listed: Vec<OptionSetDefinition> = option_sets
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), option_set)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(option_set.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    async fn find_option_set(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<Option<OptionSetDefinition>> {
        Ok(self
            .option_sets
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                option_set_logical_name.to_owned(),
            ))
            .cloned())
    }

    async fn delete_option_set(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.option_sets.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            option_set_logical_name.to_owned(),
        ));
        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "option set '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, option_set_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    async fn save_form(&self, tenant_id: TenantId, form: FormDefinition) -> AppResult<()> {
        self.forms.write().await.insert(
            (
                tenant_id,
                form.entity_logical_name().as_str().to_owned(),
                form.logical_name().as_str().to_owned(),
            ),
            form,
        );
        Ok(())
    }

    async fn list_forms(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        let forms = self.forms.read().await;
        let mut listed: Vec<FormDefinition> = forms
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), form)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(form.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    async fn find_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        Ok(self
            .forms
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                form_logical_name.to_owned(),
            ))
            .cloned())
    }

    async fn delete_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.forms.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            form_logical_name.to_owned(),
        ));
        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "form '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, form_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    async fn save_view(&self, tenant_id: TenantId, view: ViewDefinition) -> AppResult<()> {
        self.views.write().await.insert(
            (
                tenant_id,
                view.entity_logical_name().as_str().to_owned(),
                view.logical_name().as_str().to_owned(),
            ),
            view,
        );
        Ok(())
    }

    async fn list_views(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        let views = self.views.read().await;
        let mut listed: Vec<ViewDefinition> = views
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), view)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(view.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    async fn find_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        Ok(self
            .views
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                view_logical_name.to_owned(),
            ))
            .cloned())
    }

    async fn delete_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.views.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            view_logical_name.to_owned(),
        ));
        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "view '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, view_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    async fn publish_entity_schema(
        &self,
        tenant_id: TenantId,
        entity: EntityDefinition,
        fields: Vec<EntityFieldDefinition>,
        option_sets: Vec<OptionSetDefinition>,
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
        let schema = PublishedEntitySchema::new(entity, version, fields, option_sets)?;
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
        created_by_subject: &str,
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

        self.record_owners.write().await.insert(
            (
                tenant_id,
                entity_logical_name.to_owned(),
                record.record_id().as_str().to_owned(),
            ),
            created_by_subject.to_owned(),
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
        let record_owners = self.record_owners.read().await;
        let mut listed: Vec<RuntimeRecord> = records
            .iter()
            .filter_map(
                |((stored_tenant_id, stored_entity_name, stored_record_id), record)| {
                    let matches_owner = query.owner_subject.as_deref().is_none_or(|subject| {
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
            .collect();

        listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));

        Ok(listed
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect())
    }

    async fn query_runtime_records(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let records = self.runtime_records.read().await;
        let record_owners = self.record_owners.read().await;
        let runtime_index = build_runtime_record_index(&records);
        let mut listed: Vec<RuntimeRecord> = records
            .iter()
            .filter_map(
                |((stored_tenant_id, stored_entity_name, stored_record_id), record)| {
                    let matches_owner = query.owner_subject.as_deref().is_none_or(|subject| {
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
            .filter(|record| {
                let Some(scope_records) = resolve_runtime_query_scope_records(
                    &query,
                    tenant_id,
                    entity_logical_name,
                    record,
                    &runtime_index,
                ) else {
                    return false;
                };

                runtime_record_matches_filters(&scope_records, &query)
            })
            .collect();

        if query.sort.is_empty() {
            listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));
        } else {
            listed.sort_by(|left, right| {
                let left_scope_records = resolve_runtime_query_scope_records(
                    &query,
                    tenant_id,
                    entity_logical_name,
                    left,
                    &runtime_index,
                );
                let right_scope_records = resolve_runtime_query_scope_records(
                    &query,
                    tenant_id,
                    entity_logical_name,
                    right,
                    &runtime_index,
                );

                let Some(left_scope_records) = left_scope_records else {
                    return Ordering::Greater;
                };
                let Some(right_scope_records) = right_scope_records else {
                    return Ordering::Less;
                };

                for sort in &query.sort {
                    let ordering =
                        compare_values_for_sort(&left_scope_records, &right_scope_records, sort);
                    if ordering != Ordering::Equal {
                        return ordering;
                    }
                }

                left.record_id().as_str().cmp(right.record_id().as_str())
            });
        }

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

        self.record_owners.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            record_id.to_owned(),
        ));

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

    async fn runtime_record_owned_by_subject(
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
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                record_id.to_owned(),
            ))
            .map(|owner| owner == subject)
            .unwrap_or(false))
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

fn build_runtime_record_index(
    records: &HashMap<(TenantId, String, String), RuntimeRecord>,
) -> HashMap<(TenantId, String, String), RuntimeRecord> {
    records.clone()
}

fn resolve_runtime_query_scope_records(
    query: &RuntimeRecordQuery,
    tenant_id: TenantId,
    root_entity_logical_name: &str,
    root_record: &RuntimeRecord,
    runtime_index: &HashMap<(TenantId, String, String), RuntimeRecord>,
) -> Option<HashMap<String, Option<RuntimeRecord>>> {
    let mut scope_records = HashMap::new();
    scope_records.insert(String::new(), Some(root_record.clone()));

    for link in &query.links {
        let parent_scope_key = link.parent_alias.clone().unwrap_or_default();
        let Some(parent_record) = scope_records
            .get(parent_scope_key.as_str())
            .and_then(|record| record.clone())
        else {
            if link.join_type == RuntimeRecordJoinType::Inner {
                return None;
            }

            scope_records.insert(link.alias.clone(), None);
            continue;
        };

        let relation_target_record_id = parent_record
            .data()
            .as_object()
            .and_then(|data| data.get(link.relation_field_logical_name.as_str()))
            .and_then(Value::as_str)
            .map(str::to_owned);

        let linked_record = relation_target_record_id.and_then(|record_id| {
            runtime_index
                .get(&(
                    tenant_id,
                    link.target_entity_logical_name.clone(),
                    record_id,
                ))
                .cloned()
        });

        if linked_record.is_none() && link.join_type == RuntimeRecordJoinType::Inner {
            return None;
        }

        scope_records.insert(link.alias.clone(), linked_record);
    }

    if root_entity_logical_name != root_record.entity_logical_name().as_str() {
        return None;
    }

    Some(scope_records)
}

fn runtime_record_matches_filters(
    scope_records: &HashMap<String, Option<RuntimeRecord>>,
    query: &RuntimeRecordQuery,
) -> bool {
    let matches_flat_filters = if query.filters.is_empty() {
        true
    } else {
        let evaluate = |filter: &RuntimeRecordFilter| {
            let value = resolve_scope_value(
                scope_records,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
            );

            runtime_record_filter_matches_value(value, filter)
        };

        match query.logical_mode {
            RuntimeRecordLogicalMode::And => query.filters.iter().all(evaluate),
            RuntimeRecordLogicalMode::Or => query.filters.iter().any(evaluate),
        }
    };

    if !matches_flat_filters {
        return false;
    }

    query
        .where_clause
        .as_ref()
        .map(|group| runtime_record_group_matches(group, scope_records))
        .unwrap_or(true)
}

fn runtime_record_group_matches(
    group: &RuntimeRecordConditionGroup,
    scope_records: &HashMap<String, Option<RuntimeRecord>>,
) -> bool {
    let evaluate = |node: &RuntimeRecordConditionNode| match node {
        RuntimeRecordConditionNode::Filter(filter) => {
            let value = resolve_scope_value(
                scope_records,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
            );
            runtime_record_filter_matches_value(value, filter)
        }
        RuntimeRecordConditionNode::Group(nested_group) => {
            runtime_record_group_matches(nested_group, scope_records)
        }
    };

    match group.logical_mode {
        RuntimeRecordLogicalMode::And => group.nodes.iter().all(evaluate),
        RuntimeRecordLogicalMode::Or => group.nodes.iter().any(evaluate),
    }
}

fn resolve_scope_value<'a>(
    scope_records: &'a HashMap<String, Option<RuntimeRecord>>,
    scope_alias: Option<&str>,
    field_logical_name: &str,
) -> Option<&'a Value> {
    let scope_key = scope_alias.unwrap_or_default();
    scope_records
        .get(scope_key)
        .and_then(Option::as_ref)
        .and_then(|record| record.data().as_object())
        .and_then(|data| data.get(field_logical_name))
}

fn runtime_record_filter_matches_value(
    value: Option<&Value>,
    filter: &RuntimeRecordFilter,
) -> bool {
    let Some(value) = value else {
        return false;
    };

    match filter.operator {
        RuntimeRecordOperator::Eq => value == &filter.field_value,
        RuntimeRecordOperator::Neq => value != &filter.field_value,
        RuntimeRecordOperator::Gt => {
            compare_filter_values(value, &filter.field_value, filter).is_gt()
        }
        RuntimeRecordOperator::Gte => {
            let comparison = compare_filter_values(value, &filter.field_value, filter);
            comparison.is_gt() || comparison.is_eq()
        }
        RuntimeRecordOperator::Lt => {
            compare_filter_values(value, &filter.field_value, filter).is_lt()
        }
        RuntimeRecordOperator::Lte => {
            let comparison = compare_filter_values(value, &filter.field_value, filter);
            comparison.is_lt() || comparison.is_eq()
        }
        RuntimeRecordOperator::Contains => value
            .as_str()
            .zip(filter.field_value.as_str())
            .map(|(stored, expected)| stored.contains(expected))
            .unwrap_or(false),
        RuntimeRecordOperator::In => filter
            .field_value
            .as_array()
            .map(|values| values.iter().any(|candidate| candidate == value))
            .unwrap_or(false),
    }
}

fn compare_filter_values(
    stored: &Value,
    expected: &Value,
    filter: &RuntimeRecordFilter,
) -> Ordering {
    match filter.field_type {
        FieldType::Number => stored
            .as_f64()
            .zip(expected.as_f64())
            .and_then(|(left, right)| left.partial_cmp(&right))
            .unwrap_or(Ordering::Equal),
        FieldType::Choice => stored
            .as_i64()
            .zip(expected.as_i64())
            .map(|(left, right)| left.cmp(&right))
            .unwrap_or(Ordering::Equal),
        FieldType::MultiChoice => Ordering::Equal,
        FieldType::Date | FieldType::DateTime | FieldType::Text | FieldType::Relation => stored
            .as_str()
            .zip(expected.as_str())
            .map(|(left, right)| left.cmp(right))
            .unwrap_or(Ordering::Equal),
        FieldType::Boolean => stored
            .as_bool()
            .zip(expected.as_bool())
            .map(|(left, right)| left.cmp(&right))
            .unwrap_or(Ordering::Equal),
        FieldType::Json => Ordering::Equal,
    }
}

fn compare_values_for_sort(
    left_scope_records: &HashMap<String, Option<RuntimeRecord>>,
    right_scope_records: &HashMap<String, Option<RuntimeRecord>>,
    sort: &RuntimeRecordSort,
) -> Ordering {
    let left_value = resolve_scope_value(
        left_scope_records,
        sort.scope_alias.as_deref(),
        sort.field_logical_name.as_str(),
    );
    let right_value = resolve_scope_value(
        right_scope_records,
        sort.scope_alias.as_deref(),
        sort.field_logical_name.as_str(),
    );

    let mut ordering = match (left_value, right_value) {
        (Some(left), Some(right)) => match sort.field_type {
            FieldType::Number => left
                .as_f64()
                .zip(right.as_f64())
                .and_then(|(left, right)| left.partial_cmp(&right))
                .unwrap_or(Ordering::Equal),
            FieldType::Choice => left
                .as_i64()
                .zip(right.as_i64())
                .map(|(left, right)| left.cmp(&right))
                .unwrap_or(Ordering::Equal),
            FieldType::MultiChoice => Ordering::Equal,
            FieldType::Boolean => left
                .as_bool()
                .zip(right.as_bool())
                .map(|(left, right)| left.cmp(&right))
                .unwrap_or(Ordering::Equal),
            FieldType::Date | FieldType::DateTime | FieldType::Text | FieldType::Relation => left
                .as_str()
                .zip(right.as_str())
                .map(|(left, right)| left.cmp(right))
                .unwrap_or(Ordering::Equal),
            FieldType::Json => Ordering::Equal,
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };

    if sort.direction == RuntimeRecordSortDirection::Desc {
        ordering = ordering.reverse();
    }

    ordering
}

#[cfg(test)]
mod tests;
