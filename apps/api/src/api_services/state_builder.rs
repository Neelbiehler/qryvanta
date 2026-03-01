use std::sync::Arc;

use qryvanta_application::{AppService, ContactBootstrapService, MetadataService, WorkflowService};
use qryvanta_core::AppError;
use sqlx::PgPool;

use crate::api_config::ApiConfig;
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
    let security_services = security::build_security_services(&repositories);
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

    let app_runtime_service = Arc::new(metadata_service.clone());
    let workflow_runtime_service = Arc::new(metadata_service.clone());

    Ok(AppState {
        app_service: AppService::new(
            security_services.authorization_service.clone(),
            repositories.app_repository,
            app_runtime_service,
            repositories.audit_repository.clone(),
        ),
        metadata_service: metadata_service.clone(),
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
        bootstrap_token: config.bootstrap_token.clone(),
        bootstrap_tenant_id: config.bootstrap_tenant_id,
        worker_shared_secret: config.worker_shared_secret.clone(),
        workflow_worker_default_lease_seconds: config.workflow_worker_default_lease_seconds,
        workflow_worker_max_claim_limit: config.workflow_worker_max_claim_limit,
        workflow_worker_max_partition_count: config.workflow_worker_max_partition_count,
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
