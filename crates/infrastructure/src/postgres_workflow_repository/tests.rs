use chrono::Utc;
use qryvanta_application::{
    CreateWorkflowRunInput, WorkflowQueueStatsQuery, WorkflowRepository, WorkflowRunAttempt,
    WorkflowRunAttemptStatus,
};
use qryvanta_core::TenantId;
use qryvanta_domain::{WorkflowDefinition, WorkflowDefinitionInput, WorkflowStep, WorkflowTrigger};
use serde_json::json;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Uuid;

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
    workflow_with_trigger(logical_name, display_name, WorkflowTrigger::Manual)
}

fn workflow_with_trigger(
    logical_name: &str,
    display_name: &str,
    trigger: WorkflowTrigger,
) -> WorkflowDefinition {
    WorkflowDefinition::new(WorkflowDefinitionInput {
        logical_name: logical_name.to_owned(),
        display_name: display_name.to_owned(),
        description: None,
        trigger,
        steps: vec![WorkflowStep::LogMessage {
            message: format!("{display_name} executed"),
        }],
        max_attempts: 3,
    })
    .unwrap_or_else(|_| unreachable!())
}

async fn save_and_publish_workflow(
    repository: &PostgresWorkflowRepository,
    tenant_id: TenantId,
    workflow: WorkflowDefinition,
) -> WorkflowDefinition {
    let logical_name = workflow.logical_name().as_str().to_owned();
    assert!(repository.save_workflow(tenant_id, workflow).await.is_ok());

    repository
        .publish_workflow(tenant_id, logical_name.as_str(), "postgres-test")
        .await
        .unwrap_or_else(|_| unreachable!())
}

#[tokio::test]
async fn workflow_repository_persists_expanded_trigger_types() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresWorkflowRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Workflow Trigger Coverage Tenant").await;

    let workflows = vec![
        workflow_with_trigger(
            "contact_updated_ops",
            "Contact Updated Ops",
            WorkflowTrigger::RuntimeRecordUpdated {
                entity_logical_name: "contact".to_owned(),
            },
        ),
        workflow_with_trigger(
            "contact_deleted_ops",
            "Contact Deleted Ops",
            WorkflowTrigger::RuntimeRecordDeleted {
                entity_logical_name: "contact".to_owned(),
            },
        ),
        workflow_with_trigger(
            "hourly_ops",
            "Hourly Ops",
            WorkflowTrigger::ScheduleTick {
                schedule_key: "hourly".to_owned(),
            },
        ),
    ];

    for workflow in workflows {
        let _ = save_and_publish_workflow(&repository, tenant_id, workflow).await;
    }

    let updated_workflow = repository
        .find_workflow(tenant_id, "contact_updated_ops")
        .await
        .unwrap_or_else(|error| panic!("failed to load updated workflow: {error}"))
        .unwrap_or_else(|| unreachable!());
    assert_eq!(
        updated_workflow.trigger(),
        &WorkflowTrigger::RuntimeRecordUpdated {
            entity_logical_name: "contact".to_owned(),
        }
    );

    let deleted_workflow = repository
        .find_workflow(tenant_id, "contact_deleted_ops")
        .await
        .unwrap_or_else(|error| panic!("failed to load deleted workflow: {error}"))
        .unwrap_or_else(|| unreachable!());
    assert_eq!(
        deleted_workflow.trigger(),
        &WorkflowTrigger::RuntimeRecordDeleted {
            entity_logical_name: "contact".to_owned(),
        }
    );

    let schedule_workflow = repository
        .find_workflow(tenant_id, "hourly_ops")
        .await
        .unwrap_or_else(|error| panic!("failed to load scheduled workflow: {error}"))
        .unwrap_or_else(|| unreachable!());
    assert_eq!(
        schedule_workflow.trigger(),
        &WorkflowTrigger::ScheduleTick {
            schedule_key: "hourly".to_owned(),
        }
    );
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

    let left_workflow =
        save_and_publish_workflow(&repository, left_tenant, workflow("left_ops", "Left Ops")).await;

    let left_run = repository
        .create_run(
            left_tenant,
            CreateWorkflowRunInput {
                workflow_logical_name: "left_ops".to_owned(),
                workflow_version: left_workflow.published_version().unwrap_or_default(),
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

    let left_workflow = save_and_publish_workflow(
        &repository,
        left_tenant,
        workflow("left_queue", "Left Queue"),
    )
    .await;
    let right_workflow = save_and_publish_workflow(
        &repository,
        right_tenant,
        workflow("right_queue", "Right Queue"),
    )
    .await;

    let left_run = repository
        .create_run(
            left_tenant,
            CreateWorkflowRunInput {
                workflow_logical_name: "left_queue".to_owned(),
                workflow_version: left_workflow.published_version().unwrap_or_default(),
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
                workflow_version: right_workflow.published_version().unwrap_or_default(),
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

#[tokio::test]
async fn workflow_job_claims_reclaim_expired_leases_with_new_fencing_tokens() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresWorkflowRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Workflow Lease Reclaim Tenant").await;

    let workflow = save_and_publish_workflow(
        &repository,
        tenant_id,
        workflow("lease_reclaim", "Lease Reclaim"),
    )
    .await;

    let run = repository
        .create_run(
            tenant_id,
            CreateWorkflowRunInput {
                workflow_logical_name: "lease_reclaim".to_owned(),
                workflow_version: workflow.published_version().unwrap_or_default(),
                trigger_type: "manual".to_owned(),
                trigger_entity_logical_name: None,
                trigger_payload: json!({"source": "lease-reclaim"}),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(
        repository
            .enqueue_run_job(tenant_id, run.run_id.as_str())
            .await
            .is_ok()
    );

    let first_claim = repository
        .claim_jobs("worker-1", 1, 60, None, Some(tenant_id))
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(first_claim.len(), 1);
    let first_claimed_job = &first_claim[0];

    let expired = sqlx::query(
        r#"
        UPDATE workflow_execution_jobs
        SET lease_expires_at = now() - interval '5 minutes'
        WHERE id = $1
        "#,
    )
    .bind(Uuid::parse_str(first_claimed_job.job_id.as_str()).unwrap_or_else(|_| unreachable!()))
    .execute(&pool)
    .await;
    assert!(expired.is_ok());

    let queue_stats = repository
        .queue_stats(WorkflowQueueStatsQuery {
            active_window_seconds: 120,
            partition: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(queue_stats.expired_leases >= 1);

    let second_claim = repository
        .claim_jobs("worker-2", 1, 60, None, Some(tenant_id))
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(second_claim.len(), 1);
    let second_claimed_job = &second_claim[0];
    assert_eq!(second_claimed_job.job_id, first_claimed_job.job_id);
    assert_ne!(
        second_claimed_job.lease_token,
        first_claimed_job.lease_token
    );

    let stale_complete = repository
        .complete_job(
            tenant_id,
            first_claimed_job.job_id.as_str(),
            "worker-1",
            first_claimed_job.lease_token.as_str(),
        )
        .await;
    assert!(stale_complete.is_err());

    let recovered_complete = repository
        .complete_job(
            tenant_id,
            second_claimed_job.job_id.as_str(),
            "worker-2",
            second_claimed_job.lease_token.as_str(),
        )
        .await;
    assert!(recovered_complete.is_ok());
}

#[tokio::test]
async fn workflow_schedule_ticks_reclaim_expired_leases_with_new_fencing_tokens() {
    let Some(pool) = test_pool().await else {
        return;
    };

    let repository = PostgresWorkflowRepository::new(pool.clone());
    let tenant_id = TenantId::new();
    ensure_tenant(&pool, tenant_id, "Workflow Schedule Lease Reclaim Tenant").await;

    let scheduled_for = Utc::now();
    let first_claim = repository
        .claim_schedule_tick(
            tenant_id,
            "hourly",
            "2026-03-07T10",
            scheduled_for,
            "worker-1",
            60,
        )
        .await
        .unwrap_or_else(|_| unreachable!())
        .unwrap_or_else(|| unreachable!());

    let expired = sqlx::query(
        r#"
        UPDATE workflow_schedule_ticks
        SET lease_expires_at = now() - interval '5 minutes'
        WHERE tenant_id = $1
          AND schedule_key = $2
          AND slot_key = $3
        "#,
    )
    .bind(tenant_id.as_uuid())
    .bind("hourly")
    .bind("2026-03-07T10")
    .execute(&pool)
    .await;
    assert!(expired.is_ok());

    let second_claim = repository
        .claim_schedule_tick(
            tenant_id,
            "hourly",
            "2026-03-07T10",
            scheduled_for,
            "worker-2",
            60,
        )
        .await
        .unwrap_or_else(|_| unreachable!())
        .unwrap_or_else(|| unreachable!());

    assert_ne!(second_claim.lease_token, first_claim.lease_token);
    assert_eq!(second_claim.worker_id, "worker-2");

    let stale_complete = repository
        .complete_schedule_tick(
            tenant_id,
            "hourly",
            "2026-03-07T10",
            "worker-1",
            first_claim.lease_token.as_str(),
        )
        .await;
    assert!(stale_complete.is_err());

    let recovered_complete = repository
        .complete_schedule_tick(
            tenant_id,
            "hourly",
            "2026-03-07T10",
            "worker-2",
            second_claim.lease_token.as_str(),
        )
        .await;
    assert!(recovered_complete.is_ok());
}
