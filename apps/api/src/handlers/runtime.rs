use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use qryvanta_core::AppError;
use qryvanta_core::UserIdentity;

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
    let schema = metadata_service
        .latest_published_schema_unchecked(actor, entity_logical_name)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "entity '{}' must be published before runtime records can be queried",
                entity_logical_name
            ))
        })?;

    let field_types = schema
        .fields()
        .iter()
        .map(|field| (field.logical_name().as_str().to_owned(), field.field_type()))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut filters = payload
        .conditions
        .unwrap_or_default()
        .into_iter()
        .map(|condition| {
            let operator = qryvanta_application::RuntimeRecordOperator::parse_transport(
                condition.operator.as_str(),
            )?;
            let field_type = field_types
                .get(condition.field_logical_name.as_str())
                .copied()
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "unknown filter field '{}' for entity '{}'",
                        condition.field_logical_name, entity_logical_name
                    ))
                })?;

            Ok(qryvanta_application::RuntimeRecordFilter {
                field_logical_name: condition.field_logical_name,
                operator,
                field_type,
                field_value: condition.field_value,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    filters.extend(payload.filters.unwrap_or_default().into_iter().map(
        |(field_logical_name, field_value)| {
            qryvanta_application::RuntimeRecordFilter {
                field_type: field_types
                    .get(field_logical_name.as_str())
                    .copied()
                    .unwrap_or(qryvanta_domain::FieldType::Json),
                field_logical_name,
                operator: qryvanta_application::RuntimeRecordOperator::Eq,
                field_value,
            }
        },
    ));

    let sort = payload
        .sort
        .unwrap_or_default()
        .into_iter()
        .map(|entry| {
            let direction = entry
                .direction
                .as_deref()
                .map(qryvanta_application::RuntimeRecordSortDirection::parse_transport)
                .transpose()?
                .unwrap_or(qryvanta_application::RuntimeRecordSortDirection::Asc);

            let field_type = field_types
                .get(entry.field_logical_name.as_str())
                .copied()
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "unknown sort field '{}' for entity '{}'",
                        entry.field_logical_name, entity_logical_name
                    ))
                })?;

            Ok(qryvanta_application::RuntimeRecordSort {
                field_logical_name: entry.field_logical_name,
                field_type,
                direction,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let logical_mode = payload
        .logical_mode
        .as_deref()
        .map(qryvanta_application::RuntimeRecordLogicalMode::parse_transport)
        .transpose()?
        .unwrap_or(qryvanta_application::RuntimeRecordLogicalMode::And);

    Ok(qryvanta_application::RuntimeRecordQuery {
        limit: payload.limit.unwrap_or(50),
        offset: payload.offset.unwrap_or(0),
        logical_mode,
        filters,
        sort,
        owner_subject: None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::dto::QueryRuntimeRecordsRequest;
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
                .publish_entity(&actor, "contact")
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
                conditions: None,
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
                conditions: None,
                sort: None,
                filters: None,
            },
        )
        .await;

        let response =
            ApiError::from(result.err().unwrap_or_else(|| unreachable!())).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
