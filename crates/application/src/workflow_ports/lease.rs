use async_trait::async_trait;
use qryvanta_core::AppResult;

use super::execution::WorkflowWorkerLease;

/// Distributed coordination port for worker lease claims.
#[async_trait]
pub trait WorkflowWorkerLeaseCoordinator: Send + Sync {
    /// Attempts to acquire one lease for the given scope.
    async fn try_acquire_lease(
        &self,
        scope_key: &str,
        holder_id: &str,
        lease_seconds: u32,
    ) -> AppResult<Option<WorkflowWorkerLease>>;

    /// Releases one lease using token compare-and-delete semantics.
    async fn release_lease(&self, lease: &WorkflowWorkerLease) -> AppResult<()>;

    /// Renews one existing lease and returns false when token ownership changed.
    async fn renew_lease(&self, lease: &WorkflowWorkerLease, lease_seconds: u32)
    -> AppResult<bool>;
}
