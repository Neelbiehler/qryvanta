mod filters;
mod ingest;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use qryvanta_application::RecordListQuery;
use qryvanta_core::{AppError, UserIdentity};

use self::filters::{QrywellSearchFilters, plan_filters_for_query};
use self::ingest::push_records_to_qrywell;

use crate::dto::{
    GenericMessageResponse, QrywellSearchAnalyticsResponse, QrywellSearchClickEventRequest,
    QrywellSearchHitResponse, QrywellSearchLowRelevanceClickResponse,
    QrywellSearchRankMetricResponse, QrywellSearchRequest, QrywellSearchResponse,
    QrywellSearchTopQueryResponse, QrywellSearchZeroClickQueryResponse, QrywellSyncAllResponse,
    QrywellSyncFailedJobResponse, QrywellSyncHealthResponse, QrywellSyncRequest,
    QrywellSyncResponse, RuntimeRecordResponse,
};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn qrywell_search_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<QrywellSearchRequest>,
) -> ApiResult<Json<QrywellSearchResponse>> {
    if payload.query.trim().is_empty() {
        return Err(AppError::Validation("query must not be empty".to_owned()).into());
    }

    let base_url = state
        .qrywell_api_base_url
        .clone()
        .ok_or_else(|| AppError::Validation("QRYWELL_API_BASE_URL is not configured".to_owned()))?;
    let include_debug = payload.include_debug.unwrap_or(false);
    let roles = payload.roles.unwrap_or_default();
    let search_plan = plan_filters_for_query(&state, &user, payload.query.as_str()).await?;
    debug!(
        query = %payload.query,
        selected_entity = ?search_plan.selected_entity,
        planned_filter_count = search_plan.planned_filter_count,
        negated_filter_count = search_plan.negated_filter_count,
        "qrywell search plan computed"
    );
    let request_body = QrywellSearchProxyRequest {
        query: payload.query.clone(),
        limit: payload.limit,
        viewer: QrywellViewerContext {
            user_id: user.subject().to_owned(),
            tenant_id: user.tenant_id().to_string(),
            roles,
        },
        filters: search_plan.filters.clone(),
    };

    let endpoint = format!("{}/v0/search", base_url.trim_end_matches('/'));
    let mut request = state.http_client.post(endpoint).json(&request_body);
    if let Some(api_key) = &state.qrywell_api_key {
        request = request.header("x-qrywell-api-key", api_key);
    }

    let response = request.send().await.map_err(|error| {
        AppError::Internal(format!("failed calling qrywell search endpoint: {error}"))
    })?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_else(|_| String::new());
        return Err(
            AppError::Internal(format!("qrywell search request failed: {}", body.trim())).into(),
        );
    }

    let search_response = response
        .json::<QrywellSearchBackendResponse>()
        .await
        .map_err(|error| AppError::Internal(format!("invalid qrywell search response: {error}")))?;

    let search_event_id =
        match sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
        INSERT INTO qrywell_search_query_events (
            tenant_id,
            user_subject,
            query,
            normalized_query,
            total_hits,
            selected_entity,
            planned_filter_count,
            negated_filter_count,
            clicked_count,
            created_at
        )
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, 0, now())
        RETURNING id
        "#,
        )
        .bind(user.tenant_id().to_string())
        .bind(user.subject())
        .bind(payload.query.trim())
        .bind(search_plan.normalized_query.clone())
        .bind(i32::try_from(search_response.total_hits).map_err(|error| {
            AppError::Internal(format!("invalid total_hits conversion: {error}"))
        })?)
        .bind(search_plan.selected_entity.clone())
        .bind(
            i32::try_from(search_plan.planned_filter_count).map_err(|error| {
                AppError::Internal(format!("invalid planned filter conversion: {error}"))
            })?,
        )
        .bind(
            i32::try_from(search_plan.negated_filter_count).map_err(|error| {
                AppError::Internal(format!("invalid negated filter conversion: {error}"))
            })?,
        )
        .fetch_one(&state.postgres_pool)
        .await
        {
            Ok(event_id) => Some(event_id.to_string()),
            Err(error) => {
                warn!(error = %error, "failed to persist qrywell search query event");
                None
            }
        };

    Ok(Json(QrywellSearchResponse {
        search_event_id,
        query: search_response.query,
        total_hits: search_response.total_hits,
        hits: search_response
            .hits
            .into_iter()
            .map(|hit| QrywellSearchHitResponse {
                id: hit.id,
                document_id: hit.document_id,
                connector_type: hit.connector_type,
                title: hit.title,
                url: hit.url,
                text: hit.text,
                score: hit.score,
            })
            .collect(),
        debug_query_normalized: include_debug.then_some(search_plan.normalized_query),
        debug_selected_entity: include_debug
            .then_some(search_plan.selected_entity)
            .flatten(),
        debug_planned_filter_count: include_debug.then_some(search_plan.planned_filter_count),
        debug_negated_filter_count: include_debug.then_some(search_plan.negated_filter_count),
    }))
}

pub async fn qrywell_search_click_event_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<QrywellSearchClickEventRequest>,
) -> ApiResult<Json<GenericMessageResponse>> {
    if payload.query.trim().is_empty() || payload.result_id.trim().is_empty() {
        return Err(AppError::Validation("query and result_id are required".to_owned()).into());
    }

    sqlx::query(
        r#"
        INSERT INTO qrywell_search_click_events (
            tenant_id,
            user_subject,
            query,
            result_id,
            title,
            connector_type,
            rank,
            score,
            group_label,
            clicked_at
        )
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9, now())
        "#,
    )
    .bind(user.tenant_id().to_string())
    .bind(user.subject())
    .bind(payload.query.trim())
    .bind(payload.result_id.trim())
    .bind(payload.title.trim())
    .bind(payload.connector_type.trim())
    .bind(
        i32::try_from(payload.rank)
            .map_err(|error| AppError::Validation(format!("invalid rank conversion: {error}")))?,
    )
    .bind(f64::from(payload.score))
    .bind(payload.group_label)
    .execute(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to store search click event: {error}")))?;

    if let Some(search_event_id) = payload.search_event_id.as_deref() {
        if let Ok(event_uuid) = uuid::Uuid::parse_str(search_event_id) {
            let _ = sqlx::query(
                r#"
                UPDATE qrywell_search_query_events
                SET clicked_count = clicked_count + 1
                WHERE id = $1
                  AND tenant_id = $2::uuid
                "#,
            )
            .bind(event_uuid)
            .bind(user.tenant_id().to_string())
            .execute(&state.postgres_pool)
            .await;
        }
    }

    Ok(Json(GenericMessageResponse {
        message: "search click recorded".to_owned(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct QrywellSearchAnalyticsQuery {
    pub window_days: Option<i32>,
    pub limit: Option<usize>,
}

pub async fn qrywell_search_analytics_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<QrywellSearchAnalyticsQuery>,
) -> ApiResult<Json<QrywellSearchAnalyticsResponse>> {
    let window_days = query.window_days.unwrap_or(14).clamp(1, 90);
    let limit = query.limit.unwrap_or(10).clamp(3, 50);

    let total_queries = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::bigint
        FROM qrywell_search_query_events
        WHERE tenant_id = $1::uuid
          AND created_at >= now() - make_interval(days => $2)
        "#,
    )
    .bind(user.tenant_id().to_string())
    .bind(window_days)
    .fetch_one(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to load total query count: {error}")))?;

    let total_clicks = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::bigint
        FROM qrywell_search_click_events
        WHERE tenant_id = $1::uuid
          AND clicked_at >= now() - make_interval(days => $2)
        "#,
    )
    .bind(user.tenant_id().to_string())
    .bind(window_days)
    .fetch_one(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to load total click count: {error}")))?;

    let top_queries = sqlx::query_as::<_, TopQueryRow>(
        r#"
        SELECT
            normalized_query AS query,
            COUNT(*)::bigint AS runs,
            COALESCE(SUM(clicked_count), 0)::bigint AS clicks
        FROM qrywell_search_query_events
        WHERE tenant_id = $1::uuid
          AND created_at >= now() - make_interval(days => $2)
        GROUP BY normalized_query
        ORDER BY runs DESC, clicks DESC
        LIMIT $3
        "#,
    )
    .bind(user.tenant_id().to_string())
    .bind(window_days)
    .bind(i64::try_from(limit).map_err(|error| {
        AppError::Internal(format!("invalid analytics limit conversion: {error}"))
    })?)
    .fetch_all(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to load top queries: {error}")))?
    .into_iter()
    .map(|row| QrywellSearchTopQueryResponse {
        query: row.query,
        runs: row.runs,
        clicks: row.clicks,
    })
    .collect::<Vec<_>>();

    let rank_rows = sqlx::query_as::<_, RankMetricRow>(
        r#"
        WITH click_ranks AS (
            SELECT rank, COUNT(*)::bigint AS clicks
            FROM qrywell_search_click_events
            WHERE tenant_id = $1::uuid
              AND clicked_at >= now() - make_interval(days => $2)
            GROUP BY rank
        )
        SELECT
            rank,
            clicks,
            CASE
                WHEN SUM(clicks) OVER () = 0 THEN 0
                ELSE clicks::double precision / SUM(clicks) OVER ()
            END AS click_share
        FROM click_ranks
        ORDER BY rank ASC
        LIMIT 10
        "#,
    )
    .bind(user.tenant_id().to_string())
    .bind(window_days)
    .fetch_all(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to load rank metrics: {error}")))?
    .into_iter()
    .map(|row| QrywellSearchRankMetricResponse {
        rank: row.rank,
        clicks: row.clicks,
        click_share: row.click_share as f32,
    })
    .collect::<Vec<_>>();

    let zero_click_queries = sqlx::query_as::<_, ZeroClickRow>(
        r#"
        SELECT
            normalized_query AS query,
            COUNT(*)::bigint AS runs
        FROM qrywell_search_query_events
        WHERE tenant_id = $1::uuid
          AND created_at >= now() - make_interval(days => $2)
          AND total_hits > 0
          AND clicked_count = 0
        GROUP BY normalized_query
        ORDER BY runs DESC
        LIMIT $3
        "#,
    )
    .bind(user.tenant_id().to_string())
    .bind(window_days)
    .bind(i64::try_from(limit).map_err(|error| {
        AppError::Internal(format!("invalid analytics limit conversion: {error}"))
    })?)
    .fetch_all(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to load zero-click queries: {error}")))?
    .into_iter()
    .map(|row| QrywellSearchZeroClickQueryResponse {
        query: row.query,
        runs: row.runs,
    })
    .collect::<Vec<_>>();

    let low_relevance_clicks = sqlx::query_as::<_, LowRelevanceRow>(
        r#"
        SELECT
            query,
            title,
            AVG(score)::double precision AS avg_score,
            COUNT(*)::bigint AS clicks
        FROM qrywell_search_click_events
        WHERE tenant_id = $1::uuid
          AND clicked_at >= now() - make_interval(days => $2)
        GROUP BY query, title
        HAVING AVG(score) < 0.45
        ORDER BY avg_score ASC, clicks DESC
        LIMIT $3
        "#,
    )
    .bind(user.tenant_id().to_string())
    .bind(window_days)
    .bind(i64::try_from(limit).map_err(|error| {
        AppError::Internal(format!("invalid analytics limit conversion: {error}"))
    })?)
    .fetch_all(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to load low relevance clicks: {error}")))?
    .into_iter()
    .map(|row| QrywellSearchLowRelevanceClickResponse {
        query: row.query,
        title: row.title,
        avg_score: row.avg_score as f32,
        clicks: row.clicks,
    })
    .collect::<Vec<_>>();

    Ok(Json(QrywellSearchAnalyticsResponse {
        window_days,
        total_queries,
        total_clicks,
        top_queries,
        rank_metrics: rank_rows,
        zero_click_queries,
        low_relevance_clicks,
    }))
}

pub async fn qrywell_sync_entity_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Path(entity_logical_name): Path<String>,
    Json(payload): Json<QrywellSyncRequest>,
) -> ApiResult<Json<QrywellSyncResponse>> {
    let records = state
        .metadata_service
        .list_runtime_records(
            &user,
            entity_logical_name.as_str(),
            RecordListQuery {
                limit: payload.limit.unwrap_or(200),
                offset: payload.offset.unwrap_or(0),
                owner_subject: None,
            },
        )
        .await?;

    if records.is_empty() {
        return Ok(Json(QrywellSyncResponse {
            entity_logical_name,
            synced_records: 0,
            indexed_chunks: 0,
        }));
    }

    let response = push_records_to_qrywell(
        &state,
        &user,
        entity_logical_name.as_str(),
        &records
            .into_iter()
            .map(RuntimeRecordResponse::from)
            .collect::<Vec<_>>(),
    )
    .await?;

    Ok(Json(QrywellSyncResponse {
        entity_logical_name,
        synced_records: response.indexed_records,
        indexed_chunks: response.indexed_chunks,
    }))
}

pub async fn qrywell_sync_all_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Json(payload): Json<QrywellSyncRequest>,
) -> ApiResult<Json<QrywellSyncAllResponse>> {
    let entities = state.metadata_service.list_entities(&user).await?;
    let mut summaries = Vec::new();
    let mut total_records = 0_usize;
    let mut total_chunks = 0_usize;

    for entity in entities {
        let entity_name = entity.logical_name().as_str().to_owned();
        let records = state
            .metadata_service
            .list_runtime_records(
                &user,
                entity_name.as_str(),
                RecordListQuery {
                    limit: payload.limit.unwrap_or(200),
                    offset: payload.offset.unwrap_or(0),
                    owner_subject: None,
                },
            )
            .await?;

        if records.is_empty() {
            summaries.push(QrywellSyncResponse {
                entity_logical_name: entity_name,
                synced_records: 0,
                indexed_chunks: 0,
            });
            continue;
        }

        let response = push_records_to_qrywell(
            &state,
            &user,
            entity_name.as_str(),
            &records
                .into_iter()
                .map(RuntimeRecordResponse::from)
                .collect::<Vec<_>>(),
        )
        .await?;

        total_records += response.indexed_records;
        total_chunks += response.indexed_chunks;
        summaries.push(QrywellSyncResponse {
            entity_logical_name: entity_name,
            synced_records: response.indexed_records,
            indexed_chunks: response.indexed_chunks,
        });
    }

    let total_entities = summaries.len();
    Ok(Json(QrywellSyncAllResponse {
        entities: summaries,
        total_entities,
        total_records,
        total_chunks,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct QrywellSyncHealthQuery {
    pub failed_limit: Option<usize>,
}

pub async fn qrywell_sync_health_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserIdentity>,
    Query(query): Query<QrywellSyncHealthQuery>,
) -> ApiResult<Json<QrywellSyncHealthResponse>> {
    let failed_limit = query.failed_limit.unwrap_or(10).clamp(1, 50);

    let stats = sqlx::query_as::<_, SyncQueueStatsRow>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'pending')::bigint AS pending_jobs,
            COUNT(*) FILTER (WHERE status = 'processing')::bigint AS processing_jobs,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint AS failed_jobs
        FROM qrywell_sync_jobs
        WHERE tenant_id = $1::uuid
        "#,
    )
    .bind(user.tenant_id().to_string())
    .fetch_one(&state.postgres_pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to load qrywell sync stats: {error}")))?;

    let failed_recent =
        sqlx::query_as::<_, SyncFailedJobRow>(
            r#"
        SELECT
            id,
            entity_logical_name,
            record_id,
            operation,
            attempt_count,
            max_attempts,
            next_attempt_at,
            updated_at,
            last_error
        FROM qrywell_sync_jobs
        WHERE tenant_id = $1::uuid
          AND status = 'failed'
        ORDER BY updated_at DESC
        LIMIT $2
        "#,
        )
        .bind(user.tenant_id().to_string())
        .bind(i64::try_from(failed_limit).map_err(|error| {
            AppError::Internal(format!("invalid failed limit conversion: {error}"))
        })?)
        .fetch_all(&state.postgres_pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to load qrywell failed sync jobs: {error}"))
        })?
        .into_iter()
        .map(|row| QrywellSyncFailedJobResponse {
            job_id: row.id.to_string(),
            entity_logical_name: row.entity_logical_name,
            record_id: row.record_id,
            operation: row.operation,
            attempt_count: row.attempt_count,
            max_attempts: row.max_attempts,
            next_attempt_at: row.next_attempt_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            last_error: row.last_error,
        })
        .collect::<Vec<_>>();

    let activity = sqlx::query_as::<_, SyncQueueActivityRow>(
        r#"
        SELECT
            total_succeeded,
            total_failed,
            last_attempt_at,
            last_success_at,
            last_failure_at
        FROM qrywell_sync_stats
        WHERE tenant_id = $1::uuid
        "#,
    )
    .bind(user.tenant_id().to_string())
    .fetch_optional(&state.postgres_pool)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to load qrywell sync activity stats: {error}"
        ))
    })?;

    Ok(Json(QrywellSyncHealthResponse {
        pending_jobs: stats.pending_jobs,
        processing_jobs: stats.processing_jobs,
        failed_jobs: stats.failed_jobs,
        total_succeeded: activity.as_ref().map_or(0, |row| row.total_succeeded),
        total_failed: activity.as_ref().map_or(0, |row| row.total_failed),
        last_attempt_at: activity
            .as_ref()
            .and_then(|row| row.last_attempt_at.map(|value| value.to_rfc3339())),
        last_success_at: activity
            .as_ref()
            .and_then(|row| row.last_success_at.map(|value| value.to_rfc3339())),
        last_failure_at: activity
            .as_ref()
            .and_then(|row| row.last_failure_at.map(|value| value.to_rfc3339())),
        failed_recent,
    }))
}

#[derive(Debug, Serialize)]
struct QrywellSearchProxyRequest {
    query: String,
    limit: Option<usize>,
    viewer: QrywellViewerContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<QrywellSearchFilters>,
}

#[derive(Debug, Deserialize)]
struct QrywellSearchBackendResponse {
    query: String,
    total_hits: usize,
    hits: Vec<QrywellSearchHitProxy>,
}

#[derive(Debug, Deserialize)]
struct QrywellSearchHitProxy {
    id: String,
    document_id: String,
    connector_type: String,
    title: String,
    url: String,
    text: String,
    score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct QrywellViewerContext {
    user_id: String,
    tenant_id: String,
    roles: Vec<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct SyncQueueStatsRow {
    pending_jobs: i64,
    processing_jobs: i64,
    failed_jobs: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct SyncFailedJobRow {
    id: uuid::Uuid,
    entity_logical_name: String,
    record_id: String,
    operation: String,
    attempt_count: i32,
    max_attempts: i32,
    next_attempt_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    last_error: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct SyncQueueActivityRow {
    total_succeeded: i64,
    total_failed: i64,
    last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    last_success_at: Option<chrono::DateTime<chrono::Utc>>,
    last_failure_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct TopQueryRow {
    query: String,
    runs: i64,
    clicks: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct RankMetricRow {
    rank: i32,
    clicks: i64,
    click_share: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct ZeroClickRow {
    query: String,
    runs: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct LowRelevanceRow {
    query: String,
    title: String,
    avg_score: f64,
    clicks: i64,
}
