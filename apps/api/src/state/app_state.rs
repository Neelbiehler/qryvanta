use std::sync::Arc;

use ipnet::IpNet;
use qryvanta_application::{
    AppService, AuthEventService, AuthTokenService, AuthorizationService, ContactBootstrapService,
    ExtensionService, MetadataService, MfaService, RateLimitService, SecurityAdminService,
    TenantAccessService, TenantRepository, UserService, WorkflowService,
};
use qryvanta_core::{AppError, TenantId};
use qryvanta_infrastructure::PostgresPasskeyRepository;
use sqlx::PgPool;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};
use webauthn_rs::Webauthn;

use crate::api_config::PhysicalIsolationMode;
use crate::observability::ApiObservabilityMetrics;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub app_service: AppService,
    pub metadata_service: MetadataService,
    pub extension_service: ExtensionService,
    pub contact_bootstrap_service: ContactBootstrapService,
    pub security_admin_service: SecurityAdminService,
    pub authorization_service: AuthorizationService,
    pub auth_event_service: AuthEventService,
    pub user_service: UserService,
    pub tenant_access_service: TenantAccessService,
    pub auth_token_service: AuthTokenService,
    pub workflow_service: WorkflowService,
    pub mfa_service: MfaService,
    pub rate_limit_service: RateLimitService,
    pub tenant_repository: Arc<dyn TenantRepository>,
    pub passkey_repository: PostgresPasskeyRepository,
    pub webauthn: Arc<Webauthn>,
    pub frontend_url: String,
    pub trust_proxy_headers: bool,
    pub trusted_proxy_cidrs: Vec<IpNet>,
    pub physical_isolation_mode: PhysicalIsolationMode,
    pub physical_isolation_tenant_id: Option<TenantId>,
    pub bootstrap_token: String,
    pub bootstrap_tenant_id: Option<TenantId>,
    pub worker_shared_secret: Option<String>,
    pub workflow_worker_default_lease_seconds: u32,
    pub workflow_worker_max_claim_limit: usize,
    pub workflow_worker_max_partition_count: u32,
    pub runtime_query_max_limit: usize,
    pub runtime_query_backpressure: Arc<Semaphore>,
    pub workflow_burst_backpressure: Arc<Semaphore>,
    pub slow_request_threshold_ms: u64,
    pub slow_query_threshold_ms: u64,
    pub observability_metrics: Arc<ApiObservabilityMetrics>,
    pub postgres_pool: PgPool,
    pub redis_client: Option<redis::Client>,
    pub redis_required: bool,
    pub qrywell_api_base_url: Option<String>,
    pub qrywell_api_key: Option<String>,
    pub qrywell_sync_poll_interval_ms: u64,
    pub qrywell_sync_batch_size: usize,
    pub qrywell_sync_max_attempts: i32,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn try_acquire_runtime_query_permit(&self) -> Result<OwnedSemaphorePermit, AppError> {
        match try_acquire_backpressure_permit(
            self.runtime_query_backpressure.clone(),
            "runtime query",
        ) {
            Err(AppError::RateLimited(message)) => {
                self.observability_metrics
                    .on_runtime_query_backpressure_rejection();
                Err(AppError::RateLimited(message))
            }
            result => result,
        }
    }

    pub fn try_acquire_workflow_burst_permit(&self) -> Result<OwnedSemaphorePermit, AppError> {
        match try_acquire_backpressure_permit(
            self.workflow_burst_backpressure.clone(),
            "workflow execution burst",
        ) {
            Err(AppError::RateLimited(message)) => {
                self.observability_metrics
                    .on_workflow_burst_backpressure_rejection();
                Err(AppError::RateLimited(message))
            }
            result => result,
        }
    }
}

fn try_acquire_backpressure_permit(
    semaphore: Arc<Semaphore>,
    control_name: &str,
) -> Result<OwnedSemaphorePermit, AppError> {
    match semaphore.try_acquire_owned() {
        Ok(permit) => Ok(permit),
        Err(TryAcquireError::NoPermits) => Err(AppError::RateLimited(format!(
            "{control_name} backpressure active: retry after in-flight workload drains"
        ))),
        Err(TryAcquireError::Closed) => Err(AppError::Internal(format!(
            "{control_name} backpressure semaphore closed unexpectedly"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backpressure_permit_rejects_when_pool_is_full() {
        let semaphore = Arc::new(Semaphore::new(1));
        let permit = try_acquire_backpressure_permit(semaphore.clone(), "runtime query")
            .unwrap_or_else(|_| unreachable!());

        let blocked = try_acquire_backpressure_permit(semaphore.clone(), "runtime query");
        assert!(matches!(blocked, Err(AppError::RateLimited(_))));

        drop(permit);
        assert!(try_acquire_backpressure_permit(semaphore, "runtime query").is_ok());
    }
}
