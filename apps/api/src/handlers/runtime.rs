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
    let schema = state
        .metadata_service
        .latest_published_schema_unchecked(&user, entity_logical_name.as_str())
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

    let records = state
        .metadata_service
        .query_runtime_records(
            &user,
            entity_logical_name.as_str(),
            qryvanta_application::RuntimeRecordQuery {
                limit: payload.limit.unwrap_or(50),
                offset: payload.offset.unwrap_or(0),
                logical_mode,
                filters,
                sort,
                owner_subject: None,
            },
        )
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
