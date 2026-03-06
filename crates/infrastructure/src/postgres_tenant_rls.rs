use qryvanta_core::{AppError, AppResult, TenantId};
use sqlx::{Executor, PgPool, Postgres, Transaction};

const RLS_TENANT_SETTING: &str = "qryvanta.current_tenant_id";
const RLS_SCOPE_SETTING: &str = "qryvanta.rls_scope";
const RLS_SUBJECT_SETTING: &str = "qryvanta.rls_subject";
const WORKFLOW_QUEUE_SCOPE: &str = "workflow_queue";
const QRYWELL_SYNC_SCOPE: &str = "qrywell_sync";
const MEMBERSHIP_SUBJECT_LOOKUP_SCOPE: &str = "membership_subject_lookup";

/// Begins a transaction and stamps the current tenant into the PostgreSQL
/// session so row-level security policies can enforce tenant isolation.
pub async fn begin_tenant_transaction(
    pool: &PgPool,
    tenant_id: TenantId,
) -> AppResult<Transaction<'_, Postgres>> {
    let mut transaction = pool.begin().await.map_err(|error| {
        AppError::Internal(format!(
            "failed to begin tenant-scoped transaction: {error}"
        ))
    })?;

    stamp_tenant_context(&mut *transaction, tenant_id).await?;

    Ok(transaction)
}

/// Begins a transaction with the workflow queue bypass scope enabled so
/// worker claim and queue-observability paths can access tenant-owned rows
/// across partitions without disabling row-level security globally.
pub async fn begin_workflow_worker_transaction(
    pool: &PgPool,
) -> AppResult<Transaction<'_, Postgres>> {
    begin_rls_scope_transaction(pool, WORKFLOW_QUEUE_SCOPE).await
}

/// Begins a transaction with the Qrywell sync bypass scope enabled so the
/// background sync worker can claim queued jobs across tenants while tenant
/// writes stay row-level-security protected.
pub async fn begin_qrywell_sync_transaction(pool: &PgPool) -> AppResult<Transaction<'_, Postgres>> {
    begin_rls_scope_transaction(pool, QRYWELL_SYNC_SCOPE).await
}

/// Begins a transaction with a single-subject membership lookup scope enabled.
pub(crate) async fn begin_membership_subject_lookup_transaction<'a>(
    pool: &'a PgPool,
    subject: &str,
) -> AppResult<Transaction<'a, Postgres>> {
    let mut transaction =
        begin_rls_scope_transaction(pool, MEMBERSHIP_SUBJECT_LOOKUP_SCOPE).await?;
    set_rls_subject(&mut *transaction, subject).await?;
    Ok(transaction)
}

pub(crate) async fn stamp_tenant_context<'e, E>(executor: E, tenant_id: TenantId) -> AppResult<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query("SELECT set_config($1, $2, true)")
        .bind(RLS_TENANT_SETTING)
        .bind(tenant_id.to_string())
        .execute(executor)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to set tenant RLS context: {error}"))
        })?;

    Ok(())
}

async fn begin_rls_scope_transaction<'a>(
    pool: &'a PgPool,
    scope: &str,
) -> AppResult<Transaction<'a, Postgres>> {
    let mut transaction = pool.begin().await.map_err(|error| {
        AppError::Internal(format!(
            "failed to begin scope-scoped transaction for '{scope}': {error}"
        ))
    })?;

    set_rls_scope(&mut *transaction, scope).await?;

    Ok(transaction)
}

async fn set_rls_scope<'e, E>(executor: E, scope: &str) -> AppResult<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query("SELECT set_config($1, $2, true)")
        .bind(RLS_SCOPE_SETTING)
        .bind(scope)
        .execute(executor)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to set RLS scope '{scope}': {error}"))
        })?;

    Ok(())
}

async fn set_rls_subject<'e, E>(executor: E, subject: &str) -> AppResult<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query("SELECT set_config($1, $2, true)")
        .bind(RLS_SUBJECT_SETTING)
        .bind(subject)
        .execute(executor)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to set RLS subject context for '{subject}': {error}"
            ))
        })?;

    Ok(())
}
