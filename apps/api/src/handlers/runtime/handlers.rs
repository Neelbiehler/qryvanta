use super::*;

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
            record.data(),
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

    let response = RuntimeRecordResponse::from(record);
    if let Err(error) = crate::qrywell_sync::enqueue_runtime_record_upsert(
        &state.postgres_pool,
        user.tenant_id(),
        entity_logical_name.as_str(),
        &response,
        state.qrywell_sync_max_attempts,
    )
    .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            entity_logical_name = %entity_logical_name,
            record_id = %response.record_id,
            "qrywell sync failed after runtime record creation"
        );
    }

    Ok((StatusCode::CREATED, Json(response)))
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
    let previous_record = state
        .metadata_service
        .get_runtime_record(&user, entity_logical_name.as_str(), record_id.as_str())
        .await
        .ok();

    let record = state
        .metadata_service
        .update_runtime_record(
            &user,
            entity_logical_name.as_str(),
            record_id.as_str(),
            payload.data,
        )
        .await?;

    if let Err(error) = state
        .workflow_service
        .dispatch_runtime_record_updated(
            &user,
            entity_logical_name.as_str(),
            record.record_id().as_str(),
            previous_record
                .as_ref()
                .map(|runtime_record| runtime_record.data()),
            record.data(),
        )
        .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            entity_logical_name = %entity_logical_name,
            record_id = %record.record_id().as_str(),
            "workflow dispatch failed after runtime record update"
        );
    }

    let response = RuntimeRecordResponse::from(record);
    if let Err(error) = crate::qrywell_sync::enqueue_runtime_record_upsert(
        &state.postgres_pool,
        user.tenant_id(),
        entity_logical_name.as_str(),
        &response,
        state.qrywell_sync_max_attempts,
    )
    .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            entity_logical_name = %entity_logical_name,
            record_id = %response.record_id,
            "qrywell sync failed after runtime record update"
        );
    }

    Ok(Json(response))
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
    let deleted_record = state
        .metadata_service
        .get_runtime_record(&user, entity_logical_name.as_str(), record_id.as_str())
        .await
        .ok();

    state
        .metadata_service
        .delete_runtime_record(&user, entity_logical_name.as_str(), record_id.as_str())
        .await?;

    if let Err(error) = state
        .workflow_service
        .dispatch_runtime_record_deleted(
            &user,
            entity_logical_name.as_str(),
            record_id.as_str(),
            deleted_record
                .as_ref()
                .map(|runtime_record| runtime_record.data()),
        )
        .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            entity_logical_name = %entity_logical_name,
            record_id = %record_id,
            "workflow dispatch failed after runtime record deletion"
        );
    }

    if let Err(error) = crate::qrywell_sync::enqueue_runtime_record_delete(
        &state.postgres_pool,
        user.tenant_id(),
        entity_logical_name.as_str(),
        record_id.as_str(),
        state.qrywell_sync_max_attempts,
    )
    .await
    {
        warn!(
            error = %error,
            tenant_id = %user.tenant_id(),
            entity_logical_name = %entity_logical_name,
            record_id = %record_id,
            "qrywell delete sync failed after runtime record deletion"
        );
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_runtime_business_rules_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
) -> ApiResult<Json<Vec<BusinessRuleResponse>>> {
    let rules = state
        .metadata_service
        .list_business_rules(&user, entity_logical_name.as_str())
        .await?
        .into_iter()
        .filter(|rule| rule.is_active())
        .map(BusinessRuleResponse::from)
        .collect();

    Ok(Json(rules))
}
