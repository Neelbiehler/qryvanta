use async_trait::async_trait;
use qryvanta_application::{
    CompleteWorkflowRunInput, CreateWorkflowRunInput, WorkflowRepository, WorkflowRun,
    WorkflowRunAttempt, WorkflowRunAttemptStatus, WorkflowRunListQuery, WorkflowRunStatus,
};
use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{WorkflowAction, WorkflowDefinition, WorkflowStep, WorkflowTrigger};
use serde_json::Value;
use sqlx::{FromRow, PgPool};

/// PostgreSQL-backed workflow repository.
#[derive(Clone)]
pub struct PostgresWorkflowRepository {
    pool: PgPool,
}

impl PostgresWorkflowRepository {
    /// Creates a workflow repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct WorkflowDefinitionRow {
    logical_name: String,
    display_name: String,
    description: Option<String>,
    trigger_type: String,
    trigger_entity_logical_name: Option<String>,
    action_type: String,
    action_entity_logical_name: Option<String>,
    action_payload: Value,
    action_steps: Option<Value>,
    max_attempts: i16,
    is_enabled: bool,
}

#[derive(Debug, FromRow)]
struct WorkflowRunRow {
    id: uuid::Uuid,
    workflow_logical_name: String,
    trigger_type: String,
    trigger_entity_logical_name: Option<String>,
    trigger_payload: Value,
    status: String,
    attempts: i32,
    dead_letter_reason: Option<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow)]
struct WorkflowRunAttemptRow {
    run_id: uuid::Uuid,
    attempt_number: i32,
    status: String,
    error_message: Option<String>,
    executed_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
impl WorkflowRepository for PostgresWorkflowRepository {
    async fn save_workflow(
        &self,
        tenant_id: TenantId,
        workflow: WorkflowDefinition,
    ) -> AppResult<()> {
        let (trigger_type, trigger_entity) = workflow_trigger_parts(workflow.trigger());
        let (action_type, action_entity, action_payload) = workflow_action_parts(workflow.action());
        let action_steps = workflow_steps_to_json(workflow.steps())?;

        sqlx::query(
            r#"
            INSERT INTO workflow_definitions (
                tenant_id,
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, now())
            ON CONFLICT (tenant_id, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                description = EXCLUDED.description,
                trigger_type = EXCLUDED.trigger_type,
                trigger_entity_logical_name = EXCLUDED.trigger_entity_logical_name,
                action_type = EXCLUDED.action_type,
                action_entity_logical_name = EXCLUDED.action_entity_logical_name,
                action_payload = EXCLUDED.action_payload,
                action_steps = EXCLUDED.action_steps,
                max_attempts = EXCLUDED.max_attempts,
                is_enabled = EXCLUDED.is_enabled,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(workflow.logical_name().as_str())
        .bind(workflow.display_name().as_str())
        .bind(workflow.description())
        .bind(trigger_type)
        .bind(trigger_entity)
        .bind(action_type)
        .bind(action_entity)
        .bind(action_payload)
        .bind(action_steps)
        .bind(i16::try_from(workflow.max_attempts()).map_err(|error| {
            AppError::Validation(format!("invalid workflow max_attempts value: {error}"))
        })?)
        .bind(workflow.is_enabled())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save workflow '{}' for tenant '{}': {error}",
                workflow.logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    async fn list_workflows(&self, tenant_id: TenantId) -> AppResult<Vec<WorkflowDefinition>> {
        let rows = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled
            FROM workflow_definitions
            WHERE tenant_id = $1
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list workflows for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter().map(workflow_definition_from_row).collect()
    }

    async fn find_workflow(
        &self,
        tenant_id: TenantId,
        logical_name: &str,
    ) -> AppResult<Option<WorkflowDefinition>> {
        let row = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled
            FROM workflow_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find workflow '{}' for tenant '{}': {error}",
                logical_name, tenant_id
            ))
        })?;

        row.map(workflow_definition_from_row).transpose()
    }

    async fn list_enabled_workflows_for_trigger(
        &self,
        tenant_id: TenantId,
        trigger: &WorkflowTrigger,
    ) -> AppResult<Vec<WorkflowDefinition>> {
        let (trigger_type, trigger_entity) = workflow_trigger_parts(trigger);

        let rows = sqlx::query_as::<_, WorkflowDefinitionRow>(
            r#"
            SELECT
                logical_name,
                display_name,
                description,
                trigger_type,
                trigger_entity_logical_name,
                action_type,
                action_entity_logical_name,
                action_payload,
                action_steps,
                max_attempts,
                is_enabled
            FROM workflow_definitions
            WHERE tenant_id = $1
              AND is_enabled = true
              AND trigger_type = $2
              AND (
                    (trigger_entity_logical_name IS NULL AND $3::TEXT IS NULL)
                    OR trigger_entity_logical_name = $3
                  )
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(trigger_type)
        .bind(trigger_entity)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list trigger workflows for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter().map(workflow_definition_from_row).collect()
    }

    async fn create_run(
        &self,
        tenant_id: TenantId,
        input: CreateWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"
            INSERT INTO workflow_execution_runs (
                tenant_id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                started_at
            )
            VALUES ($1, $2, $3, $4, $5, 'running', 0, now())
            RETURNING
                id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                dead_letter_reason,
                started_at,
                finished_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(input.workflow_logical_name)
        .bind(input.trigger_type)
        .bind(input.trigger_entity_logical_name)
        .bind(input.trigger_payload)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to create workflow run for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        workflow_run_from_row(row)
    }

    async fn append_run_attempt(
        &self,
        tenant_id: TenantId,
        attempt: WorkflowRunAttempt,
    ) -> AppResult<()> {
        let run_id = uuid::Uuid::parse_str(attempt.run_id.as_str()).map_err(|error| {
            AppError::Validation(format!(
                "invalid workflow run id '{}': {error}",
                attempt.run_id
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO workflow_execution_attempts (
                run_id,
                tenant_id,
                attempt_number,
                status,
                error_message,
                executed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(run_id)
        .bind(tenant_id.as_uuid())
        .bind(attempt.attempt_number)
        .bind(attempt.status.as_str())
        .bind(attempt.error_message)
        .bind(attempt.executed_at)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to append workflow run attempt for run '{}' tenant '{}': {error}",
                attempt.run_id, tenant_id
            ))
        })?;

        Ok(())
    }

    async fn complete_run(
        &self,
        tenant_id: TenantId,
        input: CompleteWorkflowRunInput,
    ) -> AppResult<WorkflowRun> {
        let run_id = uuid::Uuid::parse_str(input.run_id.as_str()).map_err(|error| {
            AppError::Validation(format!(
                "invalid workflow run id '{}': {error}",
                input.run_id
            ))
        })?;

        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"
            UPDATE workflow_execution_runs
            SET
                status = $3,
                attempts = $4,
                dead_letter_reason = $5,
                finished_at = now()
            WHERE tenant_id = $1 AND id = $2
            RETURNING
                id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                dead_letter_reason,
                started_at,
                finished_at
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_id)
        .bind(input.status.as_str())
        .bind(input.attempts)
        .bind(input.dead_letter_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to complete workflow run '{}' for tenant '{}': {error}",
                run_id, tenant_id
            ))
        })?;

        workflow_run_from_row(row)
    }

    async fn list_runs(
        &self,
        tenant_id: TenantId,
        query: WorkflowRunListQuery,
    ) -> AppResult<Vec<WorkflowRun>> {
        let rows = sqlx::query_as::<_, WorkflowRunRow>(
            r#"
            SELECT
                id,
                workflow_logical_name,
                trigger_type,
                trigger_entity_logical_name,
                trigger_payload,
                status,
                attempts,
                dead_letter_reason,
                started_at,
                finished_at
            FROM workflow_execution_runs
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR workflow_logical_name = $2)
            ORDER BY started_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(query.workflow_logical_name)
        .bind(i64::try_from(query.limit).map_err(|error| {
            AppError::Validation(format!("invalid workflow run list limit: {error}"))
        })?)
        .bind(i64::try_from(query.offset).map_err(|error| {
            AppError::Validation(format!("invalid workflow run list offset: {error}"))
        })?)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list workflow runs for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter().map(workflow_run_from_row).collect()
    }

    async fn list_run_attempts(
        &self,
        tenant_id: TenantId,
        run_id: &str,
    ) -> AppResult<Vec<WorkflowRunAttempt>> {
        let run_uuid = uuid::Uuid::parse_str(run_id).map_err(|error| {
            AppError::Validation(format!("invalid workflow run id '{}': {error}", run_id))
        })?;

        let rows = sqlx::query_as::<_, WorkflowRunAttemptRow>(
            r#"
            SELECT run_id, attempt_number, status, error_message, executed_at
            FROM workflow_execution_attempts
            WHERE tenant_id = $1 AND run_id = $2
            ORDER BY attempt_number
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list workflow run attempts for run '{}' tenant '{}': {error}",
                run_id, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(workflow_run_attempt_from_row)
            .collect()
    }
}

fn workflow_definition_from_row(row: WorkflowDefinitionRow) -> AppResult<WorkflowDefinition> {
    WorkflowDefinition::new(
        row.logical_name,
        row.display_name,
        row.description,
        workflow_trigger_from_parts(
            row.trigger_type.as_str(),
            row.trigger_entity_logical_name.as_deref(),
        )?,
        workflow_action_from_parts(
            row.action_type.as_str(),
            row.action_entity_logical_name.as_deref(),
            row.action_payload,
        )?,
        workflow_steps_from_json(row.action_steps)?,
        u16::try_from(row.max_attempts).map_err(|error| {
            AppError::Validation(format!("invalid workflow max_attempts value: {error}"))
        })?,
        row.is_enabled,
    )
}

fn workflow_steps_to_json(steps: Option<&[WorkflowStep]>) -> AppResult<Option<Value>> {
    let Some(steps) = steps else {
        return Ok(None);
    };

    serde_json::to_value(steps).map(Some).map_err(|error| {
        AppError::Validation(format!(
            "failed to serialize workflow action_steps: {error}"
        ))
    })
}

fn workflow_steps_from_json(value: Option<Value>) -> AppResult<Option<Vec<WorkflowStep>>> {
    let Some(value) = value else {
        return Ok(None);
    };

    serde_json::from_value(value).map(Some).map_err(|error| {
        AppError::Validation(format!(
            "failed to deserialize workflow action_steps: {error}"
        ))
    })
}

fn workflow_trigger_parts(trigger: &WorkflowTrigger) -> (&'static str, Option<&str>) {
    match trigger {
        WorkflowTrigger::Manual => ("manual", None),
        WorkflowTrigger::RuntimeRecordCreated {
            entity_logical_name,
        } => ("runtime_record_created", Some(entity_logical_name.as_str())),
    }
}

fn workflow_action_parts(action: &WorkflowAction) -> (&'static str, Option<&str>, Value) {
    match action {
        WorkflowAction::LogMessage { message } => {
            ("log_message", None, serde_json::json!({"message": message}))
        }
        WorkflowAction::CreateRuntimeRecord {
            entity_logical_name,
            data,
        } => (
            "create_runtime_record",
            Some(entity_logical_name.as_str()),
            data.clone(),
        ),
    }
}

fn workflow_trigger_from_parts(
    trigger_type: &str,
    trigger_entity_logical_name: Option<&str>,
) -> AppResult<WorkflowTrigger> {
    match trigger_type {
        "manual" => Ok(WorkflowTrigger::Manual),
        "runtime_record_created" => {
            let entity_logical_name = trigger_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "runtime_record_created trigger requires trigger_entity_logical_name"
                        .to_owned(),
                )
            })?;

            Ok(WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: entity_logical_name.to_owned(),
            })
        }
        _ => Err(AppError::Validation(format!(
            "unknown workflow trigger_type '{trigger_type}'"
        ))),
    }
}

fn workflow_action_from_parts(
    action_type: &str,
    action_entity_logical_name: Option<&str>,
    action_payload: Value,
) -> AppResult<WorkflowAction> {
    match action_type {
        "log_message" => {
            let message = action_payload
                .as_object()
                .and_then(|payload| payload.get("message"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    AppError::Validation(
                        "log_message action payload requires string field 'message'".to_owned(),
                    )
                })?;

            Ok(WorkflowAction::LogMessage {
                message: message.to_owned(),
            })
        }
        "create_runtime_record" => {
            let entity_logical_name = action_entity_logical_name.ok_or_else(|| {
                AppError::Validation(
                    "create_runtime_record action requires action_entity_logical_name".to_owned(),
                )
            })?;

            Ok(WorkflowAction::CreateRuntimeRecord {
                entity_logical_name: entity_logical_name.to_owned(),
                data: action_payload,
            })
        }
        _ => Err(AppError::Validation(format!(
            "unknown workflow action_type '{action_type}'"
        ))),
    }
}

fn workflow_run_from_row(row: WorkflowRunRow) -> AppResult<WorkflowRun> {
    Ok(WorkflowRun {
        run_id: row.id.to_string(),
        workflow_logical_name: row.workflow_logical_name,
        trigger_type: row.trigger_type,
        trigger_entity_logical_name: row.trigger_entity_logical_name,
        trigger_payload: row.trigger_payload,
        status: WorkflowRunStatus::parse(row.status.as_str())?,
        attempts: row.attempts,
        dead_letter_reason: row.dead_letter_reason,
        started_at: row.started_at,
        finished_at: row.finished_at,
    })
}

fn workflow_run_attempt_from_row(row: WorkflowRunAttemptRow) -> AppResult<WorkflowRunAttempt> {
    Ok(WorkflowRunAttempt {
        run_id: row.run_id.to_string(),
        attempt_number: row.attempt_number,
        status: WorkflowRunAttemptStatus::parse(row.status.as_str())?,
        error_message: row.error_message,
        executed_at: row.executed_at,
    })
}
