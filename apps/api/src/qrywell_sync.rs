//! Durable sync queue for Qrywell indexing.

use std::time::Duration;

use qryvanta_core::{AppError, TenantId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::dto::RuntimeRecordResponse;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SyncOperation {
    Upsert,
    Delete,
}

impl SyncOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Upsert => "upsert",
            Self::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncPayload {
    record_id: String,
    entity_logical_name: String,
    title: Option<String>,
    content: Option<String>,
    url: Option<String>,
    tenant_id: String,
    roles: Vec<String>,
}

#[derive(Debug)]
struct SyncJob {
    id: uuid::Uuid,
    operation: SyncOperation,
    attempt_count: i32,
    max_attempts: i32,
    payload: SyncPayload,
}

#[derive(Debug, Serialize)]
struct QrywellIngestRequest {
    records: Vec<QrywellIngestRecord>,
}

#[derive(Debug, Serialize)]
struct QrywellIngestRecord {
    record_id: String,
    entity_logical_name: String,
    title: String,
    content: String,
    url: Option<String>,
    tenant_id: String,
    roles: Vec<String>,
}

#[derive(Debug, Serialize)]
struct QrywellDeleteRequest {
    record_id: String,
    entity_logical_name: String,
}

#[derive(sqlx::FromRow)]
struct ClaimedJobRow {
    id: uuid::Uuid,
    operation: String,
    payload: Value,
    attempt_count: i32,
    max_attempts: i32,
}

pub fn spawn_qrywell_sync_worker(state: AppState) {
    if state.qrywell_api_base_url.is_none() {
        info!("qrywell sync worker disabled (QRYWELL_API_BASE_URL not configured)");
        return;
    }

    tokio::spawn(async move {
        info!(
            interval_ms = state.qrywell_sync_poll_interval_ms,
            batch_size = state.qrywell_sync_batch_size,
            "qrywell sync worker started"
        );

        loop {
            if let Err(error) = process_sync_batch(&state).await {
                error!(error = %error, "qrywell sync worker batch failed");
            }

            tokio::time::sleep(Duration::from_millis(state.qrywell_sync_poll_interval_ms)).await;
        }
    });
}

pub async fn enqueue_runtime_record_upsert(
    pool: &PgPool,
    tenant_id: TenantId,
    entity_logical_name: &str,
    record: &RuntimeRecordResponse,
    max_attempts: i32,
) -> Result<(), AppError> {
    let payload = SyncPayload {
        record_id: record.record_id.clone(),
        entity_logical_name: entity_logical_name.to_owned(),
        title: Some(derive_record_title(entity_logical_name, record)),
        content: Some(flatten_record_data(&record.data)),
        url: None,
        tenant_id: tenant_id.to_string(),
        roles: Vec::new(),
    };

    enqueue_job(
        pool,
        SyncOperation::Upsert,
        payload,
        max_attempts,
        "queued qrywell upsert sync job",
    )
    .await
}

pub async fn enqueue_runtime_record_delete(
    pool: &PgPool,
    tenant_id: TenantId,
    entity_logical_name: &str,
    record_id: &str,
    max_attempts: i32,
) -> Result<(), AppError> {
    let payload = SyncPayload {
        record_id: record_id.to_owned(),
        entity_logical_name: entity_logical_name.to_owned(),
        title: None,
        content: None,
        url: None,
        tenant_id: tenant_id.to_string(),
        roles: Vec::new(),
    };

    enqueue_job(
        pool,
        SyncOperation::Delete,
        payload,
        max_attempts,
        "queued qrywell delete sync job",
    )
    .await
}

async fn enqueue_job(
    pool: &PgPool,
    operation: SyncOperation,
    payload: SyncPayload,
    max_attempts: i32,
    log_message: &str,
) -> Result<(), AppError> {
    let payload_json = serde_json::to_value(&payload).map_err(|error| {
        AppError::Internal(format!("failed to serialize sync payload: {error}"))
    })?;

    sqlx::query(
        r#"
        INSERT INTO qrywell_sync_jobs (
            tenant_id,
            entity_logical_name,
            record_id,
            operation,
            payload,
            status,
            attempt_count,
            max_attempts,
            next_attempt_at,
            last_error,
            created_at,
            updated_at
        )
        VALUES (
            $1::uuid,
            $2,
            $3,
            $4,
            $5::jsonb,
            'pending',
            0,
            $6,
            now(),
            NULL,
            now(),
            now()
        )
        ON CONFLICT (tenant_id, entity_logical_name, record_id)
        DO UPDATE
        SET
            operation = EXCLUDED.operation,
            payload = EXCLUDED.payload,
            status = 'pending',
            attempt_count = 0,
            max_attempts = EXCLUDED.max_attempts,
            next_attempt_at = now(),
            last_error = NULL,
            updated_at = now()
        "#,
    )
    .bind(payload.tenant_id.clone())
    .bind(payload.entity_logical_name.clone())
    .bind(payload.record_id.clone())
    .bind(operation.as_str())
    .bind(payload_json)
    .bind(max_attempts)
    .execute(pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to enqueue qrywell sync job: {error}")))?;

    info!(
        operation = operation.as_str(),
        tenant_id = %payload.tenant_id,
        entity_logical_name = %payload.entity_logical_name,
        record_id = %payload.record_id,
        "{}",
        log_message
    );

    Ok(())
}

async fn process_sync_batch(state: &AppState) -> Result<(), AppError> {
    let jobs = claim_jobs(&state.postgres_pool, state.qrywell_sync_batch_size).await?;
    if jobs.is_empty() {
        return Ok(());
    }

    for job in jobs {
        match execute_job(state, &job).await {
            Ok(()) => {
                mark_job_complete(&state.postgres_pool, job.id, &job.payload.tenant_id).await?
            }
            Err(error) => {
                warn!(
                    error = %error,
                    operation = ?job.operation,
                    record_id = %job.payload.record_id,
                    tenant_id = %job.payload.tenant_id,
                    "qrywell sync job failed"
                );
                mark_job_failed(&state.postgres_pool, &job, &error.to_string()).await?;
            }
        }
    }

    Ok(())
}

async fn claim_jobs(pool: &PgPool, batch_size: usize) -> Result<Vec<SyncJob>, AppError> {
    let rows = sqlx::query_as::<_, ClaimedJobRow>(
        r#"
        WITH candidates AS (
            SELECT id
            FROM qrywell_sync_jobs
            WHERE status IN ('pending', 'failed')
              AND next_attempt_at <= now()
            ORDER BY next_attempt_at ASC, created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE qrywell_sync_jobs AS jobs
        SET status = 'processing',
            updated_at = now(),
            next_attempt_at = now() + interval '2 minutes'
        FROM candidates
        WHERE jobs.id = candidates.id
        RETURNING jobs.id, jobs.operation, jobs.payload, jobs.attempt_count, jobs.max_attempts
        "#,
    )
    .bind(i64::try_from(batch_size).map_err(|error| {
        AppError::Internal(format!(
            "invalid qrywell sync batch size conversion: {error}"
        ))
    })?)
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to claim qrywell sync jobs: {error}")))?;

    rows.into_iter()
        .map(|row| {
            let operation = match row.operation.as_str() {
                "upsert" => SyncOperation::Upsert,
                "delete" => SyncOperation::Delete,
                other => {
                    return Err(AppError::Internal(format!(
                        "unsupported qrywell sync operation '{other}'"
                    )));
                }
            };

            let payload = serde_json::from_value::<SyncPayload>(row.payload).map_err(|error| {
                AppError::Internal(format!("invalid qrywell sync payload format: {error}"))
            })?;

            Ok(SyncJob {
                id: row.id,
                operation,
                attempt_count: row.attempt_count,
                max_attempts: row.max_attempts,
                payload,
            })
        })
        .collect()
}

async fn execute_job(state: &AppState, job: &SyncJob) -> Result<(), AppError> {
    let base_url = state
        .qrywell_api_base_url
        .clone()
        .ok_or_else(|| AppError::Validation("QRYWELL_API_BASE_URL is not configured".to_owned()))?;

    match job.operation {
        SyncOperation::Upsert => {
            let endpoint = format!(
                "{}/v0/connectors/qryvanta/records:ingest",
                base_url.trim_end_matches('/')
            );
            let mut request = state
                .http_client
                .post(endpoint)
                .json(&QrywellIngestRequest {
                    records: vec![QrywellIngestRecord {
                        record_id: job.payload.record_id.clone(),
                        entity_logical_name: job.payload.entity_logical_name.clone(),
                        title: job.payload.title.clone().unwrap_or_else(|| {
                            format!(
                                "{} {}",
                                job.payload.entity_logical_name, job.payload.record_id
                            )
                        }),
                        content: job.payload.content.clone().unwrap_or_default(),
                        url: job.payload.url.clone(),
                        tenant_id: job.payload.tenant_id.clone(),
                        roles: job.payload.roles.clone(),
                    }],
                });
            if let Some(api_key) = &state.qrywell_api_key {
                request = request.header("x-qrywell-api-key", api_key);
            }

            let response = request.send().await.map_err(|error| {
                AppError::Internal(format!("failed calling qrywell ingest endpoint: {error}"))
            })?;

            if !response.status().is_success() {
                let body = response.text().await.unwrap_or_else(|_| String::new());
                return Err(AppError::Internal(format!(
                    "qrywell ingest request failed: {}",
                    body.trim()
                )));
            }

            Ok(())
        }
        SyncOperation::Delete => {
            let endpoint = format!(
                "{}/v0/connectors/qryvanta/records:delete",
                base_url.trim_end_matches('/')
            );
            let mut request = state
                .http_client
                .post(endpoint)
                .json(&QrywellDeleteRequest {
                    record_id: job.payload.record_id.clone(),
                    entity_logical_name: job.payload.entity_logical_name.clone(),
                });
            if let Some(api_key) = &state.qrywell_api_key {
                request = request.header("x-qrywell-api-key", api_key);
            }

            let response = request.send().await.map_err(|error| {
                AppError::Internal(format!("failed calling qrywell delete endpoint: {error}"))
            })?;

            if !response.status().is_success() {
                let body = response.text().await.unwrap_or_else(|_| String::new());
                return Err(AppError::Internal(format!(
                    "qrywell delete request failed: {}",
                    body.trim()
                )));
            }

            Ok(())
        }
    }
}

async fn mark_job_complete(
    pool: &PgPool,
    job_id: uuid::Uuid,
    tenant_id: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM qrywell_sync_jobs WHERE id = $1")
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to delete completed sync job: {error}"))
        })?;

    sqlx::query(
        r#"
        INSERT INTO qrywell_sync_stats (
            tenant_id,
            last_attempt_at,
            last_success_at,
            total_succeeded,
            total_failed,
            updated_at
        )
        VALUES ($1::uuid, now(), now(), 1, 0, now())
        ON CONFLICT (tenant_id)
        DO UPDATE
        SET
            last_attempt_at = now(),
            last_success_at = now(),
            total_succeeded = qrywell_sync_stats.total_succeeded + 1,
            updated_at = now()
        "#,
    )
    .bind(tenant_id)
    .execute(pool)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to update qrywell sync success stats: {error}"
        ))
    })?;

    Ok(())
}

async fn mark_job_failed(
    pool: &PgPool,
    job: &SyncJob,
    error_message: &str,
) -> Result<(), AppError> {
    let attempt_count = job.attempt_count + 1;
    let capped_step = attempt_count.min(10);
    let backoff_seconds = 2_i64
        .pow(u32::try_from(capped_step).map_err(|error| {
            AppError::Internal(format!("invalid sync attempt conversion: {error}"))
        })?)
        .min(1800);
    let next_status = if attempt_count >= job.max_attempts {
        "failed"
    } else {
        "pending"
    };

    sqlx::query(
        r#"
        UPDATE qrywell_sync_jobs
        SET
            status = $2,
            attempt_count = $3,
            next_attempt_at = now() + make_interval(secs => $4),
            last_error = $5,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(job.id)
    .bind(next_status)
    .bind(attempt_count)
    .bind(
        i32::try_from(backoff_seconds).map_err(|error| {
            AppError::Internal(format!("invalid sync backoff conversion: {error}"))
        })?,
    )
    .bind(error_message)
    .execute(pool)
    .await
    .map_err(|error| AppError::Internal(format!("failed to update sync retry state: {error}")))?;

    sqlx::query(
        r#"
        INSERT INTO qrywell_sync_stats (
            tenant_id,
            last_attempt_at,
            last_failure_at,
            total_succeeded,
            total_failed,
            updated_at
        )
        VALUES ($1::uuid, now(), now(), 0, 1, now())
        ON CONFLICT (tenant_id)
        DO UPDATE
        SET
            last_attempt_at = now(),
            last_failure_at = now(),
            total_failed = qrywell_sync_stats.total_failed + 1,
            updated_at = now()
        "#,
    )
    .bind(job.payload.tenant_id.clone())
    .execute(pool)
    .await
    .map_err(|error| {
        AppError::Internal(format!(
            "failed to update qrywell sync failure stats: {error}"
        ))
    })?;

    Ok(())
}

fn derive_record_title(entity_logical_name: &str, record: &RuntimeRecordResponse) -> String {
    let preferred_fields = ["title", "name", "subject", "code", "id"];
    for field in preferred_fields {
        let Some(value) = record.data.get(field) else {
            continue;
        };
        let Some(text) = value.as_str() else {
            continue;
        };
        if !text.trim().is_empty() {
            return text.trim().to_owned();
        }
    }

    format!("{} {}", entity_logical_name, record.record_id)
}

fn flatten_record_data(value: &Value) -> String {
    let mut output = Vec::new();
    flatten_value(String::new(), value, &mut output);
    output.join("\n")
}

fn flatten_value(prefix: String, value: &Value, output: &mut Vec<String>) {
    match value {
        Value::Null => {}
        Value::Bool(boolean) => {
            if !prefix.is_empty() {
                output.push(format!("{}: {}", prefix, boolean));
            }
        }
        Value::Number(number) => {
            if !prefix.is_empty() {
                output.push(format!("{}: {}", prefix, number));
            }
        }
        Value::String(text) => {
            if !prefix.is_empty() && !text.trim().is_empty() {
                output.push(format!("{}: {}", prefix, text.trim()));
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let path = if prefix.is_empty() {
                    format!("[{index}]")
                } else {
                    format!("{}[{index}]", prefix)
                };
                flatten_value(path, item, output);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                let path = if prefix.is_empty() {
                    key.to_owned()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_value(path, item, output);
            }
        }
    }
}
