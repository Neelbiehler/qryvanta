use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use uuid::Uuid;

use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{
    EntityDefinition, EntityFieldDefinition, FormDefinition, OptionSetDefinition,
    PublishedEntitySchema, RuntimeRecord, ViewDefinition,
};

use crate::{
    ContactBootstrapService, MetadataRepository, RecordListQuery, RuntimeRecordQuery,
    TenantRepository, UniqueFieldValue,
};

struct FakeMetadataRepository {
    entities: Mutex<HashMap<(TenantId, String), EntityDefinition>>,
    fields: Mutex<HashMap<(TenantId, String, String), EntityFieldDefinition>>,
    option_sets: Mutex<HashMap<(TenantId, String, String), OptionSetDefinition>>,
    forms: Mutex<HashMap<(TenantId, String, String), FormDefinition>>,
    views: Mutex<HashMap<(TenantId, String, String), ViewDefinition>>,
    published_schemas: Mutex<HashMap<(TenantId, String), Vec<PublishedEntitySchema>>>,
    runtime_records: Mutex<HashMap<(TenantId, String, String), RuntimeRecord>>,
}

impl FakeMetadataRepository {
    fn new() -> Self {
        Self {
            entities: Mutex::new(HashMap::new()),
            fields: Mutex::new(HashMap::new()),
            option_sets: Mutex::new(HashMap::new()),
            forms: Mutex::new(HashMap::new()),
            views: Mutex::new(HashMap::new()),
            published_schemas: Mutex::new(HashMap::new()),
            runtime_records: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MetadataRepository for FakeMetadataRepository {
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()> {
        self.entities.lock().await.insert(
            (tenant_id, entity.logical_name().as_str().to_owned()),
            entity,
        );
        Ok(())
    }

    async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
        let entities = self.entities.lock().await;
        Ok(entities
            .iter()
            .filter_map(|((stored_tenant_id, _), entity)| {
                (stored_tenant_id == &tenant_id).then_some(entity.clone())
            })
            .collect())
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
        self.fields.lock().await.insert(
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

    async fn find_field(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        field_logical_name: &str,
    ) -> AppResult<Option<EntityFieldDefinition>> {
        Ok(self
            .fields
            .lock()
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
        let removed = self.fields.lock().await.remove(&(
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
        let published_schemas = self.published_schemas.lock().await;
        let Some(versions) = published_schemas.get(&(tenant_id, entity_logical_name.to_owned()))
        else {
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
        self.option_sets.lock().await.insert(
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
        let option_sets = self.option_sets.lock().await;
        let mut listed: Vec<OptionSetDefinition> = option_sets
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity, _), option_set)| {
                (stored_tenant_id == &tenant_id && stored_entity == entity_logical_name)
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
            .lock()
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
        let removed = self.option_sets.lock().await.remove(&(
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
        self.forms.lock().await.insert(
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
        Ok(self
            .forms
            .lock()
            .await
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity, _), form)| {
                (stored_tenant_id == &tenant_id && stored_entity == entity_logical_name)
                    .then_some(form.clone())
            })
            .collect())
    }

    async fn find_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        Ok(self
            .forms
            .lock()
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
        self.forms.lock().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            form_logical_name.to_owned(),
        ));
        Ok(())
    }

    async fn save_view(&self, tenant_id: TenantId, view: ViewDefinition) -> AppResult<()> {
        self.views.lock().await.insert(
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
        Ok(self
            .views
            .lock()
            .await
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity, _), view)| {
                (stored_tenant_id == &tenant_id && stored_entity == entity_logical_name)
                    .then_some(view.clone())
            })
            .collect())
    }

    async fn find_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        Ok(self
            .views
            .lock()
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
        self.views.lock().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            view_logical_name.to_owned(),
        ));
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
        let key = (tenant_id, entity.logical_name().as_str().to_owned());
        let mut published_schemas = self.published_schemas.lock().await;
        let next_version = published_schemas
            .get(&key)
            .and_then(|versions| versions.last().map(|schema| schema.version() + 1))
            .unwrap_or(1);
        let schema = PublishedEntitySchema::new(entity, next_version, fields, option_sets)?;
        published_schemas
            .entry(key)
            .or_default()
            .push(schema.clone());
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
        _unique_values: Vec<UniqueFieldValue>,
        _created_by_subject: &str,
    ) -> AppResult<RuntimeRecord> {
        let record = RuntimeRecord::new(Uuid::new_v4().to_string(), entity_logical_name, data)?;
        self.runtime_records.lock().await.insert(
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
        _tenant_id: TenantId,
        _entity_logical_name: &str,
        _record_id: &str,
        _data: Value,
        _unique_values: Vec<UniqueFieldValue>,
    ) -> AppResult<RuntimeRecord> {
        Err(AppError::Internal(
            "update_runtime_record is not used in contact bootstrap tests".to_owned(),
        ))
    }

    async fn list_runtime_records(
        &self,
        _tenant_id: TenantId,
        _entity_logical_name: &str,
        _query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        Ok(Vec::new())
    }

    async fn query_runtime_records(
        &self,
        _tenant_id: TenantId,
        _entity_logical_name: &str,
        _query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        Ok(Vec::new())
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
        self.runtime_records.lock().await.remove(&(
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
        _tenant_id: TenantId,
        _entity_logical_name: &str,
        _record_id: &str,
        _subject: &str,
    ) -> AppResult<bool> {
        Ok(false)
    }

    async fn has_relation_reference(
        &self,
        _tenant_id: TenantId,
        _target_entity_logical_name: &str,
        _target_record_id: &str,
    ) -> AppResult<bool> {
        Ok(false)
    }
}

#[derive(Default)]
struct FakeTenantRepository {
    mappings: Mutex<HashMap<(TenantId, String), String>>,
}

#[async_trait]
impl TenantRepository for FakeTenantRepository {
    async fn find_tenant_for_subject(&self, _subject: &str) -> AppResult<Option<TenantId>> {
        Ok(None)
    }

    async fn registration_mode_for_tenant(
        &self,
        _tenant_id: TenantId,
    ) -> AppResult<qryvanta_domain::RegistrationMode> {
        Ok(qryvanta_domain::RegistrationMode::Open)
    }

    async fn create_membership(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _display_name: &str,
        _email: Option<&str>,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn ensure_membership_for_subject(
        &self,
        _subject: &str,
        _display_name: &str,
        _email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId> {
        Ok(preferred_tenant_id.unwrap_or_default())
    }

    async fn contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Option<String>> {
        Ok(self
            .mappings
            .lock()
            .await
            .get(&(tenant_id, subject.to_owned()))
            .cloned())
    }

    async fn save_contact_record_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        contact_record_id: &str,
    ) -> AppResult<()> {
        self.mappings.lock().await.insert(
            (tenant_id, subject.to_owned()),
            contact_record_id.to_owned(),
        );
        Ok(())
    }
}

fn build_service(
    metadata_repository: Arc<FakeMetadataRepository>,
    tenant_repository: Arc<FakeTenantRepository>,
) -> ContactBootstrapService {
    ContactBootstrapService::new(metadata_repository, tenant_repository)
}

#[tokio::test]
async fn ensure_subject_contact_bootstraps_contact_schema_and_mapping() {
    let metadata_repository = Arc::new(FakeMetadataRepository::new());
    let tenant_repository = Arc::new(FakeTenantRepository::default());
    let service = build_service(metadata_repository.clone(), tenant_repository.clone());
    let tenant_id = TenantId::new();

    let record_id = service
        .ensure_subject_contact(
            tenant_id,
            "user-1",
            "User One",
            Some("user-one@example.com"),
        )
        .await;
    assert!(record_id.is_ok());
    let record_id = record_id.unwrap_or_default();

    let entity = metadata_repository.find_entity(tenant_id, "contact").await;
    assert!(entity.is_ok());
    assert!(entity.unwrap_or(None).is_some());

    let fields = metadata_repository.list_fields(tenant_id, "contact").await;
    assert!(fields.is_ok());
    let fields = fields.unwrap_or_default();
    assert_eq!(fields.len(), 3);

    let published = metadata_repository
        .latest_published_schema(tenant_id, "contact")
        .await;
    assert!(published.is_ok());
    assert!(published.unwrap_or(None).is_some());

    let mapped_record_id = tenant_repository
        .contact_record_for_subject(tenant_id, "user-1")
        .await;
    assert!(mapped_record_id.is_ok());
    assert_eq!(mapped_record_id.unwrap_or(None), Some(record_id.clone()));

    let stored_record = metadata_repository
        .find_runtime_record(tenant_id, "contact", record_id.as_str())
        .await;
    assert!(stored_record.is_ok());
    let stored_record = stored_record.unwrap_or(None);
    assert!(stored_record.is_some());
    let stored_record = stored_record.unwrap_or_else(|| unreachable!());

    assert_eq!(
        stored_record
            .data()
            .as_object()
            .and_then(|value| value.get("subject")),
        Some(&json!("user-1"))
    );
    assert_eq!(
        stored_record
            .data()
            .as_object()
            .and_then(|value| value.get("display_name")),
        Some(&json!("User One"))
    );
    assert_eq!(
        stored_record
            .data()
            .as_object()
            .and_then(|value| value.get("email")),
        Some(&json!("user-one@example.com"))
    );
}

#[tokio::test]
async fn ensure_subject_contact_is_idempotent_for_existing_mapping() {
    let metadata_repository = Arc::new(FakeMetadataRepository::new());
    let tenant_repository = Arc::new(FakeTenantRepository::default());
    let service = build_service(metadata_repository.clone(), tenant_repository.clone());
    let tenant_id = TenantId::new();

    let first_record_id = service
        .ensure_subject_contact(tenant_id, "user-2", "User Two", None)
        .await;
    assert!(first_record_id.is_ok());
    let first_record_id = first_record_id.unwrap_or_default();

    let second_record_id = service
        .ensure_subject_contact(tenant_id, "user-2", "User Two", None)
        .await;
    assert!(second_record_id.is_ok());
    let second_record_id = second_record_id.unwrap_or_default();

    assert_eq!(first_record_id, second_record_id);

    let records = metadata_repository.runtime_records.lock().await;
    let record_count = records
        .iter()
        .filter(|((stored_tenant_id, entity_name, _), _)| {
            stored_tenant_id == &tenant_id && entity_name == "contact"
        })
        .count();
    assert_eq!(record_count, 1);
}
