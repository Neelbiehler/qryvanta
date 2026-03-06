use async_trait::async_trait;
use sqlx::FromRow;
use sqlx::PgPool;

use crate::audit_chain::{AuditChainInput, compute_audit_entry_hash};
use crate::begin_tenant_transaction;
use qryvanta_application::{AuditEvent, AuditRepository};
use qryvanta_core::{AppError, AppResult};

/// PostgreSQL-backed append-only audit repository.
#[derive(Clone)]
pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct LatestAuditChainRow {
    chain_position: i64,
    entry_hash: String,
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
        let mut transaction = begin_tenant_transaction(&self.pool, event.tenant_id).await?;
        let latest_chain = sqlx::query_as::<_, LatestAuditChainRow>(
            r#"
            SELECT chain_position, entry_hash
            FROM audit_log_entries
            WHERE tenant_id = $1
            ORDER BY chain_position DESC
            LIMIT 1
            "#,
        )
        .bind(event.tenant_id.as_uuid())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to query latest tenant audit chain position: {error}"
            ))
        })?;
        let next_chain_position = latest_chain
            .as_ref()
            .map_or(1_i64, |row| row.chain_position + 1);
        let previous_entry_hash = latest_chain.as_ref().map(|row| row.entry_hash.as_str());
        let created_at = chrono::Utc::now();
        let created_at_utc = created_at.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
        let entry_hash = compute_audit_entry_hash(&AuditChainInput {
            tenant_id: event.tenant_id,
            chain_position: next_chain_position,
            previous_entry_hash,
            subject: &event.subject,
            action: event.action.as_str(),
            resource_type: &event.resource_type,
            resource_id: &event.resource_id,
            detail: event.detail.as_deref(),
            created_at_utc: created_at_utc.as_str(),
        });

        sqlx::query(
            r#"
            INSERT INTO audit_log_entries (
                tenant_id,
                subject,
                action,
                resource_type,
                resource_id,
                detail,
                created_at,
                chain_position,
                previous_entry_hash,
                entry_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(event.tenant_id.as_uuid())
        .bind(event.subject)
        .bind(event.action.as_str())
        .bind(event.resource_type)
        .bind(event.resource_id)
        .bind(event.detail)
        .bind(created_at)
        .bind(next_chain_position)
        .bind(previous_entry_hash)
        .bind(entry_hash)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to append audit event: {error}")))?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped audit append transaction: {error}"
            ))
        })?;

        Ok(())
    }
}
