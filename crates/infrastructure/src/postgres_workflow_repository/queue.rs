use super::*;

impl PostgresWorkflowRepository {
    pub(super) async fn list_enabled_schedule_triggers_impl(
        &self,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<Vec<WorkflowScheduledTrigger>> {
        let mut transaction = begin_workflow_worker_transaction(&self.pool).await?;

        let rows = sqlx::query_as::<_, WorkflowScheduledTriggerRow>(
            r#"
            SELECT DISTINCT
                definitions.tenant_id,
                versions.trigger_entity_logical_name AS schedule_key
            FROM workflow_definitions definitions
            INNER JOIN workflow_published_versions versions
                ON versions.tenant_id = definitions.tenant_id
               AND versions.logical_name = definitions.logical_name
               AND versions.version = definitions.current_published_version
            WHERE definitions.lifecycle_state = 'published'
              AND versions.trigger_type = 'schedule_tick'
              AND versions.trigger_entity_logical_name IS NOT NULL
              AND ($1::UUID IS NULL OR definitions.tenant_id = $1)
            ORDER BY definitions.tenant_id, schedule_key
            "#,
        )
        .bind(tenant_filter.map(|value| value.as_uuid()))
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list enabled workflow schedule triggers: {error}"
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit workflow schedule trigger list transaction: {error}"
            ))
        })?;

        rows.into_iter()
            .map(workflow_scheduled_trigger_from_row)
            .collect()
    }

    pub(super) async fn claim_schedule_tick_impl(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        scheduled_for: chrono::DateTime<chrono::Utc>,
        worker_id: &str,
        lease_seconds: u32,
    ) -> AppResult<Option<ClaimedWorkflowScheduleTick>> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

        sqlx::query(
            r#"
            INSERT INTO workflow_schedule_ticks (
                tenant_id,
                schedule_key,
                slot_key,
                scheduled_for,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, 'pending', now(), now())
            ON CONFLICT (tenant_id, schedule_key, slot_key)
            DO NOTHING
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(schedule_key)
        .bind(slot_key)
        .bind(scheduled_for)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to enqueue workflow schedule tick '{schedule_key}/{slot_key}' for tenant '{tenant_id}': {error}"
            ))
        })?;

        let row = sqlx::query_as::<_, ClaimedWorkflowScheduleTickRow>(
            r#"
            UPDATE workflow_schedule_ticks
            SET
                status = 'leased',
                leased_by = $4,
                lease_token = gen_random_uuid()::TEXT,
                lease_expires_at = now() + make_interval(secs => $5::INT),
                last_error = NULL,
                updated_at = now()
            WHERE tenant_id = $1
              AND schedule_key = $2
              AND slot_key = $3
              AND (
                    status = 'pending'
                    OR (status = 'leased' AND lease_expires_at < now())
                  )
            RETURNING tenant_id, schedule_key, slot_key, scheduled_for, leased_by, lease_token
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(schedule_key)
        .bind(slot_key)
        .bind(worker_id)
        .bind(i32::try_from(lease_seconds).map_err(|error| {
            AppError::Validation(format!("invalid workflow schedule lease_seconds: {error}"))
        })?)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to claim workflow schedule tick '{schedule_key}/{slot_key}' for tenant '{tenant_id}': {error}"
            ))
        })?;

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit workflow schedule tick claim transaction: {error}"
            ))
        })?;

        row.map(claimed_workflow_schedule_tick_from_row).transpose()
    }

    pub(super) async fn complete_schedule_tick_impl(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
    ) -> AppResult<()> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

        let result = sqlx::query(
            r#"
            UPDATE workflow_schedule_ticks
            SET
                status = 'completed',
                leased_by = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                updated_at = now()
            WHERE tenant_id = $1
              AND schedule_key = $2
              AND slot_key = $3
              AND leased_by = $4
              AND lease_token = $5
              AND status = 'leased'
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(schedule_key)
        .bind(slot_key)
        .bind(worker_id)
        .bind(lease_token)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to complete workflow schedule tick '{schedule_key}/{slot_key}' for tenant '{tenant_id}': {error}"
            ))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "workflow schedule tick '{schedule_key}/{slot_key}' is not leased by worker '{worker_id}' with matching lease token"
            )));
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit workflow schedule tick completion transaction: {error}"
            ))
        })?;

        Ok(())
    }

    pub(super) async fn release_schedule_tick_impl(
        &self,
        tenant_id: TenantId,
        schedule_key: &str,
        slot_key: &str,
        worker_id: &str,
        lease_token: &str,
        error_message: &str,
    ) -> AppResult<()> {
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

        let result = sqlx::query(
            r#"
            UPDATE workflow_schedule_ticks
            SET
                status = 'pending',
                leased_by = NULL,
                lease_token = NULL,
                lease_expires_at = NULL,
                last_error = $6,
                updated_at = now()
            WHERE tenant_id = $1
              AND schedule_key = $2
              AND slot_key = $3
              AND leased_by = $4
              AND lease_token = $5
              AND status = 'leased'
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(schedule_key)
        .bind(slot_key)
        .bind(worker_id)
        .bind(lease_token)
        .bind(error_message)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to release workflow schedule tick '{schedule_key}/{slot_key}' for tenant '{tenant_id}': {error}"
            ))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "workflow schedule tick '{schedule_key}/{slot_key}' is not leased by worker '{worker_id}' with matching lease token"
            )));
        }

        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit workflow schedule tick release transaction: {error}"
            ))
        })?;

        Ok(())
    }

    pub(super) async fn enqueue_run_job_impl(
        &self,
        tenant_id: TenantId,
        run_id: &str,
    ) -> AppResult<()> {
        let run_uuid = uuid::Uuid::parse_str(run_id).map_err(|error| {
            AppError::Validation(format!("invalid workflow run id '{run_id}': {error}"))
        })?;
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

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
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to enqueue workflow run '{run_id}' for tenant '{tenant_id}': {error}"
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow enqueue transaction: {error}"
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
        tenant_filter: Option<TenantId>,
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

        let mut transaction = begin_workflow_worker_transaction(&self.pool).await?;

        let claim_rows = sqlx::query_as::<_, ClaimedWorkflowJobRow>(
            r#"
            WITH candidate_jobs AS (
                SELECT id
                FROM workflow_execution_jobs
                WHERE (
                        status = 'pending'
                        OR (status = 'leased' AND lease_expires_at < now())
                      )
                  AND ($6::UUID IS NULL OR tenant_id = $6)
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
                runs.workflow_version,
                leased_jobs.lease_token,
                runs.trigger_payload,
                versions.logical_name,
                versions.display_name,
                versions.description,
                versions.trigger_type,
                versions.trigger_entity_logical_name,
                versions.steps,
                versions.max_attempts,
                definitions.lifecycle_state,
                definitions.current_published_version
            FROM leased_jobs
            INNER JOIN workflow_execution_runs runs
                ON runs.id = leased_jobs.run_id
               AND runs.tenant_id = leased_jobs.tenant_id
            INNER JOIN workflow_definitions definitions
                ON definitions.tenant_id = runs.tenant_id
               AND definitions.logical_name = runs.workflow_logical_name
            INNER JOIN workflow_published_versions versions
                ON versions.tenant_id = runs.tenant_id
               AND versions.logical_name = runs.workflow_logical_name
               AND versions.version = runs.workflow_version
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
        .bind(tenant_filter.map(|value| value.as_uuid()))
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
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

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
        .execute(&mut *transaction)
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
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow job completion transaction: {error}"
            ))
        })?;

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
        let mut transaction = begin_tenant_transaction(&self.pool, tenant_id).await?;

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
        .execute(&mut *transaction)
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
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit tenant-scoped workflow job failure transaction: {error}"
            ))
        })?;

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
        let mut transaction = begin_workflow_worker_transaction(&self.pool).await?;
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
        .fetch_one(&mut *transaction)
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
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load workflow active worker stats: {error}"
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit workflow queue stats transaction: {error}"
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

fn workflow_scheduled_trigger_from_row(
    row: WorkflowScheduledTriggerRow,
) -> AppResult<WorkflowScheduledTrigger> {
    Ok(WorkflowScheduledTrigger {
        tenant_id: TenantId::from_uuid(row.tenant_id),
        schedule_key: row.schedule_key,
    })
}

fn claimed_workflow_schedule_tick_from_row(
    row: ClaimedWorkflowScheduleTickRow,
) -> AppResult<ClaimedWorkflowScheduleTick> {
    Ok(ClaimedWorkflowScheduleTick {
        tenant_id: TenantId::from_uuid(row.tenant_id),
        schedule_key: row.schedule_key,
        slot_key: row.slot_key,
        scheduled_for: row.scheduled_for,
        worker_id: row.leased_by,
        lease_token: row.lease_token,
    })
}
