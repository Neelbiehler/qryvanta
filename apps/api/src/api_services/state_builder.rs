use std::sync::Arc;

use qryvanta_application::{
    AppService, ContactBootstrapService, ExtensionService, MetadataService, WorkflowService,
};
use qryvanta_core::AppError;
use qryvanta_infrastructure::{HttpWorkflowActionDispatcher, WasmExtensionRuntime};
use sqlx::PgPool;
use tokio::sync::Semaphore;

use crate::api_config::ApiConfig;
use crate::observability::ApiObservabilityMetrics;
use crate::state::AppState;

use super::redis::build_redis_client;

mod caches;
mod repositories;
mod security;
mod users;
mod webauthn;

pub fn build_app_state(pool: PgPool, config: &ApiConfig) -> Result<AppState, AppError> {
    let redis_client = config
        .redis_url
        .as_deref()
        .map(build_redis_client)
        .transpose()?;

    let repositories = repositories::build_repository_set(&pool);
    let security_services = security::build_security_services(&repositories, config);
    let user_services = users::build_user_services(
        &pool,
        config,
        repositories.tenant_repository.clone(),
        repositories.user_repository.clone(),
        security_services.auth_event_service.clone(),
    )?;
    let workflow_queue_stats_cache =
        caches::build_workflow_queue_stats_cache(config, redis_client.clone())?;
    let rate_limit_service = caches::build_rate_limit_service(&pool, config, redis_client.clone())?;
    let webauthn = webauthn::build_webauthn(config)?;

    let metadata_service = MetadataService::new(
        repositories.metadata_repository.clone(),
        security_services.authorization_service.clone(),
        repositories.audit_repository.clone(),
    );
    let extension_service = ExtensionService::new(
        security_services.authorization_service.clone(),
        repositories.extension_repository.clone(),
        Arc::new(WasmExtensionRuntime::new()),
    );

    let app_runtime_service = Arc::new(metadata_service.clone());
    let workflow_runtime_service = Arc::new(metadata_service.clone());
    let workflow_email_service = super::email::build_email_service(config)?;
    let workflow_action_dispatcher = Arc::new(HttpWorkflowActionDispatcher::new(
        reqwest::Client::new(),
        workflow_email_service,
        3,
        250,
    ));

    Ok(AppState {
        app_service: AppService::new(
            security_services.authorization_service.clone(),
            repositories.app_repository,
            app_runtime_service,
            repositories.audit_repository.clone(),
        ),
        metadata_service: metadata_service.clone(),
        extension_service,
        contact_bootstrap_service: ContactBootstrapService::new(
            repositories.metadata_repository.clone(),
            repositories.tenant_repository.clone(),
        ),
        security_admin_service: security_services.security_admin_service,
        authorization_service: security_services.authorization_service.clone(),
        auth_event_service: security_services.auth_event_service,
        user_service: user_services.user_service,
        auth_token_service: user_services.auth_token_service,
        workflow_service: WorkflowService::new(
            security_services.authorization_service,
            repositories.workflow_repository,
            workflow_runtime_service,
            repositories.audit_repository.clone(),
            config.workflow_execution_mode,
        )
        .with_action_dispatcher(workflow_action_dispatcher)
        .with_queue_stats_cache(
            workflow_queue_stats_cache,
            config.workflow_queue_stats_cache_ttl_seconds,
        ),
        mfa_service: user_services.mfa_service,
        rate_limit_service,
        tenant_repository: repositories.tenant_repository,
        passkey_repository: repositories.passkey_repository,
        webauthn,
        frontend_url: config.frontend_url.clone(),
        physical_isolation_mode: config.physical_isolation_mode,
        physical_isolation_tenant_id: config.physical_isolation_tenant_id,
        bootstrap_token: config.bootstrap_token.clone(),
        bootstrap_tenant_id: config.bootstrap_tenant_id,
        worker_shared_secret: config.worker_shared_secret.clone(),
        workflow_worker_default_lease_seconds: config.workflow_worker_default_lease_seconds,
        workflow_worker_max_claim_limit: config.workflow_worker_max_claim_limit,
        workflow_worker_max_partition_count: config.workflow_worker_max_partition_count,
        runtime_query_max_limit: config.runtime_query_max_limit,
        runtime_query_backpressure: Arc::new(Semaphore::new(config.runtime_query_max_in_flight)),
        workflow_burst_backpressure: Arc::new(Semaphore::new(config.workflow_burst_max_in_flight)),
        slow_request_threshold_ms: config.slow_request_threshold_ms,
        slow_query_threshold_ms: config.slow_query_threshold_ms,
        observability_metrics: Arc::new(ApiObservabilityMetrics::default()),
        postgres_pool: pool,
        redis_client,
        redis_required: config.requires_redis(),
        qrywell_api_base_url: config.qrywell_api_base_url.clone(),
        qrywell_api_key: config.qrywell_api_key.clone(),
        qrywell_sync_poll_interval_ms: config.qrywell_sync_poll_interval_ms,
        qrywell_sync_batch_size: config.qrywell_sync_batch_size,
        qrywell_sync_max_attempts: config.qrywell_sync_max_attempts,
        http_client: reqwest::Client::new(),
    })
}
