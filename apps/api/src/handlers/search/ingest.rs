use std::collections::BTreeMap;

use qryvanta_core::{AppError, UserIdentity};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dto::RuntimeRecordResponse;
use crate::state::AppState;

pub(super) async fn push_records_to_qrywell(
    state: &AppState,
    user: &UserIdentity,
    entity_logical_name: &str,
    records: &[RuntimeRecordResponse],
) -> Result<QrywellIngestResponse, AppError> {
    let base_url = state
        .qrywell_api_base_url
        .clone()
        .ok_or_else(|| AppError::Validation("QRYWELL_API_BASE_URL is not configured".to_owned()))?;
    let ingest_records = records
        .iter()
        .map(|record| QrywellIngestRecord {
            record_id: record.record_id.clone(),
            entity_logical_name: entity_logical_name.to_owned(),
            title: derive_record_title(entity_logical_name, record),
            content: flatten_record_data(&record.data),
            url: None,
            tenant_id: user.tenant_id().to_string(),
            roles: Vec::new(),
            facets: extract_record_facets(entity_logical_name, record),
        })
        .collect::<Vec<_>>();

    let endpoint = format!(
        "{}/v0/connectors/qryvanta/records:ingest",
        base_url.trim_end_matches('/')
    );
    let mut request = state
        .http_client
        .post(endpoint)
        .json(&QrywellIngestRequest {
            records: ingest_records,
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

    response
        .json::<QrywellIngestResponse>()
        .await
        .map_err(|error| AppError::Internal(format!("invalid qrywell ingest response: {error}")))
}

fn derive_record_title(entity_logical_name: &str, record: &RuntimeRecordResponse) -> String {
    let Some(object) = record.data.as_object() else {
        return format!("{} {}", entity_logical_name, record.record_id);
    };

    let mut best: Option<(&str, i32)> = None;
    for (key, value) in object {
        let Some(text) = value.as_str() else {
            continue;
        };
        let candidate = text.trim();
        if candidate.is_empty() {
            continue;
        }

        let mut score = 0_i32;
        let length = candidate.chars().count();
        if (3..=80).contains(&length) {
            score += 3;
        }
        if candidate.chars().any(char::is_alphabetic) {
            score += 2;
        }
        if candidate.split_whitespace().count() <= 8 {
            score += 1;
        }
        if key.to_lowercase().ends_with("_id") {
            score -= 3;
        }
        if looks_like_uuid(candidate) {
            score -= 4;
        }
        if candidate.contains('@') {
            score -= 2;
        }

        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((candidate, score));
        }
    }

    if let Some((title, _)) = best {
        return title.to_owned();
    }

    format!("{} {}", entity_logical_name, record.record_id)
}

fn looks_like_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value.trim()).is_ok()
}

fn flatten_record_data(value: &Value) -> String {
    let mut output = Vec::new();
    flatten_value(String::new(), value, &mut output);
    output.join("\n")
}

fn extract_record_facets(
    entity_logical_name: &str,
    record: &RuntimeRecordResponse,
) -> BTreeMap<String, String> {
    let mut facets = BTreeMap::new();
    facets.insert(
        "entity".to_owned(),
        entity_logical_name.trim().to_lowercase(),
    );

    if let Some(object) = record.data.as_object() {
        for (key, value) in object {
            let normalized_key = key.trim().to_lowercase();
            if normalized_key.is_empty() {
                continue;
            }

            let normalized_value = match value {
                Value::String(text) => {
                    let cleaned = text.trim().to_lowercase();
                    if cleaned.is_empty() {
                        continue;
                    }
                    cleaned
                }
                Value::Bool(boolean) => boolean.to_string(),
                Value::Number(number) => number.to_string(),
                _ => continue,
            };

            facets.insert(normalized_key, normalized_value);
        }
    }

    facets
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
    facets: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct QrywellIngestResponse {
    pub(super) indexed_records: usize,
    pub(super) indexed_chunks: usize,
}
