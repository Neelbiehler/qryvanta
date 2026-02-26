use serde::{Deserialize, Serialize};

use qryvanta_application::AuditLogEntry;

use crate::dto::WorkspacePublishHistoryEntryResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublishRunAuditDetail {
    requested_entities: usize,
    requested_apps: usize,
    #[serde(default)]
    requested_entity_logical_names: Vec<String>,
    #[serde(default)]
    requested_app_logical_names: Vec<String>,
    published_entities: Vec<String>,
    validated_apps: Vec<String>,
    issue_count: usize,
    is_publishable: bool,
}

pub(super) fn map_workspace_publish_history_entries(
    entries: Vec<AuditLogEntry>,
) -> Vec<WorkspacePublishHistoryEntryResponse> {
    let mut history = Vec::new();
    for entry in entries {
        let Some(detail) = entry
            .detail
            .as_deref()
            .and_then(|value| serde_json::from_str::<PublishRunAuditDetail>(value).ok())
        else {
            continue;
        };

        history.push(WorkspacePublishHistoryEntryResponse {
            run_id: entry.event_id,
            run_at: entry.created_at,
            subject: entry.subject,
            requested_entities: detail.requested_entities,
            requested_apps: detail.requested_apps,
            requested_entity_logical_names: if detail.requested_entity_logical_names.is_empty() {
                detail.published_entities.clone()
            } else {
                detail.requested_entity_logical_names
            },
            requested_app_logical_names: if detail.requested_app_logical_names.is_empty() {
                detail.validated_apps.clone()
            } else {
                detail.requested_app_logical_names
            },
            published_entities: detail.published_entities,
            validated_apps: detail.validated_apps,
            issue_count: detail.issue_count,
            is_publishable: detail.is_publishable,
        });
    }

    history
}
