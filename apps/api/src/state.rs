use std::sync::Arc;

use qryvanta_application::{MetadataService, TenantRepository};
use qryvanta_core::TenantId;
use qryvanta_infrastructure::PostgresPasskeyRepository;
use webauthn_rs::Webauthn;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub metadata_service: MetadataService,
    pub tenant_repository: Arc<dyn TenantRepository>,
    pub passkey_repository: PostgresPasskeyRepository,
    pub webauthn: Arc<Webauthn>,
    pub frontend_url: String,
    pub bootstrap_token: String,
    pub bootstrap_tenant_id: Option<TenantId>,
}
