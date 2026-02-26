use super::*;

impl PostgresWorkflowRepository {
    pub(super) async fn enqueue_run_job_impl(
        &self,
        tenant_id: TenantId,
        run_id: &str,
    ) -> AppResult<()> {
        let run_uuid = uuid::Uuid::parse_str(run_id).map_err(|error| {
            AppError::Validation(format!("invalid workflow run id '{run_id}': {error}"))
        })?;

        sqlx::query(
            r#"
            INSERT INTO workflow_execution_jobs (
                tenant_id,
                run_id,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, 'pending', now(), now())
            ON CONFLICT (run_id)
            DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(run_uuid)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to enqueue workflow run '{run_id}' for tenant '{tenant_id}': {error}"
            ))
        })?;

        Ok(())
    }

    pub(super) async fn claim_jobs_impl(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        partition: Option<WorkflowClaimPartition>,
    ) -> AppResult<Vec<ClaimedWorkflowJob>> {
        let partition_count = partition
            .map(|value| {
                i32::try_from(value.partition_count()).map_err(|error| {
                    AppError::Validation(format!("invalid workflow partition_count value: {error}"))
                })
            })
            .transpose()?;
        let partition_index = partition
            .map(|value| {
                i32::try_from(value.partition_index()).map_err(|error| {
                    AppError::Validation(format!("invalid workflow partition_index value: {error}"))
                })
            })
            .transpose()?;

        let mut transaction = self.pool.begin().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to start workflow job claim transaction: {error}"
            ))
        })?;

        let claim_rows = sqlx::query_as::<_, ClaimedWorkflowJobRow>(
            r#"
            WITH candidate_jobs AS (
                SELECT id
                FROM workflow_execution_jobs
                WHERE (
                        status = 'pending'
                        OR (status = 'leased' AND lease_expires_at < now())
                      )
                  AND (
                        $4::INT IS NULL
                        OR mod(
                            (hashtext(tenant_id::text)::BIGINT & 2147483647),
                            $4::BIGINT
                        ) = $5::BIGINT
                      )
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            ),
            leased_jobs AS (
                UPDATE workflow_execution_jobs jobs
                SET
                    status = 'leased',
                    leased_by = $2,
                    lease_token = gen_random_uuid()::TEXT,
                    lease_expires_at = now() + make_interval(secs => $3::INT),
                    updated_at = now(),
                    last_error = NULL
                FROM candidate_jobs
                WHERE jobs.id = candidate_jobs.id
                RETURNING jobs.id, jobs.tenant_id, jobs.run_id, jobs.lease_token
            )
            SELECT
                leased_jobs.id AS job_id,
                leased_jobs.tenant_id,
                leased_jobs.run_id,
                leased_jobs.lease_token,
                runs.trigger_payload,
                workflows.logical_name,
                workflows.display_name,
                workflows.description,
                workflows.trigger_type,
                workflows.trigger_entity_logical_name,
                workflows.action_type,
                workflows.action_entity_logical_name,
                workflows.action_payload,
                workflows.action_steps,
                workflows.max_attempts,
                workflows.is_enabled
            FROM leased_jobs
            INNER JOIN workflow_execution_runs runs
                ON runs.id = leased_jobs.run_id
               AND runs.tenant_id = leased_jobs.tenant_id
            INNER JOIN workflow_definitions workflows
                ON workflows.tenant_id = runs.tenant_id
               AND workflows.logical_name = runs.workflow_logical_name
            ORDER BY runs.started_at ASC
            "#,
        )
        .bind(i64::try_from(limit).map_err(|error| {
            AppError::Validation(format!("invalid workflow claim limit: {error}"))
        })?)
        .bind(worker_id)
        .bind(i32::try_from(lease_seconds).map_err(|error| {
            AppError::Validation(format!("invalid workflow lease_seconds: {error}"))
        })?)
        .bind(partition_count)
        .bind(partition_index)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to claim workflow jobs for worker '{worker_id}': {error}"
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit workflow job claim transaction: {error}"
            ))
        })?;

        claim_rows
            .into_iter()
            .map(claimed_workflow_job_from_row)
            .collect()
    }

    pub(super) async fn complete_job_impl(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        let job_uuid = uuid::Uuid::parse_str(job_id).map_err(|error| {
            AppError::Validation(format!("invalid workflow job id '{job_id}': {error}"))
        })?;

        let result = sqlx::query(
            r#"
            UPDATE workflow_execution_jobs
            SET
                status = 'completed',
                leased_by = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                updated_at = now()
            WHERE tenant_id = $1
              AND id = $2
              AND leased_by = $3
              AND lease_token = $4
              AND status = 'leased'
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(job_uuid)
        .bind(worker_id)
        .bind(lease_token)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to complete workflow job '{job_id}' for tenant '{tenant_id}' worker '{worker_id}': {error}"
            ))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "workflow job '{job_id}' is not currently leased by worker '{worker_id}' with matching lease token"
            )));
        }

        Ok(())
    }

    pub(super) async fn fail_job_impl(
        &self,
        tenant_id: TenantId,
        job_id: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        let job_uuid = uuid::Uuid::parse_str(job_id).map_err(|error| {
            AppError::Validation(format!("invalid workflow job id '{job_id}': {error}"))
        })?;

        let result = sqlx::query(
            r#"
            UPDATE workflow_execution_jobs
            SET
                status = 'failed',
                leased_by = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                updated_at = now(),
                last_error = $5
            WHERE tenant_id = $1
              AND id = $2
              AND leased_by = $3
              AND lease_token = $4
              AND status = 'leased'
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(job_uuid)
        .bind(worker_id)
        .bind(lease_token)
        .bind(error_message)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to mark workflow job '{job_id}' as failed for tenant '{tenant_id}' worker '{worker_id}': {error}"
            ))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "workflow job '{job_id}' is not currently leased by worker '{worker_id}' with matching lease token"
            )));
        }

        Ok(())
    }

    pub(super) async fn upsert_worker_heartbeat_impl(
        &self,
        worker_id: &str,
        input: WorkflowWorkerHeartbeatInput,
    ) -> AppResult<()> {
        let partition_count = input
            .partition
            .map(|value| {
                i32::try_from(value.partition_count()).map_err(|error| {
                    AppError::Validation(format!(
                        "invalid worker heartbeat partition_count value: {error}"
                    ))
                })
            })
            .transpose()?;
        let partition_index = input
            .partition
            .map(|value| {
                i32::try_from(value.partition_index()).map_err(|error| {
                    AppError::Validation(format!(
                        "invalid worker heartbeat partition_index value: {error}"
                    ))
                })
            })
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO workflow_worker_heartbeats (
                worker_id,
                last_seen_at,
                last_claimed_jobs,
                last_executed_jobs,
                last_failed_jobs,
                partition_count,
                partition_index,
                updated_at
            )
            VALUES ($1, now(), $2, $3, $4, $5, $6, now())
            ON CONFLICT (worker_id)
            DO UPDATE SET
                last_seen_at = now(),
                last_claimed_jobs = EXCLUDED.last_claimed_jobs,
                last_executed_jobs = EXCLUDED.last_executed_jobs,
                last_failed_jobs = EXCLUDED.last_failed_jobs,
                partition_count = EXCLUDED.partition_count,
                partition_index = EXCLUDED.partition_index,
                updated_at = now()
            "#,
        )
        .bind(worker_id)
        .bind(i64::from(input.claimed_jobs))
        .bind(i64::from(input.executed_jobs))
        .bind(i64::from(input.failed_jobs))
        .bind(partition_count)
        .bind(partition_index)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to upsert workflow worker heartbeat for '{worker_id}': {error}"
            ))
        })?;

        Ok(())
    }

    pub(super) async fn queue_stats_impl(
        &self,
        query: WorkflowQueueStatsQuery,
    ) -> AppResult<WorkflowQueueStats> {
        let partition_count = query
            .partition
            .map(|value| {
                i32::try_from(value.partition_count()).map_err(|error| {
                    AppError::Validation(format!(
                        "invalid queue stats partition_count value: {error}"
                    ))
                })
            })
            .transpose()?;
        let partition_index = query
            .partition
            .map(|value| {
                i32::try_from(value.partition_index()).map_err(|error| {
                    AppError::Validation(format!(
                        "invalid queue stats partition_index value: {error}"
                    ))
                })
            })
            .transpose()?;

        let queue_stats = sqlx::query_as::<_, WorkflowQueueStatsRow>(
            r#"
            SELECT
                COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0) AS pending_jobs,
                COALESCE(SUM(CASE WHEN status = 'leased' THEN 1 ELSE 0 END), 0) AS leased_jobs,
                COALESCE(SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END), 0) AS completed_jobs,
                COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0) AS failed_jobs,
                COALESCE(
                    SUM(
                        CASE
                            WHEN status = 'leased' AND lease_expires_at < now() THEN 1
                            ELSE 0
                        END
                    ),
                    0
                ) AS expired_leases
            FROM workflow_execution_jobs
            WHERE (
                    $1::INT IS NULL
                    OR mod(
                        (hashtext(tenant_id::text)::BIGINT & 2147483647),
                        $1::BIGINT
                    ) = $2::BIGINT
                  )
            "#,
        )
        .bind(partition_count)
        .bind(partition_index)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to load workflow queue stats: {error}"))
        })?;

        let active_workers = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM workflow_worker_heartbeats
            WHERE last_seen_at >= now() - make_interval(secs => $1::INT)
              AND (
                    $2::INT IS NULL
                    OR (partition_count = $2 AND partition_index = $3)
                  )
            "#,
        )
        .bind(i32::try_from(query.active_window_seconds).map_err(|error| {
            AppError::Validation(format!("invalid active heartbeat window: {error}"))
        })?)
        .bind(partition_count)
        .bind(partition_index)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load workflow active worker stats: {error}"
            ))
        })?;

        Ok(WorkflowQueueStats {
            pending_jobs: queue_stats.pending_jobs,
            leased_jobs: queue_stats.leased_jobs,
            completed_jobs: queue_stats.completed_jobs,
            failed_jobs: queue_stats.failed_jobs,
            expired_leases: queue_stats.expired_leases,
            active_workers,
        })
    }
}
