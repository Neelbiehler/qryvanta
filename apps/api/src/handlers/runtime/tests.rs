use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use qryvanta_application::{
    AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService, MetadataService,
    RuntimeFieldGrant, SaveFieldInput, TemporaryPermissionGrant,
};
use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{FieldType, Permission};
use qryvanta_infrastructure::InMemoryMetadataRepository;

use crate::dto::runtime::RuntimeRecordQuerySortRequest;
use crate::dto::{
    QueryRuntimeRecordsRequest, RuntimeRecordQueryFilterRequest, RuntimeRecordQueryGroupRequest,
    RuntimeRecordQueryLinkEntityRequest,
};
use crate::error::ApiError;

use super::runtime_record_query_from_request;

#[derive(Default)]
struct NoopAuditRepository;

#[async_trait]
impl AuditRepository for NoopAuditRepository {
    async fn append_event(&self, _event: AuditEvent) -> AppResult<()> {
        Ok(())
    }
}

struct FakeAuthorizationRepository {
    grants: HashMap<(TenantId, String), Vec<Permission>>,
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
        _tenant_id: TenantId,
        _subject: &str,
        _entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        Ok(Vec::new())
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

async fn seed_metadata_service() -> (MetadataService, UserIdentity) {
    let tenant_id = TenantId::new();
    let subject = "query-tester";
    let actor = UserIdentity::new(subject, subject, None, tenant_id);
    let audit_repository = Arc::new(NoopAuditRepository);
    let authorization_service = AuthorizationService::new(
        Arc::new(FakeAuthorizationRepository {
            grants: HashMap::from([(
                (tenant_id, subject.to_owned()),
                vec![
                    Permission::MetadataEntityCreate,
                    Permission::MetadataFieldWrite,
                    Permission::RuntimeRecordWrite,
                    Permission::RuntimeRecordRead,
                ],
            )]),
        }),
        audit_repository.clone(),
    );
    let metadata_service = MetadataService::new(
        Arc::new(InMemoryMetadataRepository::new()),
        authorization_service,
        audit_repository,
    );

    assert!(
        metadata_service
            .register_entity(&actor, "contact", "Contact")
            .await
            .is_ok()
    );
    assert!(
        metadata_service
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
                    calculation_expression: None,
                    relation_target_entity: None,
                    option_set_logical_name: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(
        metadata_service
            .register_entity(&actor, "deal", "Deal")
            .await
            .is_ok()
    );
    assert!(
        metadata_service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "deal".to_owned(),
                    logical_name: "title".to_owned(),
                    display_name: "Title".to_owned(),
                    field_type: FieldType::Text,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    calculation_expression: None,
                    relation_target_entity: None,
                    option_set_logical_name: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(
        metadata_service
            .save_field(
                &actor,
                SaveFieldInput {
                    entity_logical_name: "deal".to_owned(),
                    logical_name: "owner_contact_id".to_owned(),
                    display_name: "Owner".to_owned(),
                    field_type: FieldType::Relation,
                    is_required: true,
                    is_unique: false,
                    default_value: None,
                    calculation_expression: None,
                    relation_target_entity: Some("contact".to_owned()),
                    option_set_logical_name: None,
                },
            )
            .await
            .is_ok()
    );
    assert!(
        metadata_service
            .publish_entity(&actor, "contact")
            .await
            .is_ok()
    );
    assert!(
        metadata_service
            .publish_entity(&actor, "deal")
            .await
            .is_ok()
    );

    (metadata_service, actor)
}

#[tokio::test]
async fn runtime_query_payload_rejects_unpublished_entity() {
    let (metadata_service, actor) = seed_metadata_service().await;

    let result = runtime_record_query_from_request(
        &metadata_service,
        &actor,
        "not_published",
        QueryRuntimeRecordsRequest {
            limit: Some(25),
            offset: Some(0),
            logical_mode: Some("and".to_owned()),
            where_clause: None,
            conditions: None,
            link_entities: None,
            sort: None,
            filters: None,
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn runtime_query_validation_maps_to_bad_request_response() {
    let (metadata_service, actor) = seed_metadata_service().await;

    let result = runtime_record_query_from_request(
        &metadata_service,
        &actor,
        "contact",
        QueryRuntimeRecordsRequest {
            limit: Some(25),
            offset: Some(0),
            logical_mode: Some("xor".to_owned()),
            where_clause: None,
            conditions: None,
            link_entities: None,
            sort: None,
            filters: None,
        },
    )
    .await;

    let response = ApiError::from(result.err().unwrap_or_else(|| unreachable!())).into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn runtime_query_full_where_and_link_entities_executes() {
    let (metadata_service, actor) = seed_metadata_service().await;

    let alice_contact = metadata_service
        .create_runtime_record(&actor, "contact", serde_json::json!({"name": "Alice"}))
        .await;
    assert!(alice_contact.is_ok());
    let alice_contact = alice_contact.unwrap_or_else(|_| unreachable!());

    let bob_contact = metadata_service
        .create_runtime_record(&actor, "contact", serde_json::json!({"name": "Bob"}))
        .await;
    assert!(bob_contact.is_ok());
    let bob_contact = bob_contact.unwrap_or_else(|_| unreachable!());

    assert!(
        metadata_service
            .create_runtime_record(
                &actor,
                "deal",
                serde_json::json!({
                    "title": "Alpha",
                    "owner_contact_id": alice_contact.record_id().as_str()
                }),
            )
            .await
            .is_ok()
    );
    assert!(
        metadata_service
            .create_runtime_record(
                &actor,
                "deal",
                serde_json::json!({
                    "title": "Beta",
                    "owner_contact_id": bob_contact.record_id().as_str()
                }),
            )
            .await
            .is_ok()
    );

    let query = runtime_record_query_from_request(
        &metadata_service,
        &actor,
        "deal",
        QueryRuntimeRecordsRequest {
            limit: Some(50),
            offset: Some(0),
            logical_mode: Some("and".to_owned()),
            where_clause: Some(RuntimeRecordQueryGroupRequest {
                logical_mode: Some("and".to_owned()),
                conditions: Some(vec![RuntimeRecordQueryFilterRequest {
                    scope_alias: Some("owner".to_owned()),
                    field_logical_name: "name".to_owned(),
                    operator: "eq".to_owned(),
                    field_value: serde_json::json!("Alice"),
                }]),
                groups: None,
            }),
            conditions: Some(vec![RuntimeRecordQueryFilterRequest {
                scope_alias: None,
                field_logical_name: "title".to_owned(),
                operator: "contains".to_owned(),
                field_value: serde_json::json!("A"),
            }]),
            link_entities: Some(vec![RuntimeRecordQueryLinkEntityRequest {
                alias: "owner".to_owned(),
                parent_alias: None,
                relation_field_logical_name: "owner_contact_id".to_owned(),
                join_type: Some("inner".to_owned()),
            }]),
            sort: Some(vec![RuntimeRecordQuerySortRequest {
                scope_alias: Some("owner".to_owned()),
                field_logical_name: "name".to_owned(),
                direction: Some("asc".to_owned()),
            }]),
            filters: None,
        },
    )
    .await;
    assert!(query.is_ok());

    let records = metadata_service
        .query_runtime_records(&actor, "deal", query.unwrap_or_else(|_| unreachable!()))
        .await;
    assert!(records.is_ok());
    let records = records.unwrap_or_default();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0]
            .data()
            .as_object()
            .and_then(|value| value.get("title")),
        Some(&serde_json::json!("Alpha"))
    );
}
