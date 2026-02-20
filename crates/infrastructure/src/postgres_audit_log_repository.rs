use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use qryvanta_application::{AuditLogEntry, AuditLogQuery, AuditLogRepository};
use qryvanta_core::{AppError, AppResult, TenantId};

/// PostgreSQL-backed repository for audit log read models.
#[derive(Clone)]
pub struct PostgresAuditLogRepository {
    pool: PgPool,
}

impl PostgresAuditLogRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct AuditLogRow {
    event_id: uuid::Uuid,
    subject: String,
    action: String,
    resource_type: String,
    resource_id: String,
    detail: Option<String>,
    created_at: String,
}

#[async_trait]
impl AuditLogRepository for PostgresAuditLogRepository {
    async fn list_recent_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        let capped_limit = query.limit.clamp(1, 200) as i64;
        let capped_offset = query.offset.min(5_000) as i64;
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id AS event_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
            FROM audit_log_entries
            WHERE tenant_id = $1
                AND ($2::TEXT IS NULL OR action = $2)
                AND ($3::TEXT IS NULL OR subject = $3)
            ORDER BY created_at DESC
            LIMIT $4
            OFFSET $5
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(query.action)
        .bind(query.subject)
        .bind(capped_limit)
        .bind(capped_offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to list audit log entries: {error}"))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| AuditLogEntry {
                event_id: row.event_id.to_string(),
                subject: row.subject,
                action: row.action,
                resource_type: row.resource_type,
                resource_id: row.resource_id,
                detail: row.detail,
                created_at: row.created_at,
            })
            .collect())
    }
}
