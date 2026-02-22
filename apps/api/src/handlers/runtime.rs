use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use qryvanta_core::AppError;
use qryvanta_core::UserIdentity;
use tracing::warn;

use crate::dto::{
    CreateRuntimeRecordRequest, QueryRuntimeRecordsRequest, RuntimeRecordResponse,
    UpdateRuntimeRecordRequest,
};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct RuntimeRecordListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_runtime_records_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Query(query): Query<RuntimeRecordListQuery>,
) -> ApiResult<Json<Vec<RuntimeRecordResponse>>> {
    let records = state
        .metadata_service
        .list_runtime_records(
            &user,
            entity_logical_name.as_str(),
            qryvanta_application::RecordListQuery {
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
                owner_subject: None,
            },
        )
        .await?
        .into_iter()
        .map(RuntimeRecordResponse::from)
        .collect();

    Ok(Json(records))
}

pub async fn create_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<CreateRuntimeRecordRequest>,
) -> ApiResult<(StatusCode, Json<RuntimeRecordResponse>)> {
    let record = state
        .metadata_service
        .create_runtime_record(&user, entity_logical_name.as_str(), payload.data)
        .await?;

    if let Err(error) = state
        .workflow_service
        .dispatch_runtime_record_created(
            &user,
            entity_logical_name.as_str(),
            record.record_id().as_str(),
        )
        .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            entity_logical_name = %entity_logical_name,
            record_id = %record.record_id().as_str(),
            "workflow dispatch failed after runtime record creation"
        );
    }

    Ok((
        StatusCode::CREATED,
        Json(RuntimeRecordResponse::from(record)),
    ))
}

pub async fn query_runtime_records_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<QueryRuntimeRecordsRequest>,
) -> ApiResult<Json<Vec<RuntimeRecordResponse>>> {
    let query = runtime_record_query_from_request(
        &state.metadata_service,
        &user,
        entity_logical_name.as_str(),
        payload,
    )
    .await?;

    let records = state
        .metadata_service
        .query_runtime_records(&user, entity_logical_name.as_str(), query)
        .await?
        .into_iter()
        .map(RuntimeRecordResponse::from)
        .collect();

    Ok(Json(records))
}

pub async fn update_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, record_id)): Path<(String, String)>,
    Json(payload): Json<UpdateRuntimeRecordRequest>,
) -> ApiResult<Json<RuntimeRecordResponse>> {
    let record = state
        .metadata_service
        .update_runtime_record(
            &user,
            entity_logical_name.as_str(),
            record_id.as_str(),
            payload.data,
        )
        .await?;

    Ok(Json(RuntimeRecordResponse::from(record)))
}

pub async fn get_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, record_id)): Path<(String, String)>,
) -> ApiResult<Json<RuntimeRecordResponse>> {
    let record = state
        .metadata_service
        .get_runtime_record(&user, entity_logical_name.as_str(), record_id.as_str())
        .await?;

    Ok(Json(RuntimeRecordResponse::from(record)))
}

pub async fn delete_runtime_record_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path((entity_logical_name, record_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    state
        .metadata_service
        .delete_runtime_record(&user, entity_logical_name.as_str(), record_id.as_str())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn runtime_record_query_from_request(
    metadata_service: &qryvanta_application::MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    payload: QueryRuntimeRecordsRequest,
) -> Result<qryvanta_application::RuntimeRecordQuery, AppError> {
    let QueryRuntimeRecordsRequest {
        limit,
        offset,
        logical_mode,
        where_clause,
        conditions,
        link_entities,
        sort,
        filters: legacy_filters,
    } = payload;

    let root_scope_key = String::new();
    let schema = metadata_service
        .latest_published_schema_unchecked(actor, entity_logical_name)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "entity '{}' must be published before runtime records can be queried",
                entity_logical_name
            ))
        })?;

    let mut scope_field_types = std::collections::BTreeMap::new();
    scope_field_types.insert(
        root_scope_key.clone(),
        schema
            .fields()
            .iter()
            .map(|field| (field.logical_name().as_str().to_owned(), field.field_type()))
            .collect::<std::collections::BTreeMap<_, _>>(),
    );

    let links = runtime_record_links_from_request(
        metadata_service,
        actor,
        entity_logical_name,
        &schema,
        link_entities,
        &mut scope_field_types,
    )
    .await?;

    let mut filters = conditions
        .unwrap_or_default()
        .into_iter()
        .map(|condition| {
            runtime_record_filter_from_request(
                condition,
                entity_logical_name,
                &scope_field_types,
                true,
            )
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    filters.extend(legacy_filters.unwrap_or_default().into_iter().map(
        |(field_logical_name, field_value)| {
            qryvanta_application::RuntimeRecordFilter {
                scope_alias: None,
                field_type: scope_field_types
                    .get(&root_scope_key)
                    .and_then(|field_types| field_types.get(field_logical_name.as_str()))
                    .copied()
                    .unwrap_or(qryvanta_domain::FieldType::Json),
                field_logical_name,
                operator: qryvanta_application::RuntimeRecordOperator::Eq,
                field_value,
            }
        },
    ));

    let sort = sort
        .unwrap_or_default()
        .into_iter()
        .map(|entry| {
            let scope_alias = normalize_scope_alias(entry.scope_alias, "sort")?;
            let scope_key = scope_alias.clone().unwrap_or_default();
            let direction = entry
                .direction
                .as_deref()
                .map(qryvanta_application::RuntimeRecordSortDirection::parse_transport)
                .transpose()?
                .unwrap_or(qryvanta_application::RuntimeRecordSortDirection::Asc);

            let field_type = scope_field_types
                .get(&scope_key)
                .and_then(|field_types| field_types.get(entry.field_logical_name.as_str()))
                .copied()
                .ok_or_else(|| match scope_alias.as_deref() {
                    Some(alias) => AppError::Validation(format!(
                        "unknown sort field '{}' for alias '{}'",
                        entry.field_logical_name, alias
                    )),
                    None => AppError::Validation(format!(
                        "unknown sort field '{}' for entity '{}'",
                        entry.field_logical_name, entity_logical_name
                    )),
                })?;

            Ok(qryvanta_application::RuntimeRecordSort {
                scope_alias,
                field_logical_name: entry.field_logical_name,
                field_type,
                direction,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let where_clause = where_clause
        .map(|group| {
            runtime_record_group_from_request(group, entity_logical_name, &scope_field_types)
        })
        .transpose()?;

    let logical_mode = logical_mode
        .as_deref()
        .map(qryvanta_application::RuntimeRecordLogicalMode::parse_transport)
        .transpose()?
        .unwrap_or(qryvanta_application::RuntimeRecordLogicalMode::And);

    Ok(qryvanta_application::RuntimeRecordQuery {
        limit: limit.unwrap_or(50),
        offset: offset.unwrap_or(0),
        logical_mode,
        where_clause,
        filters,
        links,
        sort,
        owner_subject: None,
    })
}

fn normalize_scope_alias(
    scope_alias: Option<String>,
    context: &str,
) -> Result<Option<String>, AppError> {
    match scope_alias {
        Some(alias) => {
            let trimmed = alias.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation(format!(
                    "runtime query {context} scope_alias cannot be empty"
                )));
            }

            Ok(Some(trimmed.to_owned()))
        }
        None => Ok(None),
    }
}

fn runtime_record_filter_from_request(
    condition: crate::dto::RuntimeRecordQueryFilterRequest,
    root_entity_logical_name: &str,
    scope_field_types: &std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, qryvanta_domain::FieldType>,
    >,
    validate_field_exists: bool,
) -> Result<qryvanta_application::RuntimeRecordFilter, AppError> {
    let scope_alias = normalize_scope_alias(condition.scope_alias, "condition")?;
    let scope_key = scope_alias.clone().unwrap_or_default();
    let operator =
        qryvanta_application::RuntimeRecordOperator::parse_transport(condition.operator.as_str())?;

    let field_type = scope_field_types
        .get(&scope_key)
        .and_then(|field_types| field_types.get(condition.field_logical_name.as_str()))
        .copied();

    let field_type = match (validate_field_exists, field_type, scope_alias.as_deref()) {
        (_, Some(field_type), _) => field_type,
        (true, None, Some(alias)) => {
            return Err(AppError::Validation(format!(
                "unknown filter field '{}' for alias '{}'",
                condition.field_logical_name, alias
            )));
        }
        (true, None, None) => {
            return Err(AppError::Validation(format!(
                "unknown filter field '{}' for entity '{}'",
                condition.field_logical_name, root_entity_logical_name
            )));
        }
        (false, None, _) => qryvanta_domain::FieldType::Json,
    };

    Ok(qryvanta_application::RuntimeRecordFilter {
        scope_alias,
        field_logical_name: condition.field_logical_name,
        operator,
        field_type,
        field_value: condition.field_value,
    })
}

fn runtime_record_group_from_request(
    group: crate::dto::RuntimeRecordQueryGroupRequest,
    root_entity_logical_name: &str,
    scope_field_types: &std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, qryvanta_domain::FieldType>,
    >,
) -> Result<qryvanta_application::RuntimeRecordConditionGroup, AppError> {
    let logical_mode = group
        .logical_mode
        .as_deref()
        .map(qryvanta_application::RuntimeRecordLogicalMode::parse_transport)
        .transpose()?
        .unwrap_or(qryvanta_application::RuntimeRecordLogicalMode::And);

    let mut nodes = Vec::new();
    for condition in group.conditions.unwrap_or_default() {
        nodes.push(qryvanta_application::RuntimeRecordConditionNode::Filter(
            runtime_record_filter_from_request(
                condition,
                root_entity_logical_name,
                scope_field_types,
                true,
            )?,
        ));
    }

    for nested_group in group.groups.unwrap_or_default() {
        nodes.push(qryvanta_application::RuntimeRecordConditionNode::Group(
            runtime_record_group_from_request(
                nested_group,
                root_entity_logical_name,
                scope_field_types,
            )?,
        ));
    }

    if nodes.is_empty() {
        return Err(AppError::Validation(
            "runtime query where clause must include at least one condition or nested group"
                .to_owned(),
        ));
    }

    Ok(qryvanta_application::RuntimeRecordConditionGroup {
        logical_mode,
        nodes,
    })
}

async fn runtime_record_links_from_request(
    metadata_service: &qryvanta_application::MetadataService,
    actor: &UserIdentity,
    root_entity_logical_name: &str,
    root_schema: &qryvanta_domain::PublishedEntitySchema,
    link_entities: Option<Vec<crate::dto::RuntimeRecordQueryLinkEntityRequest>>,
    scope_field_types: &mut std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, qryvanta_domain::FieldType>,
    >,
) -> Result<Vec<qryvanta_application::RuntimeRecordLink>, AppError> {
    let mut links = Vec::new();
    let mut scope_entities = std::collections::BTreeMap::new();
    scope_entities.insert(String::new(), root_entity_logical_name.to_owned());
    let mut schema_cache = std::collections::BTreeMap::new();
    schema_cache.insert(root_entity_logical_name.to_owned(), root_schema.clone());

    for entry in link_entities.unwrap_or_default() {
        let alias = entry.alias.trim().to_owned();
        if alias.is_empty() {
            return Err(AppError::Validation(
                "runtime query link alias cannot be empty".to_owned(),
            ));
        }

        if alias == "root" {
            return Err(AppError::Validation(
                "runtime query link alias 'root' is reserved".to_owned(),
            ));
        }

        if scope_entities.contains_key(alias.as_str()) {
            return Err(AppError::Validation(format!(
                "duplicate runtime query link alias '{}'",
                alias
            )));
        }

        let parent_alias = normalize_scope_alias(entry.parent_alias, "link")?;
        let parent_scope_key = parent_alias.clone().unwrap_or_default();
        let Some(parent_entity_logical_name) = scope_entities.get(parent_scope_key.as_str()) else {
            return Err(AppError::Validation(format!(
                "unknown runtime query parent alias '{}'",
                parent_scope_key
            )));
        };

        let parent_schema = load_published_schema_for_entity(
            metadata_service,
            actor,
            parent_entity_logical_name.as_str(),
            &mut schema_cache,
        )
        .await?;

        let relation_field_name = entry.relation_field_logical_name.trim().to_owned();
        if relation_field_name.is_empty() {
            return Err(AppError::Validation(
                "runtime query link relation_field_logical_name cannot be empty".to_owned(),
            ));
        }

        let Some(relation_field) = parent_schema
            .fields()
            .iter()
            .find(|field| field.logical_name().as_str() == relation_field_name)
        else {
            return Err(AppError::Validation(format!(
                "unknown relation field '{}' for parent entity '{}'",
                relation_field_name,
                parent_schema.entity().logical_name().as_str()
            )));
        };

        if relation_field.field_type() != qryvanta_domain::FieldType::Relation {
            return Err(AppError::Validation(format!(
                "link relation field '{}' on entity '{}' must be of type 'relation'",
                relation_field_name,
                parent_schema.entity().logical_name().as_str()
            )));
        }

        let Some(target_entity) = relation_field.relation_target_entity() else {
            return Err(AppError::Validation(format!(
                "relation field '{}' on entity '{}' is missing relation target metadata",
                relation_field_name,
                parent_schema.entity().logical_name().as_str()
            )));
        };

        let join_type = entry
            .join_type
            .as_deref()
            .map(qryvanta_application::RuntimeRecordJoinType::parse_transport)
            .transpose()?
            .unwrap_or(qryvanta_application::RuntimeRecordJoinType::Inner);

        let target_schema = load_published_schema_for_entity(
            metadata_service,
            actor,
            target_entity.as_str(),
            &mut schema_cache,
        )
        .await?;

        scope_field_types.insert(
            alias.clone(),
            target_schema
                .fields()
                .iter()
                .map(|field| (field.logical_name().as_str().to_owned(), field.field_type()))
                .collect(),
        );

        links.push(qryvanta_application::RuntimeRecordLink {
            alias: alias.clone(),
            parent_alias,
            relation_field_logical_name: relation_field_name,
            target_entity_logical_name: target_entity.as_str().to_owned(),
            join_type,
        });
        scope_entities.insert(alias, target_entity.as_str().to_owned());
    }

    Ok(links)
}

async fn load_published_schema_for_entity(
    metadata_service: &qryvanta_application::MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    schema_cache: &mut std::collections::BTreeMap<String, qryvanta_domain::PublishedEntitySchema>,
) -> Result<qryvanta_domain::PublishedEntitySchema, AppError> {
    if let Some(schema) = schema_cache.get(entity_logical_name) {
        return Ok(schema.clone());
    }

    let schema = metadata_service
        .latest_published_schema_unchecked(actor, entity_logical_name)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "entity '{}' must be published before runtime query links can be used",
                entity_logical_name
            ))
        })?;

    schema_cache.insert(entity_logical_name.to_owned(), schema.clone());
    Ok(schema)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::dto::runtime::RuntimeRecordQuerySortRequest;
    use crate::dto::{
        QueryRuntimeRecordsRequest, RuntimeRecordQueryFilterRequest,
        RuntimeRecordQueryGroupRequest, RuntimeRecordQueryLinkEntityRequest,
    };
    use crate::error::ApiError;
    use async_trait::async_trait;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use qryvanta_application::{
        AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService,
        MetadataService, RuntimeFieldGrant, SaveFieldInput, TemporaryPermissionGrant,
    };
    use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
    use qryvanta_domain::{FieldType, Permission};
    use qryvanta_infrastructure::InMemoryMetadataRepository;

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
                        relation_target_entity: None,
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
                        relation_target_entity: None,
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
                        relation_target_entity: Some("contact".to_owned()),
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

        let response =
            ApiError::from(result.err().unwrap_or_else(|| unreachable!())).into_response();
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
}
