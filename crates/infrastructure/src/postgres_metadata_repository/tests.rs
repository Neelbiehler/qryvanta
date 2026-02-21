use qryvanta_application::{
    MetadataRepository, RecordListQuery, RuntimeRecordConditionGroup, RuntimeRecordConditionNode,
    RuntimeRecordFilter, RuntimeRecordJoinType, RuntimeRecordLink, RuntimeRecordLogicalMode,
    RuntimeRecordOperator, RuntimeRecordQuery,
};
use qryvanta_core::{AppError, TenantId};
use qryvanta_domain::{EntityDefinition, EntityFieldDefinition, FieldType};
use serde_json::json;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresMetadataRepository;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn test_pool() -> Option<PgPool> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        return None;
    };

    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .connect(database_url.as_str())
        .await
    {
        Ok(pool) => pool,
        Err(error) => panic!("failed to connect to DATABASE_URL in test: {error}"),
    };

    if let Err(error) = MIGRATOR.run(&pool).await {
        panic!("failed to run migrations for postgres metadata tests: {error}");
    }

    Some(pool)
}

async fn ensure_tenant(pool: &PgPool, tenant_id: TenantId, name: &str) {
    let insert = sqlx::query(
        r#"
            INSERT INTO tenants (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id) DO NOTHING
            "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(name)
    .execute(pool)
    .await;

    assert!(insert.is_ok());
}

#[tokio::test]
async fn runtime_record_queries_are_tenant_scoped() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresMetadataRepository::new(pool.clone());
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();

    ensure_tenant(&pool, left_tenant, "Left Tenant").await;
    ensure_tenant(&pool, right_tenant, "Right Tenant").await;

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
        .create_runtime_record(
            left_tenant,
            "contact",
            json!({"name": "Alice"}),
            Vec::new(),
            "alice",
        )
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
                owner_subject: None,
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
                logical_mode: RuntimeRecordLogicalMode::And,
                where_clause: None,
                filters: vec![RuntimeRecordFilter {
                    scope_alias: None,
                    field_logical_name: "name".to_owned(),
                    operator: RuntimeRecordOperator::Eq,
                    field_type: FieldType::Text,
                    field_value: json!("Alice"),
                }],
                links: Vec::new(),
                sort: Vec::new(),
                owner_subject: None,
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
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresMetadataRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Query Tenant").await;

    let entity = EntityDefinition::new("contact", "Contact");
    assert!(entity.is_ok());
    assert!(
        repository
            .save_entity(tenant_id, entity.unwrap_or_else(|_| unreachable!()))
            .await
            .is_ok()
    );

    assert!(
        repository
            .create_runtime_record(
                tenant_id,
                "contact",
                json!({"name": "Alice", "active": true}),
                Vec::new(),
                "alice",
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
                Vec::new(),
                "alice",
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
                Vec::new(),
                "alice",
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
                logical_mode: RuntimeRecordLogicalMode::And,
                where_clause: None,
                filters: vec![RuntimeRecordFilter {
                    scope_alias: None,
                    field_logical_name: "active".to_owned(),
                    operator: RuntimeRecordOperator::Eq,
                    field_type: FieldType::Boolean,
                    field_value: json!(true),
                }],
                links: Vec::new(),
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
async fn query_runtime_records_supports_link_entity_alias_filters_and_where_groups() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresMetadataRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Query Link Tenant").await;

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

    let contact_name = EntityFieldDefinition::new(
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
    let deal_title = EntityFieldDefinition::new(
        "deal",
        "title",
        "Title",
        FieldType::Text,
        true,
        false,
        None,
        None,
    )
    .unwrap_or_else(|_| unreachable!());
    let deal_owner = EntityFieldDefinition::new(
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
    assert!(repository.save_field(tenant_id, contact_name).await.is_ok());
    assert!(repository.save_field(tenant_id, deal_title).await.is_ok());
    assert!(repository.save_field(tenant_id, deal_owner).await.is_ok());

    assert!(
        repository
            .publish_entity_schema(
                tenant_id,
                contact,
                repository
                    .list_fields(tenant_id, "contact")
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
                tenant_id,
                deal,
                repository
                    .list_fields(tenant_id, "deal")
                    .await
                    .unwrap_or_default(),
                "alice",
            )
            .await
            .is_ok()
    );

    let alice_contact = repository
        .create_runtime_record(
            tenant_id,
            "contact",
            json!({"name": "Alice"}),
            Vec::new(),
            "alice",
        )
        .await;
    assert!(alice_contact.is_ok());
    let alice_contact = alice_contact.unwrap_or_else(|_| unreachable!());

    let bob_contact = repository
        .create_runtime_record(
            tenant_id,
            "contact",
            json!({"name": "Bob"}),
            Vec::new(),
            "alice",
        )
        .await;
    assert!(bob_contact.is_ok());
    let bob_contact = bob_contact.unwrap_or_else(|_| unreachable!());

    assert!(
        repository
            .create_runtime_record(
                tenant_id,
                "deal",
                json!({"title": "Alpha", "owner_contact_id": alice_contact.record_id().as_str()}),
                Vec::new(),
                "alice",
            )
            .await
            .is_ok()
    );
    assert!(
        repository
            .create_runtime_record(
                tenant_id,
                "deal",
                json!({"title": "Beta", "owner_contact_id": bob_contact.record_id().as_str()}),
                Vec::new(),
                "alice",
            )
            .await
            .is_ok()
    );

    let queried = repository
        .query_runtime_records(
            tenant_id,
            "deal",
            RuntimeRecordQuery {
                limit: 50,
                offset: 0,
                logical_mode: RuntimeRecordLogicalMode::And,
                where_clause: Some(RuntimeRecordConditionGroup {
                    logical_mode: RuntimeRecordLogicalMode::And,
                    nodes: vec![RuntimeRecordConditionNode::Filter(RuntimeRecordFilter {
                        scope_alias: Some("owner".to_owned()),
                        field_logical_name: "name".to_owned(),
                        operator: RuntimeRecordOperator::Eq,
                        field_type: FieldType::Text,
                        field_value: json!("Alice"),
                    })],
                }),
                filters: Vec::new(),
                links: vec![RuntimeRecordLink {
                    alias: "owner".to_owned(),
                    parent_alias: None,
                    relation_field_logical_name: "owner_contact_id".to_owned(),
                    target_entity_logical_name: "contact".to_owned(),
                    join_type: RuntimeRecordJoinType::Inner,
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
            .and_then(|value| value.get("title")),
        Some(&json!("Alpha"))
    );
}

#[tokio::test]
async fn relation_reference_check_does_not_leak_across_tenants() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresMetadataRepository::new(pool.clone());
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();

    ensure_tenant(&pool, left_tenant, "Left Tenant").await;
    ensure_tenant(&pool, right_tenant, "Right Tenant").await;

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

    let left_deal_fields = repository.list_fields(left_tenant, "deal").await;
    assert!(left_deal_fields.is_ok());
    assert!(
        repository
            .publish_entity_schema(
                left_tenant,
                left_deal,
                left_deal_fields.unwrap_or_default(),
                "alice",
            )
            .await
            .is_ok()
    );

    let right_deal_fields = repository.list_fields(right_tenant, "deal").await;
    assert!(right_deal_fields.is_ok());
    assert!(
        repository
            .publish_entity_schema(
                right_tenant,
                right_deal,
                right_deal_fields.unwrap_or_default(),
                "alice",
            )
            .await
            .is_ok()
    );

    let left_contact_record = repository
        .create_runtime_record(
            left_tenant,
            "contact",
            json!({"name": "Alice"}),
            Vec::new(),
            "alice",
        )
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
                "alice",
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
                "alice",
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
