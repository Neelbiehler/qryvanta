use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use crate::audit_chain::{AuditChainInput, compute_audit_entry_hash};
use crate::begin_tenant_transaction;
use qryvanta_application::{
    AuditIntegrityStatus, AuditLogEntry, AuditLogQuery, AuditLogRepository,
};
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
    tenant_id: uuid::Uuid,
    subject: String,
    action: String,
    resource_type: String,
    resource_id: String,
    detail: Option<String>,
    created_at: String,
    chain_position: i64,
    previous_entry_hash: Option<String>,
    entry_hash: String,
}

#[async_trait]
impl AuditLogRepository for PostgresAuditLogRepository {
    async fn list_recent_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let capped_limit = query.limit.clamp(1, 200) as i64;
        let capped_offset = query.offset.min(5_000) as i64;
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id AS event_id,
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at,
                chain_position,
                previous_entry_hash,
                entry_hash
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
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to list audit log entries: {error}"))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped audit list transaction: {error}"
            ))
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
                chain_position: row.chain_position,
                previous_entry_hash: row.previous_entry_hash,
                entry_hash: row.entry_hash,
            })
            .collect())
    }

    async fn export_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let capped_limit = query.limit.clamp(1, 5_000) as i64;
        let capped_offset = query.offset.min(100_000) as i64;
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id AS event_id,
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at,
                chain_position,
                previous_entry_hash,
                entry_hash
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
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to export audit log entries: {error}"))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped audit export transaction: {error}"
            ))
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
                chain_position: row.chain_position,
                previous_entry_hash: row.previous_entry_hash,
                entry_hash: row.entry_hash,
            })
            .collect())
    }

    async fn purge_entries_older_than(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<u64> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let result = sqlx::query(
            r#"
            DELETE FROM audit_log_entries
            WHERE tenant_id = $1
              AND created_at < now() - make_interval(days => $2::INTEGER)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(i32::from(retention_days))
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to purge audit log entries: {error}"))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped audit purge transaction: {error}"
            ))
        })?;

        Ok(result.rows_affected())
    }

    async fn verify_integrity(&self, tenant_id: TenantId) -> AppResult<AuditIntegrityStatus> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id AS event_id,
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at,
                chain_position,
                previous_entry_hash,
                entry_hash
            FROM audit_log_entries
            WHERE tenant_id = $1
            ORDER BY chain_position ASC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to verify audit log integrity: {error}"))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped audit verification transaction: {error}"
            ))
        })?;

        let mut failures = Vec::new();
        let mut previous_hash: Option<String> = None;
        let mut latest_chain_position = None;
        let mut latest_entry_hash = None;

        for (index, row) in rows.iter().enumerate() {
            let expected_position = i64::try_from(index + 1).unwrap_or(i64::MAX);
            if row.chain_position != expected_position {
                failures.push(format!(
                    "event {} expected chain_position {}, found {}",
                    row.event_id, expected_position, row.chain_position
                ));
            }

            if row.previous_entry_hash.as_deref() != previous_hash.as_deref() {
                failures.push(format!(
                    "event {} previous_entry_hash mismatch at chain_position {}",
                    row.event_id, row.chain_position
                ));
            }

            let computed_hash = compute_audit_entry_hash(&AuditChainInput {
                tenant_id: TenantId::from_uuid(row.tenant_id),
                chain_position: row.chain_position,
                previous_entry_hash: row.previous_entry_hash.as_deref(),
                subject: &row.subject,
                action: &row.action,
                resource_type: &row.resource_type,
                resource_id: &row.resource_id,
                detail: row.detail.as_deref(),
                created_at_utc: &row.created_at,
            });
            if row.entry_hash != computed_hash {
                failures.push(format!(
                    "event {} entry_hash mismatch at chain_position {}",
                    row.event_id, row.chain_position
                ));
            }

            previous_hash = Some(row.entry_hash.clone());
            latest_chain_position = Some(row.chain_position);
            latest_entry_hash = Some(row.entry_hash.clone());
        }

        Ok(AuditIntegrityStatus {
            is_valid: failures.is_empty(),
            verified_entries: rows.len(),
            latest_chain_position,
            latest_entry_hash,
            failures,
        })
    }
}

#[cfg(test)]
mod tests;
