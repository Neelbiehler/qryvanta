use chrono::Utc;
use qryvanta_application::{
    CreateWorkflowRunInput, WorkflowQueueStatsQuery, WorkflowRepository, WorkflowRunAttempt,
    WorkflowRunAttemptStatus,
};
use qryvanta_core::TenantId;
use qryvanta_domain::{
    WorkflowAction, WorkflowDefinition, WorkflowDefinitionInput, WorkflowTrigger,
};
use serde_json::json;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

use super::PostgresWorkflowRepository;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn test_pool() -> Option<PgPool> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        return None;
    };

    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .connect(database_url.as_str())
        .await
    {
        Ok(pool) => pool,
        Err(error) => panic!("failed to connect to DATABASE_URL in test: {error}"),
    };

    if let Err(error) = MIGRATOR.run(&pool).await {
        panic!("failed to run migrations for postgres workflow tests: {error}");
    }

    Some(pool)
}

async fn ensure_tenant(pool: &PgPool, tenant_id: TenantId, name: &str) {
    let insert = sqlx::query(
        r#"
            INSERT INTO tenants (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id) DO NOTHING
            "#,
    )
    .bind(tenant_id.as_uuid())
    .bind(name)
    .execute(pool)
    .await;

    assert!(insert.is_ok());
}

fn workflow(logical_name: &str, display_name: &str) -> WorkflowDefinition {
    WorkflowDefinition::new(WorkflowDefinitionInput {
        logical_name: logical_name.to_owned(),
        display_name: display_name.to_owned(),
        description: None,
        trigger: WorkflowTrigger::Manual,
        action: WorkflowAction::LogMessage {
            message: format!("{display_name} executed"),
        },
        steps: None,
        max_attempts: 3,
        is_enabled: true,
    })
    .unwrap_or_else(|_| unreachable!())
}

#[tokio::test]
async fn workflow_repository_reads_are_tenant_scoped() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresWorkflowRepository::new(pool.clone());
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();
    ensure_tenant(&pool, left_tenant, "Workflow Left Tenant").await;
    ensure_tenant(&pool, right_tenant, "Workflow Right Tenant").await;

    assert!(
        repository
            .save_workflow(left_tenant, workflow("left_ops", "Left Ops"))
            .await
            .is_ok()
    );

    let left_run = repository
        .create_run(
            left_tenant,
            CreateWorkflowRunInput {
                workflow_logical_name: "left_ops".to_owned(),
                trigger_type: "manual".to_owned(),
                trigger_entity_logical_name: None,
                trigger_payload: json!({"source": "test"}),
            },
        )
        .await;
    assert!(left_run.is_ok());
    let left_run = left_run.unwrap_or_else(|_| unreachable!());

    assert!(
        repository
            .append_run_attempt(
                left_tenant,
                WorkflowRunAttempt {
                    run_id: left_run.run_id.clone(),
                    attempt_number: 1,
                    status: WorkflowRunAttemptStatus::Succeeded,
                    error_message: None,
                    executed_at: Utc::now(),
                    step_traces: Vec::new(),
                },
            )
            .await
            .is_ok()
    );

    let right_workflow = repository.find_workflow(right_tenant, "left_ops").await;
    assert!(right_workflow.is_ok());
    assert!(right_workflow.unwrap_or_default().is_none());

    let right_run = repository
        .find_run(right_tenant, left_run.run_id.as_str())
        .await;
    assert!(right_run.is_ok());
    assert!(right_run.unwrap_or_default().is_none());

    let right_attempts = repository
        .list_run_attempts(right_tenant, left_run.run_id.as_str())
        .await;
    assert!(right_attempts.is_ok());
    assert!(right_attempts.unwrap_or_default().is_empty());
}

#[tokio::test]
async fn workflow_job_claims_use_operational_bypass_across_tenants() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresWorkflowRepository::new(pool.clone());
    let left_tenant = TenantId::new();
    let right_tenant = TenantId::new();
    ensure_tenant(&pool, left_tenant, "Workflow Queue Left Tenant").await;
    ensure_tenant(&pool, right_tenant, "Workflow Queue Right Tenant").await;

    assert!(
        repository
            .save_workflow(left_tenant, workflow("left_queue", "Left Queue"))
            .await
            .is_ok()
    );
    assert!(
        repository
            .save_workflow(right_tenant, workflow("right_queue", "Right Queue"))
            .await
            .is_ok()
    );

    let left_run = repository
        .create_run(
            left_tenant,
            CreateWorkflowRunInput {
                workflow_logical_name: "left_queue".to_owned(),
                trigger_type: "manual".to_owned(),
                trigger_entity_logical_name: None,
                trigger_payload: json!({"tenant": "left"}),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());
    let right_run = repository
        .create_run(
            right_tenant,
            CreateWorkflowRunInput {
                workflow_logical_name: "right_queue".to_owned(),
                trigger_type: "manual".to_owned(),
                trigger_entity_logical_name: None,
                trigger_payload: json!({"tenant": "right"}),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(
        repository
            .enqueue_run_job(left_tenant, left_run.run_id.as_str())
            .await
            .is_ok()
    );
    assert!(
        repository
            .enqueue_run_job(right_tenant, right_run.run_id.as_str())
            .await
            .is_ok()
    );

    let claimed = repository.claim_jobs("worker-1", 10, 60, None, None).await;
    assert!(claimed.is_ok());
    let mut claimed = claimed.unwrap_or_default();
    claimed.sort_by_key(|job| job.tenant_id.to_string());
    let claimed_tenant_ids: Vec<TenantId> = claimed.into_iter().map(|job| job.tenant_id).collect();

    assert!(claimed_tenant_ids.len() >= 2);
    assert!(claimed_tenant_ids.contains(&left_tenant));
    assert!(claimed_tenant_ids.contains(&right_tenant));

    let queue_stats = repository
        .queue_stats(WorkflowQueueStatsQuery {
            active_window_seconds: 120,
            partition: None,
        })
        .await;
    assert!(queue_stats.is_ok());
    assert!(queue_stats.unwrap_or_else(|_| unreachable!()).leased_jobs >= 2);
}
