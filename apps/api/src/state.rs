use std::sync::Arc;

use qryvanta_application::{
    AppService, AuthEventService, AuthTokenService, AuthorizationService, ContactBootstrapService,
    MetadataService, MfaService, RateLimitService, SecurityAdminService, TenantRepository,
    UserService, WorkflowService,
};
use qryvanta_core::TenantId;
use qryvanta_infrastructure::PostgresPasskeyRepository;
use webauthn_rs::Webauthn;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub app_service: AppService,
    pub metadata_service: MetadataService,
    pub contact_bootstrap_service: ContactBootstrapService,
    pub security_admin_service: SecurityAdminService,
    pub authorization_service: AuthorizationService,
    pub auth_event_service: AuthEventService,
    pub user_service: UserService,
    pub auth_token_service: AuthTokenService,
    pub workflow_service: WorkflowService,
    pub mfa_service: MfaService,
    pub rate_limit_service: RateLimitService,
    pub tenant_repository: Arc<dyn TenantRepository>,
    pub passkey_repository: PostgresPasskeyRepository,
    pub webauthn: Arc<Webauthn>,
    pub frontend_url: String,
    pub bootstrap_token: String,
    pub bootstrap_tenant_id: Option<TenantId>,
    pub worker_shared_secret: Option<String>,
    pub workflow_worker_default_lease_seconds: u32,
    pub workflow_worker_max_claim_limit: usize,
}
