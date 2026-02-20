use std::sync::Arc;

use qryvanta_application::{
    AuthEventService, AuthTokenService, AuthorizationService, MetadataService, MfaService,
    RateLimitService, SecurityAdminService, TenantRepository, UserService,
};
use qryvanta_core::TenantId;
use qryvanta_infrastructure::PostgresPasskeyRepository;
use webauthn_rs::Webauthn;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub metadata_service: MetadataService,
    pub security_admin_service: SecurityAdminService,
    pub authorization_service: AuthorizationService,
    pub auth_event_service: AuthEventService,
    pub user_service: UserService,
    pub auth_token_service: AuthTokenService,
    pub mfa_service: MfaService,
    pub rate_limit_service: RateLimitService,
    pub tenant_repository: Arc<dyn TenantRepository>,
    pub passkey_repository: PostgresPasskeyRepository,
    pub webauthn: Arc<Webauthn>,
    pub frontend_url: String,
    pub bootstrap_token: String,
    pub bootstrap_tenant_id: Option<TenantId>,
}
