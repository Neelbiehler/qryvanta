use qryvanta_application::{AuthEventService, AuthorizationService, SecurityAdminService};

use crate::api_config::ApiConfig;

use super::repositories::RepositorySet;

pub(super) struct SecurityServices {
    pub(super) authorization_service: AuthorizationService,
    pub(super) security_admin_service: SecurityAdminService,
    pub(super) auth_event_service: AuthEventService,
}

pub(super) fn build_security_services(
    repositories: &RepositorySet,
    config: &ApiConfig,
) -> SecurityServices {
    let authorization_service = AuthorizationService::new(
        repositories.authorization_repository.clone(),
        repositories.audit_repository.clone(),
    );

    let security_admin_service = SecurityAdminService::new(
        authorization_service.clone(),
        repositories.security_admin_repository.clone(),
        repositories.audit_log_repository.clone(),
        repositories.audit_repository.clone(),
    )
    .with_audit_immutable_mode(config.audit_immutable_mode);

    let auth_event_service = AuthEventService::new(repositories.auth_event_repository.clone());

    SecurityServices {
        authorization_service,
        security_admin_service,
        auth_event_service,
    }
}
