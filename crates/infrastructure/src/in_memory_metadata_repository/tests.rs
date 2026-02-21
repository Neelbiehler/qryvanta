use qryvanta_application::{
    MetadataRepository, RecordListQuery, RuntimeRecordFilter, RuntimeRecordQuery, UniqueFieldValue,
};
use qryvanta_core::{AppError, TenantId};
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
async fn runtime_record_queries_do_not_leak_across_tenants() {
    let repository = InMemoryMetadataRepository::new();
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();

    let left_entity = EntityDefinition::new("contact", "Contact");
    assert!(left_entity.is_ok());
    let right_entity = EntityDefinition::new("contact", "Contact");
    assert!(right_entity.is_ok());

    assert!(
        repository
            .save_entity(left_tenant, left_entity.unwrap_or_else(|_| unreachable!()))
            .await
            .is_ok()
    );
    assert!(
        repository
            .save_entity(
                right_tenant,
                right_entity.unwrap_or_else(|_| unreachable!())
            )
            .await
            .is_ok()
    );

    let left_record = repository
        .create_runtime_record(left_tenant, "contact", json!({"name": "Alice"}), Vec::new())
        .await;
    assert!(left_record.is_ok());
    let left_record = left_record.unwrap_or_else(|_| unreachable!());

    let right_listed = repository
        .list_runtime_records(
            right_tenant,
            "contact",
            RecordListQuery {
                limit: 50,
                offset: 0,
            },
        )
        .await;
    assert!(right_listed.is_ok());
    assert!(right_listed.unwrap_or_default().is_empty());

    let right_queried = repository
        .query_runtime_records(
            right_tenant,
            "contact",
            RuntimeRecordQuery {
                limit: 50,
                offset: 0,
                filters: vec![RuntimeRecordFilter {
                    field_logical_name: "name".to_owned(),
                    field_value: json!("Alice"),
                }],
            },
        )
        .await;
    assert!(right_queried.is_ok());
    assert!(right_queried.unwrap_or_default().is_empty());

    let right_found = repository
        .find_runtime_record(right_tenant, "contact", left_record.record_id().as_str())
        .await;
    assert!(right_found.is_ok());
    assert!(right_found.unwrap_or_default().is_none());

    let right_exists = repository
        .runtime_record_exists(right_tenant, "contact", left_record.record_id().as_str())
        .await;
    assert!(right_exists.is_ok());
    assert!(!right_exists.unwrap_or(true));

    let right_delete = repository
        .delete_runtime_record(right_tenant, "contact", left_record.record_id().as_str())
        .await;
    assert!(matches!(right_delete, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn query_runtime_records_filters_and_paginates() {
    let repository = InMemoryMetadataRepository::new();
    let tenant_id = TenantId::new();

    assert!(
        repository
            .create_runtime_record(
                tenant_id,
                "contact",
                json!({"name": "Alice", "active": true}),
                Vec::new()
            )
            .await
            .is_ok()
    );
    assert!(
        repository
            .create_runtime_record(
                tenant_id,
                "contact",
                json!({"name": "Bob", "active": false}),
                Vec::new()
            )
            .await
            .is_ok()
    );
    assert!(
        repository
            .create_runtime_record(
                tenant_id,
                "contact",
                json!({"name": "Carol", "active": true}),
                Vec::new()
            )
            .await
            .is_ok()
    );

    let queried = repository
        .query_runtime_records(
            tenant_id,
            "contact",
            RuntimeRecordQuery {
                limit: 1,
                offset: 1,
                filters: vec![RuntimeRecordFilter {
                    field_logical_name: "active".to_owned(),
                    field_value: json!(true),
                }],
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
async fn relation_reference_check_detects_incoming_reference() {
    let repository = InMemoryMetadataRepository::new();
    let tenant_id = TenantId::new();

    let contact = EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
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

#[tokio::test]
async fn relation_reference_check_does_not_leak_across_tenants() {
    let repository = InMemoryMetadataRepository::new();
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();

    let left_contact =
        EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
    let left_deal = EntityDefinition::new("deal", "Deal").unwrap_or_else(|_| unreachable!());
    let right_contact =
        EntityDefinition::new("contact", "Contact").unwrap_or_else(|_| unreachable!());
    let right_deal = EntityDefinition::new("deal", "Deal").unwrap_or_else(|_| unreachable!());

    assert!(
        repository
            .save_entity(left_tenant, left_contact)
            .await
            .is_ok()
    );
    assert!(
        repository
            .save_entity(left_tenant, left_deal.clone())
            .await
            .is_ok()
    );
    assert!(
        repository
            .save_entity(right_tenant, right_contact)
            .await
            .is_ok()
    );
    assert!(
        repository
            .save_entity(right_tenant, right_deal.clone())
            .await
            .is_ok()
    );

    let left_relation_field = EntityFieldDefinition::new(
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
    let right_relation_field = EntityFieldDefinition::new(
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
            .save_field(left_tenant, left_relation_field)
            .await
            .is_ok()
    );
    assert!(
        repository
            .save_field(right_tenant, right_relation_field)
            .await
            .is_ok()
    );

    assert!(
        repository
            .publish_entity_schema(
                left_tenant,
                left_deal,
                repository
                    .list_fields(left_tenant, "deal")
                    .await
                    .unwrap_or_default(),
                "alice",
            )
            .await
            .is_ok()
    );
    assert!(
        repository
            .publish_entity_schema(
                right_tenant,
                right_deal,
                repository
                    .list_fields(right_tenant, "deal")
                    .await
                    .unwrap_or_default(),
                "alice",
            )
            .await
            .is_ok()
    );

    let left_contact_record = repository
        .create_runtime_record(left_tenant, "contact", json!({"name": "Alice"}), Vec::new())
        .await;
    assert!(left_contact_record.is_ok());
    let left_contact_record = left_contact_record.unwrap_or_else(|_| unreachable!());

    assert!(
        repository
            .create_runtime_record(
                right_tenant,
                "deal",
                json!({"owner_contact_id": left_contact_record.record_id().as_str()}),
                Vec::new(),
            )
            .await
            .is_ok()
    );

    let cross_tenant_reference = repository
        .has_relation_reference(
            left_tenant,
            "contact",
            left_contact_record.record_id().as_str(),
        )
        .await;
    assert!(cross_tenant_reference.is_ok());
    assert!(!cross_tenant_reference.unwrap_or(true));

    assert!(
        repository
            .create_runtime_record(
                left_tenant,
                "deal",
                json!({"owner_contact_id": left_contact_record.record_id().as_str()}),
                Vec::new(),
            )
            .await
            .is_ok()
    );

    let in_tenant_reference = repository
        .has_relation_reference(
            left_tenant,
            "contact",
            left_contact_record.record_id().as_str(),
        )
        .await;
    assert!(in_tenant_reference.is_ok());
    assert!(in_tenant_reference.unwrap_or(false));
}
