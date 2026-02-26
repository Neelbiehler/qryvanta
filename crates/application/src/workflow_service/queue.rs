use super::*;

impl WorkflowService {
    /// Claims queued workflow jobs for one worker.
    pub async fn claim_jobs_for_worker(
        &self,
        worker_id: &str,
        limit: usize,
        lease_seconds: u32,
        partition: Option<WorkflowClaimPartition>,
    ) -> AppResult<Vec<ClaimedWorkflowJob>> {
        if self.execution_mode != WorkflowExecutionMode::Queued {
            return Err(AppError::Conflict(
                "queued workflow execution mode is not enabled".to_owned(),
            ));
        }

        if worker_id.trim().is_empty() {
            return Err(AppError::Validation(
                "worker_id must not be empty".to_owned(),
            ));
        }

        if limit == 0 {
            return Err(AppError::Validation(
                "limit must be greater than zero".to_owned(),
            ));
        }

        if lease_seconds == 0 {
            return Err(AppError::Validation(
                "lease_seconds must be greater than zero".to_owned(),
            ));
        }

        self.repository
            .claim_jobs(worker_id, limit, lease_seconds, partition)
            .await
    }

    /// Executes one claimed queued job and finalizes queue state.
    pub async fn execute_claimed_job(
        &self,
        worker_id: &str,
        job: ClaimedWorkflowJob,
    ) -> AppResult<WorkflowRun> {
        if self.execution_mode != WorkflowExecutionMode::Queued {
            return Err(AppError::Conflict(
                "queued workflow execution mode is not enabled".to_owned(),
            ));
        }

        if worker_id.trim().is_empty() {
            return Err(AppError::Validation(
                "worker_id must not be empty".to_owned(),
            ));
        }

        if job.lease_token.trim().is_empty() {
            return Err(AppError::Validation(
                "claimed workflow job lease_token must not be empty".to_owned(),
            ));
        }

        let job_id = job.job_id.clone();
        let tenant_id = job.tenant_id;
        let lease_token = job.lease_token.clone();
        let actor = UserIdentity::new(
            format!("workflow-worker:{worker_id}"),
            "Workflow Worker",
            None,
            tenant_id,
        );

        let run_result = self
            .execute_existing_run(
                &actor,
                &job.workflow,
                job.run_id.as_str(),
                job.trigger_payload,
            )
            .await;

        match run_result {
            Ok(run) => {
                self.repository
                    .complete_job(tenant_id, job_id.as_str(), worker_id, lease_token.as_str())
                    .await?;
                Ok(run)
            }
            Err(error) => {
                let error_message = error.to_string();
                if let Err(mark_error) = self
                    .repository
                    .fail_job(
                        tenant_id,
                        job_id.as_str(),
                        worker_id,
                        lease_token.as_str(),
                        error_message.as_str(),
                    )
                    .await
                {
                    return Err(AppError::Internal(format!(
                        "failed to execute claimed workflow job '{job_id}': {error}; additionally failed to mark queue job failed: {mark_error}"
                    )));
                }

                Err(error)
            }
        }
    }

    /// Stores one worker heartbeat snapshot for queue observability.
    pub async fn heartbeat_worker(
        &self,
        worker_id: &str,
        input: WorkflowWorkerHeartbeatInput,
    ) -> AppResult<()> {
        if self.execution_mode != WorkflowExecutionMode::Queued {
            return Err(AppError::Conflict(
                "queued workflow execution mode is not enabled".to_owned(),
            ));
        }

        if worker_id.trim().is_empty() {
            return Err(AppError::Validation(
                "worker_id must not be empty".to_owned(),
            ));
        }

        self.repository
            .upsert_worker_heartbeat(worker_id, input)
            .await
    }

    /// Returns queue and worker heartbeat stats for operations.
    pub async fn queue_stats(&self, active_window_seconds: u32) -> AppResult<WorkflowQueueStats> {
        self.queue_stats_with_partition(active_window_seconds, None)
            .await
    }

    /// Returns queue and worker heartbeat stats for one optional partition.
    pub async fn queue_stats_with_partition(
        &self,
        active_window_seconds: u32,
        partition: Option<WorkflowClaimPartition>,
    ) -> AppResult<WorkflowQueueStats> {
        if self.execution_mode != WorkflowExecutionMode::Queued {
            return Err(AppError::Conflict(
                "queued workflow execution mode is not enabled".to_owned(),
            ));
        }

        if active_window_seconds == 0 {
            return Err(AppError::Validation(
                "active_window_seconds must be greater than zero".to_owned(),
            ));
        }

        let query = WorkflowQueueStatsQuery {
            active_window_seconds,
            partition,
        };

        if self.queue_stats_cache_ttl_seconds > 0
            && let Some(cache) = &self.queue_stats_cache
            && let Some(stats) = cache.get_queue_stats(query).await?
        {
            return Ok(stats);
        }

        let stats = self.repository.queue_stats(query).await?;

        if self.queue_stats_cache_ttl_seconds > 0
            && let Some(cache) = &self.queue_stats_cache
        {
            cache
                .set_queue_stats(query, stats, self.queue_stats_cache_ttl_seconds)
                .await?;
        }

        Ok(stats)
    }
}
