use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AuditAction, EntityDefinition, EntityFieldDefinition, FieldType, Permission,
    PublishedEntitySchema, RuntimeRecord,
};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService, MetadataRepository,
    RecordListQuery, RuntimeFieldGrant, RuntimeRecordFilter, RuntimeRecordLogicalMode,
    RuntimeRecordOperator, RuntimeRecordQuery, RuntimeRecordSortDirection, SaveFieldInput,
    TemporaryPermissionGrant, UniqueFieldValue,
};

use super::MetadataService;

struct FakeRepository {
    entities: Mutex<HashMap<(TenantId, String), EntityDefinition>>,
    fields: Mutex<HashMap<(TenantId, String, String), EntityFieldDefinition>>,
    published_schemas: Mutex<HashMap<(TenantId, String), Vec<PublishedEntitySchema>>>,
    runtime_records: Mutex<HashMap<(TenantId, String, String), RuntimeRecord>>,
    record_owners: Mutex<HashMap<(TenantId, String, String), String>>,
    unique_values: Mutex<HashMap<(TenantId, String, String, String), String>>,
}

impl FakeRepository {
    fn new() -> Self {
        Self {
            entities: Mutex::new(HashMap::new()),
            fields: Mutex::new(HashMap::new()),
            published_schemas: Mutex::new(HashMap::new()),
            runtime_records: Mutex::new(HashMap::new()),
            record_owners: Mutex::new(HashMap::new()),
            unique_values: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MetadataRepository for FakeRepository {
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()> {
        let key = (tenant_id, entity.logical_name().as_str().to_owned());
        let mut entities = self.entities.lock().await;

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
        let entities = self.entities.lock().await;
        let mut listed: Vec<EntityDefinition> = entities
            .iter()
            .filter_map(|((stored_tenant_id, _), entity)| {
                (stored_tenant_id == &tenant_id).then_some(entity.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    async fn find_entity(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<EntityDefinition>> {
        Ok(self
            .entities
            .lock()
            .await
            .get(&(tenant_id, logical_name.to_owned()))
            .cloned())
    }

    async fn save_field(&self, tenant_id: TenantId, field: EntityFieldDefinition) -> AppResult<()> {
        let key = (
            tenant_id,
            field.entity_logical_name().as_str().to_owned(),
            field.logical_name().as_str().to_owned(),
        );
        self.fields.lock().await.insert(key, field);
        Ok(())
    }

    async fn list_fields(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<EntityFieldDefinition>> {
        let fields = self.fields.lock().await;
        let mut listed: Vec<EntityFieldDefinition> = fields
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity, _), field)| {
                (stored_tenant_id == &tenant_id && stored_entity == entity_logical_name)
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
        let key = (tenant_id, entity.logical_name().as_str().to_owned());
        let mut published = self.published_schemas.lock().await;
        let existing = published.entry(key).or_default();
        let version = existing
            .last()
            .map(|schema| schema.version() + 1)
            .unwrap_or(1);
        let schema = PublishedEntitySchema::new(entity, version, fields)?;
        existing.push(schema.clone());
        Ok(schema)
    }

    async fn latest_published_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        Ok(self
            .published_schemas
            .lock()
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
        let record_id = Uuid::new_v4().to_string();
        let record = RuntimeRecord::new(record_id, entity_logical_name, data)?;

        let mut unique_index = self.unique_values.lock().await;
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

        self.runtime_records.lock().await.insert(
            (
                tenant_id,
                entity_logical_name.to_owned(),
                record.record_id().as_str().to_owned(),
            ),
            record.clone(),
        );

        self.record_owners.lock().await.insert(
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
        if !self.runtime_records.lock().await.contains_key(&record_key) {
            return Err(AppError::NotFound(format!(
                "runtime record '{}' does not exist",
                record_id
            )));
        }

        let mut unique_index = self.unique_values.lock().await;
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
            .lock()
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
        let records = self.runtime_records.lock().await;
        let record_owners = self.record_owners.lock().await;
        let mut listed: Vec<RuntimeRecord> = records
            .iter()
            .filter_map(
                |((stored_tenant_id, stored_entity, stored_record_id), record)| {
                    let matches_owner = query.owner_subject.as_deref().is_none_or(|subject| {
                        record_owners
                            .get(&(
                                *stored_tenant_id,
                                stored_entity.clone(),
                                stored_record_id.clone(),
                            ))
                            .map(|owner| owner == subject)
                            .unwrap_or(false)
                    });

                    (stored_tenant_id == &tenant_id
                        && stored_entity == entity_logical_name
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
        let records = self.runtime_records.lock().await;
        let record_owners = self.record_owners.lock().await;
        let mut listed: Vec<RuntimeRecord> = records
            .iter()
            .filter_map(
                |((stored_tenant_id, stored_entity, stored_record_id), record)| {
                    let matches_owner = query.owner_subject.as_deref().is_none_or(|subject| {
                        record_owners
                            .get(&(
                                *stored_tenant_id,
                                stored_entity.clone(),
                                stored_record_id.clone(),
                            ))
                            .map(|owner| owner == subject)
                            .unwrap_or(false)
                    });

                    (stored_tenant_id == &tenant_id
                        && stored_entity == entity_logical_name
                        && matches_owner)
                        .then_some(record.clone())
                },
            )
            .filter(|record| {
                if query.filters.is_empty() {
                    return true;
                }

                let evaluate = |filter: &RuntimeRecordFilter| {
                    let value = record
                        .data()
                        .as_object()
                        .and_then(|data| data.get(filter.field_logical_name.as_str()));

                    let Some(value) = value else {
                        return false;
                    };

                    match filter.operator {
                        RuntimeRecordOperator::Eq => value == &filter.field_value,
                        RuntimeRecordOperator::Neq => value != &filter.field_value,
                        RuntimeRecordOperator::Gt => value
                            .as_f64()
                            .zip(filter.field_value.as_f64())
                            .map(|(left, right)| left > right)
                            .unwrap_or(false),
                        RuntimeRecordOperator::Gte => value
                            .as_f64()
                            .zip(filter.field_value.as_f64())
                            .map(|(left, right)| left >= right)
                            .unwrap_or(false),
                        RuntimeRecordOperator::Lt => value
                            .as_f64()
                            .zip(filter.field_value.as_f64())
                            .map(|(left, right)| left < right)
                            .unwrap_or(false),
                        RuntimeRecordOperator::Lte => value
                            .as_f64()
                            .zip(filter.field_value.as_f64())
                            .map(|(left, right)| left <= right)
                            .unwrap_or(false),
                        RuntimeRecordOperator::Contains => value
                            .as_str()
                            .zip(filter.field_value.as_str())
                            .map(|(left, right)| left.contains(right))
                            .unwrap_or(false),
                        RuntimeRecordOperator::In => filter
                            .field_value
                            .as_array()
                            .map(|values| values.iter().any(|candidate| candidate == value))
                            .unwrap_or(false),
                    }
                };

                match query.logical_mode {
                    RuntimeRecordLogicalMode::And => query.filters.iter().all(evaluate),
                    RuntimeRecordLogicalMode::Or => query.filters.iter().any(evaluate),
                }
            })
            .collect();

        if query.sort.is_empty() {
            listed.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));
        } else {
            listed.sort_by(|left, right| {
                for sort in &query.sort {
                    let left_value = left
                        .data()
                        .as_object()
                        .and_then(|data| data.get(sort.field_logical_name.as_str()));
                    let right_value = right
                        .data()
                        .as_object()
                        .and_then(|data| data.get(sort.field_logical_name.as_str()));

                    let mut ordering = left_value
                        .zip(right_value)
                        .and_then(|(left, right)| {
                            left.as_str()
                                .zip(right.as_str())
                                .map(|(left, right)| left.cmp(right))
                        })
                        .unwrap_or(std::cmp::Ordering::Equal);

                    if sort.direction == RuntimeRecordSortDirection::Desc {
                        ordering = ordering.reverse();
                    }

                    if ordering != std::cmp::Ordering::Equal {
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
            .lock()
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
        let removed = self.runtime_records.lock().await.remove(&(
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
            .lock()
            .await
            .retain(|(_, entity, _, _), existing_record_id| {
                !(entity == entity_logical_name && existing_record_id == record_id)
            });

        self.record_owners.lock().await.remove(&(
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
        Ok(self.runtime_records.lock().await.contains_key(&(
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
            .lock()
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
        let published_schemas = self.published_schemas.lock().await;
        let runtime_records = self.runtime_records.lock().await;

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

#[derive(Default)]
struct FakeAuditRepository {
    events: Mutex<Vec<AuditEvent>>,
}

#[async_trait]
impl AuditRepository for FakeAuditRepository {
    async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
        self.events.lock().await.push(event);
        Ok(())
    }
}

struct FakeAuthorizationRepository {
    grants: HashMap<(TenantId, String), Vec<Permission>>,
    runtime_field_grants: HashMap<(TenantId, String, String), Vec<RuntimeFieldGrant>>,
}

#[async_trait]
impl AuthorizationRepository for FakeAuthorizationRepository {
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>> {
        Ok(self
            .grants
            .get(&(tenant_id, subject.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn list_runtime_field_grants_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        Ok(self
            .runtime_field_grants
            .get(&(
                tenant_id,
                subject.to_owned(),
                entity_logical_name.to_owned(),
            ))
            .cloned()
            .unwrap_or_default())
    }

    async fn find_active_temporary_permission_grant(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>> {
        Ok(None)
    }
}

fn actor(tenant_id: TenantId, subject: &str) -> UserIdentity {
    UserIdentity::new(subject, subject, None, tenant_id)
}

fn build_service(
    grants: HashMap<(TenantId, String), Vec<Permission>>,
) -> (MetadataService, Arc<FakeAuditRepository>) {
    build_service_with_runtime_field_grants(grants, HashMap::new())
}

fn build_service_with_runtime_field_grants(
    grants: HashMap<(TenantId, String), Vec<Permission>>,
    runtime_field_grants: HashMap<(TenantId, String, String), Vec<RuntimeFieldGrant>>,
) -> (MetadataService, Arc<FakeAuditRepository>) {
    let audit_repository = Arc::new(FakeAuditRepository::default());
    let authorization_service = AuthorizationService::new(
        Arc::new(FakeAuthorizationRepository {
            grants,
            runtime_field_grants,
        }),
        audit_repository.clone(),
    );
    let service = MetadataService::new(
        Arc::new(FakeRepository::new()),
        authorization_service,
        audit_repository.clone(),
    );
    (service, audit_repository)
}

#[tokio::test]
async fn register_entity_persists_data_and_writes_audit_event() {
    let tenant_id = TenantId::new();
    let subject = "alice";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataEntityRead,
            Permission::MetadataFieldWrite,
        ],
    )]);
    let (service, audit_repository) = build_service(grants);
    let actor = actor(tenant_id, subject);

    let created = service.register_entity(&actor, "contact", "Contact").await;
    assert!(created.is_ok());

    let entities = service.list_entities(&actor).await;
    assert!(entities.is_ok());
    assert_eq!(entities.unwrap_or_default().len(), 1);

    let events = audit_repository.events.lock().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, AuditAction::MetadataEntityCreated);
    assert_eq!(events[0].resource_id, "contact");
}

#[tokio::test]
async fn save_field_requires_field_write_permission() {
    let tenant_id = TenantId::new();
    let subject = "bob";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![Permission::MetadataEntityCreate],
    )]);
    let (service, _) = build_service(grants);
    let actor = actor(tenant_id, subject);
    let created = service.register_entity(&actor, "account", "Account").await;
    assert!(created.is_err());
}

#[tokio::test]
async fn publish_entity_requires_fields() {
    let tenant_id = TenantId::new();
    let subject = "carol";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::MetadataFieldRead,
        ],
    )]);
    let (service, _) = build_service(grants);
    let actor = actor(tenant_id, subject);

    let created = service.register_entity(&actor, "contact", "Contact").await;
    assert!(created.is_ok());

    let publish_result = service.publish_entity(&actor, "contact").await;
    assert!(matches!(publish_result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn create_runtime_record_applies_defaults_and_writes_audit_event() {
    let tenant_id = TenantId::new();
    let subject = "dan";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordWrite,
            Permission::RuntimeRecordRead,
        ],
    )]);
    let (service, audit_repository) = build_service(grants);
    let actor = actor(tenant_id, subject);

    let created = service.register_entity(&actor, "contact", "Contact").await;
    assert!(created.is_ok());

    let saved_field = service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "contact".to_owned(),
                logical_name: "name".to_owned(),
                display_name: "Name".to_owned(),
                field_type: FieldType::Text,
                is_required: true,
                is_unique: true,
                default_value: None,
                relation_target_entity: None,
            },
        )
        .await;
    assert!(saved_field.is_ok());

    let saved_default = service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "contact".to_owned(),
                logical_name: "active".to_owned(),
                display_name: "Active".to_owned(),
                field_type: FieldType::Boolean,
                is_required: false,
                is_unique: false,
                default_value: Some(json!(true)),
                relation_target_entity: None,
            },
        )
        .await;
    assert!(saved_default.is_ok());

    let published = service.publish_entity(&actor, "contact").await;
    assert!(published.is_ok());

    let created_record = service
        .create_runtime_record(&actor, "contact", json!({"name": "Alice"}))
        .await;
    assert!(created_record.is_ok());
    let created_record = created_record.unwrap_or_else(|_| unreachable!());

    let data = created_record.data().as_object();
    assert!(data.is_some());
    assert_eq!(
        data.and_then(|object| object.get("active")),
        Some(&json!(true))
    );

    let listed = service
        .list_runtime_records(
            &actor,
            "contact",
            RecordListQuery {
                limit: 20,
                offset: 0,
                owner_subject: None,
            },
        )
        .await;
    assert!(listed.is_ok());
    assert_eq!(listed.unwrap_or_default().len(), 1);

    let events = audit_repository.events.lock().await;
    assert!(events.iter().any(|event| {
        event.action == AuditAction::RuntimeRecordCreated
            && event.resource_id == created_record.record_id().as_str()
    }));
}

#[tokio::test]
async fn query_runtime_records_filters_and_paginates() {
    let tenant_id = TenantId::new();
    let subject = "grace";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordWrite,
            Permission::RuntimeRecordRead,
        ],
    )]);
    let (service, _) = build_service(grants);
    let actor = actor(tenant_id, subject);

    assert!(
        service
            .register_entity(&actor, "contact", "Contact")
            .await
            .is_ok()
    );
    assert!(
        service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "name".to_owned(),
                    display_name: "Name".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(
        service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "active".to_owned(),
                    display_name: "Active".to_owned(),
                    field_type: FieldType::Boolean,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(service.publish_entity(&actor, "contact").await.is_ok());

    assert!(
        service
            .create_runtime_record(&actor, "contact", json!({"name": "Alice", "active": true}))
            .await
            .is_ok()
    );
    assert!(
        service
            .create_runtime_record(&actor, "contact", json!({"name": "Bob", "active": false}))
            .await
            .is_ok()
    );
    assert!(
        service
            .create_runtime_record(&actor, "contact", json!({"name": "Carol", "active": true}))
            .await
            .is_ok()
    );

    let queried = service
        .query_runtime_records(
            &actor,
            "contact",
            RuntimeRecordQuery {
                limit: 1,
                offset: 1,
                logical_mode: RuntimeRecordLogicalMode::And,
                filters: vec![RuntimeRecordFilter {
                    field_logical_name: "active".to_owned(),
                    operator: RuntimeRecordOperator::Eq,
                    field_type: FieldType::Boolean,
                    field_value: json!(true),
                }],
                sort: Vec::new(),
                owner_subject: None,
            },
        )
        .await;
    assert!(queried.is_ok());

    let queried = queried.unwrap_or_default();
    assert_eq!(queried.len(), 1);
    assert_eq!(
        queried[0]
            .data()
            .as_object()
            .and_then(|value| value.get("active")),
        Some(&json!(true))
    );
}

#[tokio::test]
async fn query_runtime_records_requires_runtime_read_permission() {
    let tenant_id = TenantId::new();
    let subject = "heidi";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordWrite,
        ],
    )]);
    let (service, _) = build_service(grants);
    let actor = actor(tenant_id, subject);

    assert!(
        service
            .register_entity(&actor, "contact", "Contact")
            .await
            .is_ok()
    );
    assert!(
        service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "name".to_owned(),
                    display_name: "Name".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(service.publish_entity(&actor, "contact").await.is_ok());

    let queried = service
        .query_runtime_records(
            &actor,
            "contact",
            RuntimeRecordQuery {
                limit: 50,
                offset: 0,
                logical_mode: RuntimeRecordLogicalMode::And,
                filters: vec![RuntimeRecordFilter {
                    field_logical_name: "name".to_owned(),
                    operator: RuntimeRecordOperator::Eq,
                    field_type: FieldType::Text,
                    field_value: json!("Alice"),
                }],
                sort: Vec::new(),
                owner_subject: None,
            },
        )
        .await;

    assert!(matches!(queried, Err(AppError::Forbidden(_))));
}

#[tokio::test]
async fn delete_runtime_record_blocks_when_relation_exists() {
    let tenant_id = TenantId::new();
    let subject = "eve";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordWrite,
            Permission::RuntimeRecordRead,
        ],
    )]);
    let (service, _) = build_service(grants);
    let actor = actor(tenant_id, subject);

    let created_contact = service.register_entity(&actor, "contact", "Contact").await;
    assert!(created_contact.is_ok());
    let created_deal = service.register_entity(&actor, "deal", "Deal").await;
    assert!(created_deal.is_ok());

    let contact_name_field = service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "contact".to_owned(),
                logical_name: "name".to_owned(),
                display_name: "Name".to_owned(),
                field_type: FieldType::Text,
                is_required: true,
                is_unique: false,
                default_value: None,
                relation_target_entity: None,
            },
        )
        .await;
    assert!(contact_name_field.is_ok());

    let deal_owner_field = service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "deal".to_owned(),
                logical_name: "owner_contact_id".to_owned(),
                display_name: "Owner Contact".to_owned(),
                field_type: FieldType::Relation,
                is_required: true,
                is_unique: false,
                default_value: None,
                relation_target_entity: Some("contact".to_owned()),
            },
        )
        .await;
    assert!(deal_owner_field.is_ok());

    let published_contact = service.publish_entity(&actor, "contact").await;
    assert!(published_contact.is_ok());
    let published_deal = service.publish_entity(&actor, "deal").await;
    assert!(published_deal.is_ok());

    let contact_record = service
        .create_runtime_record(&actor, "contact", json!({"name": "Alice"}))
        .await;
    assert!(contact_record.is_ok());
    let contact_record = contact_record.unwrap_or_else(|_| unreachable!());

    let created_deal_record = service
        .create_runtime_record(
            &actor,
            "deal",
            json!({"owner_contact_id": contact_record.record_id().as_str()}),
        )
        .await;
    assert!(created_deal_record.is_ok());

    let delete_result = service
        .delete_runtime_record(&actor, "contact", contact_record.record_id().as_str())
        .await;
    assert!(matches!(delete_result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn get_and_delete_runtime_record_succeed_when_unreferenced() {
    let tenant_id = TenantId::new();
    let subject = "frank";
    let grants = HashMap::from([(
        (tenant_id, subject.to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordWrite,
            Permission::RuntimeRecordRead,
        ],
    )]);
    let (service, audit_repository) = build_service(grants);
    let actor = actor(tenant_id, subject);

    let created_entity = service.register_entity(&actor, "note", "Note").await;
    assert!(created_entity.is_ok());
    let saved_field = service
        .save_field(
            &actor,
            SaveFieldInput {
                entity_logical_name: "note".to_owned(),
                logical_name: "title".to_owned(),
                display_name: "Title".to_owned(),
                field_type: FieldType::Text,
                is_required: true,
                is_unique: false,
                default_value: None,
                relation_target_entity: None,
            },
        )
        .await;
    assert!(saved_field.is_ok());

    let published = service.publish_entity(&actor, "note").await;
    assert!(published.is_ok());

    let created_record = service
        .create_runtime_record(&actor, "note", json!({"title": "A"}))
        .await;
    assert!(created_record.is_ok());
    let created_record = created_record.unwrap_or_else(|_| unreachable!());

    let fetched = service
        .get_runtime_record(&actor, "note", created_record.record_id().as_str())
        .await;
    assert!(fetched.is_ok());

    let deleted = service
        .delete_runtime_record(&actor, "note", created_record.record_id().as_str())
        .await;
    assert!(deleted.is_ok());

    let refetch = service
        .get_runtime_record(&actor, "note", created_record.record_id().as_str())
        .await;
    assert!(matches!(refetch, Err(AppError::NotFound(_))));

    let events = audit_repository.events.lock().await;
    assert!(events.iter().any(|event| {
        event.action == AuditAction::RuntimeRecordDeleted
            && event.resource_id == created_record.record_id().as_str()
    }));
}

#[tokio::test]
async fn list_runtime_records_unchecked_honors_own_read_scope_when_configured() {
    let tenant_id = TenantId::new();
    let grants = HashMap::from([(
        (tenant_id, "alice".to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordReadOwn,
            Permission::RuntimeRecordWriteOwn,
        ],
    )]);
    let (service, _) = build_service(grants);
    let alice = actor(tenant_id, "alice");
    let bob = actor(tenant_id, "bob");

    assert!(
        service
            .register_entity(&alice, "contact", "Contact")
            .await
            .is_ok()
    );
    assert!(
        service
            .save_field(
                &alice,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "name".to_owned(),
                    display_name: "Name".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(service.publish_entity(&alice, "contact").await.is_ok());

    let alice_record = service
        .create_runtime_record_unchecked(&alice, "contact", json!({"name": "Alice"}))
        .await;
    assert!(alice_record.is_ok());

    let bob_record = service
        .create_runtime_record_unchecked(&bob, "contact", json!({"name": "Bob"}))
        .await;
    assert!(bob_record.is_ok());

    let listed = service
        .list_runtime_records_unchecked(
            &alice,
            "contact",
            RecordListQuery {
                limit: 20,
                offset: 0,
                owner_subject: None,
            },
        )
        .await;
    assert!(listed.is_ok());
    let listed = listed.unwrap_or_default();
    assert_eq!(listed.len(), 1);
    assert_eq!(
        listed[0]
            .data()
            .as_object()
            .and_then(|value| value.get("name")),
        Some(&json!("Alice"))
    );
}

#[tokio::test]
async fn update_runtime_record_unchecked_blocks_non_owned_records_for_own_write_scope() {
    let tenant_id = TenantId::new();
    let grants = HashMap::from([(
        (tenant_id, "alice".to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordWriteOwn,
        ],
    )]);
    let (service, _) = build_service(grants);
    let alice = actor(tenant_id, "alice");
    let bob = actor(tenant_id, "bob");

    assert!(
        service
            .register_entity(&alice, "task", "Task")
            .await
            .is_ok()
    );
    assert!(
        service
            .save_field(
                &alice,
                SaveFieldInput {
                    entity_logical_name: "task".to_owned(),
                    logical_name: "title".to_owned(),
                    display_name: "Title".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(service.publish_entity(&alice, "task").await.is_ok());

    let bob_record = service
        .create_runtime_record_unchecked(&bob, "task", json!({"title": "Owned by bob"}))
        .await;
    assert!(bob_record.is_ok());
    let bob_record = bob_record.unwrap_or_else(|_| unreachable!());

    let update_result = service
        .update_runtime_record_unchecked(
            &alice,
            "task",
            bob_record.record_id().as_str(),
            json!({"title": "Alice edit"}),
        )
        .await;

    assert!(matches!(update_result, Err(AppError::Forbidden(_))));
}

#[tokio::test]
async fn get_runtime_record_unchecked_redacts_using_runtime_field_permissions() {
    let tenant_id = TenantId::new();
    let grants = HashMap::from([(
        (tenant_id, "alice".to_owned()),
        vec![
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordRead,
        ],
    )]);
    let runtime_field_grants = HashMap::from([(
        (tenant_id, "alice".to_owned(), "contact".to_owned()),
        vec![RuntimeFieldGrant {
            field_logical_name: "email".to_owned(),
            can_read: true,
            can_write: false,
        }],
    )]);
    let (service, _) = build_service_with_runtime_field_grants(grants, runtime_field_grants);
    let alice = actor(tenant_id, "alice");
    let bob = actor(tenant_id, "bob");

    assert!(
        service
            .register_entity(&alice, "contact", "Contact")
            .await
            .is_ok()
    );
    assert!(
        service
            .save_field(
                &alice,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "email".to_owned(),
                    display_name: "Email".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(
        service
            .save_field(
                &alice,
                SaveFieldInput {
                    entity_logical_name: "contact".to_owned(),
                    logical_name: "secret".to_owned(),
                    display_name: "Secret".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    relation_target_entity: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(service.publish_entity(&alice, "contact").await.is_ok());

    let created = service
        .create_runtime_record_unchecked(
            &bob,
            "contact",
            json!({"email": "a@qryvanta.dev", "secret": "top"}),
        )
        .await;
    assert!(created.is_ok());
    let created = created.unwrap_or_else(|_| unreachable!());

    let fetched = service
        .get_runtime_record_unchecked(&alice, "contact", created.record_id().as_str())
        .await;
    assert!(fetched.is_ok());
    let fetched = fetched.unwrap_or_else(|_| unreachable!());

    let data = fetched.data().as_object();
    assert!(data.is_some());
    let data = data.unwrap_or_else(|| unreachable!());
    assert_eq!(data.get("email"), Some(&json!("a@qryvanta.dev")));
    assert!(data.get("secret").is_none());
}
