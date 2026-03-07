use qryvanta_application::WorkflowService;
use qryvanta_core::AppResult;
use qryvanta_domain::{WorkflowDefinition, WorkflowStep};
use tracing::{info, warn};

use crate::ClaimedWorkflowJobResponse;
use crate::config::WorkerLeaseLossStrategy;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct JobExecutionTotals {
    pub(crate) executed_jobs: u32,
    pub(crate) failed_jobs: u32,
    pub(crate) cancelled_due_to_lease_loss: bool,
}

type WorkerExecutionTaskResult = (
    String,
    String,
    String,
    AppResult<qryvanta_application::WorkflowRun>,
);

pub(crate) async fn execute_claimed_jobs(
    workflow_service: WorkflowService,
    worker_id: &str,
    claimed_jobs: Vec<ClaimedWorkflowJobResponse>,
    max_concurrency: usize,
    lease_loss_strategy: WorkerLeaseLossStrategy,
    mut cancel_signal: Option<tokio::sync::watch::Receiver<bool>>,
) -> JobExecutionTotals {
    let mut in_flight = tokio::task::JoinSet::new();
    let mut remaining_jobs = claimed_jobs.into_iter();
    let mut mutating_abort_handles: Vec<tokio::task::AbortHandle> = Vec::new();
    let worker_id = worker_id.to_owned();
    let max_concurrency = max_concurrency.max(1);
    let mut totals = JobExecutionTotals::default();
    let mut lease_loss_detected = false;

    loop {
        while !lease_loss_detected && in_flight.len() < max_concurrency {
            let Some(claimed_job) = remaining_jobs.next() else {
                break;
            };

            let queued_job = match claimed_job.try_into_claimed_job() {
                Ok(job) => job,
                Err(error) => {
                    totals.failed_jobs = totals.failed_jobs.saturating_add(1);
                    warn!(
                        worker_id = %worker_id,
                        error = %error,
                        "failed to parse claimed workflow job payload"
                    );
                    continue;
                }
            };

            let workflow_service = workflow_service.clone();
            let worker_id = worker_id.clone();
            let is_mutating = workflow_has_mutating_effects(&queued_job.workflow);
            let job_id = queued_job.job_id.clone();
            let run_id = queued_job.run_id.clone();
            let abort_handle = in_flight.spawn(async move {
                let result = workflow_service
                    .execute_claimed_job(worker_id.as_str(), queued_job)
                    .await;
                (worker_id, job_id, run_id, result)
            });

            if is_mutating {
                mutating_abort_handles.push(abort_handle);
            }
        }

        if lease_loss_detected && in_flight.is_empty() {
            break;
        }

        if !lease_loss_detected && cancellation_requested(cancel_signal.as_ref()) {
            lease_loss_detected = true;
            totals.cancelled_due_to_lease_loss = true;

            if matches!(lease_loss_strategy, WorkerLeaseLossStrategy::AbortAll) {
                cancel_in_flight_jobs(&mut in_flight, worker_id.as_str()).await;
                return totals;
            }

            abort_mutating_in_flight_jobs(&mut mutating_abort_handles, worker_id.as_str());
            continue;
        }

        let join_result = if let Some(cancel_signal) = cancel_signal.as_mut() {
            tokio::select! {
                changed = cancel_signal.changed() => {
                    if changed.is_ok() && *cancel_signal.borrow() {
                        lease_loss_detected = true;
                        totals.cancelled_due_to_lease_loss = true;

                        if matches!(lease_loss_strategy, WorkerLeaseLossStrategy::AbortAll) {
                            cancel_in_flight_jobs(&mut in_flight, worker_id.as_str()).await;
                            return totals;
                        }

                        abort_mutating_in_flight_jobs(&mut mutating_abort_handles, worker_id.as_str());
                    }
                    continue;
                }
                joined = in_flight.join_next() => joined,
            }
        } else {
            in_flight.join_next().await
        };

        let Some(join_result) = join_result else {
            break;
        };

        match join_result {
            Ok((worker_id, job_id, run_id, result)) => match result {
                Ok(run) => {
                    totals.executed_jobs = totals.executed_jobs.saturating_add(1);
                    info!(
                        worker_id = %worker_id,
                        job_id = %job_id,
                        run_id = %run_id,
                        status = %run.status.as_str(),
                        attempts = run.attempts,
                        "workflow job executed"
                    );
                }
                Err(error) => {
                    totals.failed_jobs = totals.failed_jobs.saturating_add(1);
                    warn!(
                        worker_id = %worker_id,
                        job_id = %job_id,
                        run_id = %run_id,
                        error = %error,
                        "workflow job execution failed"
                    );
                }
            },
            Err(error) => {
                totals.failed_jobs = totals.failed_jobs.saturating_add(1);
                warn!(
                    worker_id = %worker_id,
                    error = %error,
                    "workflow execution task join failed"
                );
            }
        }
    }

    totals
}

fn workflow_has_mutating_effects(workflow: &WorkflowDefinition) -> bool {
    workflow.steps().iter().any(step_is_mutating)
}

fn step_is_mutating(step: &WorkflowStep) -> bool {
    match step {
        WorkflowStep::LogMessage { .. } => false,
        WorkflowStep::CreateRuntimeRecord { .. }
        | WorkflowStep::UpdateRuntimeRecord { .. }
        | WorkflowStep::DeleteRuntimeRecord { .. }
        | WorkflowStep::SendEmail { .. }
        | WorkflowStep::HttpRequest { .. }
        | WorkflowStep::Webhook { .. }
        | WorkflowStep::AssignOwner { .. }
        | WorkflowStep::ApprovalRequest { .. } => true,
        WorkflowStep::Delay { .. } => false,
        WorkflowStep::Condition {
            then_steps,
            else_steps,
            ..
        } => then_steps.iter().any(step_is_mutating) || else_steps.iter().any(step_is_mutating),
    }
}

fn cancellation_requested(cancel_signal: Option<&tokio::sync::watch::Receiver<bool>>) -> bool {
    cancel_signal.is_some_and(|receiver| *receiver.borrow())
}

fn abort_mutating_in_flight_jobs(
    abort_handles: &mut Vec<tokio::task::AbortHandle>,
    worker_id: &str,
) {
    if abort_handles.is_empty() {
        return;
    }

    let mut aborted = 0_usize;
    for abort_handle in abort_handles.drain(..) {
        abort_handle.abort();
        aborted = aborted.saturating_add(1);
    }

    warn!(
        worker_id = %worker_id,
        aborted,
        "aborted mutating in-flight workflow tasks due to lease loss"
    );
}

async fn cancel_in_flight_jobs(
    worker_tasks: &mut tokio::task::JoinSet<WorkerExecutionTaskResult>,
    worker_id: &str,
) {
    if worker_tasks.is_empty() {
        return;
    }

    warn!(
        worker_id = %worker_id,
        in_flight = worker_tasks.len(),
        "cancelling in-flight workflow job tasks due to lease loss"
    );

    worker_tasks.abort_all();
    while worker_tasks.join_next().await.is_some() {}
}
