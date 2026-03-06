use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::{Permission, UserId};

use crate::{
    AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService, RuntimeFieldGrant,
    TemporaryPermissionGrant, TenantAccessService, TenantMembership, TenantRepository, UserRecord,
    UserRepository,
};

#[derive(Default)]
struct FakeTenantRepository {
    memberships: Mutex<HashMap<String, Vec<TenantMembership>>>,
}

#[async_trait]
impl TenantRepository for FakeTenantRepository {
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>> {
        let memberships = self.memberships.lock().await;
        Ok(memberships.get(subject).and_then(|subject_memberships| {
            subject_memberships
                .first()
                .map(|membership| membership.tenant_id)
        }))
    }

    async fn registration_mode_for_tenant(
        &self,
        _tenant_id: TenantId,
    ) -> AppResult<qryvanta_domain::RegistrationMode> {
        Ok(qryvanta_domain::RegistrationMode::Open)
    }

    async fn create_membership(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _display_name: &str,
        _email: Option<&str>,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn ensure_membership_for_subject(
        &self,
        _subject: &str,
        _display_name: &str,
        _email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId> {
        Ok(preferred_tenant_id.unwrap_or_default())
    }

    async fn list_memberships_for_subject(
        &self,
        subject: &str,
    ) -> AppResult<Vec<TenantMembership>> {
        let memberships = self.memberships.lock().await;
        Ok(memberships.get(subject).cloned().unwrap_or_default())
    }

    async fn contact_record_for_subject(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
    ) -> AppResult<Option<String>> {
        Ok(None)
    }

    async fn save_contact_record_for_subject(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _contact_record_id: &str,
    ) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct FakeUserRepository {
    defaults: Mutex<HashMap<UserId, TenantId>>,
}

#[async_trait]
impl UserRepository for FakeUserRepository {
    async fn find_by_email(&self, _email: &str) -> AppResult<Option<UserRecord>> {
        Ok(None)
    }

    async fn find_by_id(&self, _user_id: UserId) -> AppResult<Option<UserRecord>> {
        Ok(None)
    }

    async fn create(
        &self,
        _email: &str,
        _password_hash: Option<&str>,
        _email_verified: bool,
    ) -> AppResult<UserId> {
        Ok(UserId::default())
    }

    async fn update_password(&self, _user_id: UserId, _password_hash: &str) -> AppResult<()> {
        Ok(())
    }

    async fn revoke_sessions(&self, _user_id: UserId) -> AppResult<()> {
        Ok(())
    }

    async fn default_tenant_id(&self, user_id: UserId) -> AppResult<Option<TenantId>> {
        Ok(self.defaults.lock().await.get(&user_id).copied())
    }

    async fn set_default_tenant_id(&self, user_id: UserId, tenant_id: TenantId) -> AppResult<()> {
        self.defaults.lock().await.insert(user_id, tenant_id);
        Ok(())
    }

    async fn record_failed_login(&self, _user_id: UserId) -> AppResult<()> {
        Ok(())
    }

    async fn reset_failed_logins(&self, _user_id: UserId) -> AppResult<()> {
        Ok(())
    }

    async fn mark_email_verified(&self, _user_id: UserId) -> AppResult<()> {
        Ok(())
    }

    async fn update_display_name(
        &self,
        _user_id: UserId,
        _tenant_id: TenantId,
        _display_name: &str,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn update_email(&self, _user_id: UserId, _new_email: &str) -> AppResult<()> {
        Ok(())
    }

    async fn enable_totp(
        &self,
        _user_id: UserId,
        _totp_secret_enc: &[u8],
        _recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn begin_totp_enrollment(
        &self,
        _user_id: UserId,
        _totp_secret_enc: &[u8],
        _recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn confirm_totp_enrollment(&self, _user_id: UserId) -> AppResult<()> {
        Ok(())
    }

    async fn disable_totp(&self, _user_id: UserId) -> AppResult<()> {
        Ok(())
    }

    async fn update_recovery_codes(
        &self,
        _user_id: UserId,
        _recovery_codes_hash: &serde_json::Value,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn find_by_subject(&self, _subject: &str) -> AppResult<Option<UserRecord>> {
        Ok(None)
    }
}

#[derive(Default)]
struct FakeAuthorizationRepository {
    permissions: HashMap<(TenantId, String), Vec<Permission>>,
}

#[async_trait]
impl AuthorizationRepository for FakeAuthorizationRepository {
    async fn list_permissions_for_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<Permission>> {
        Ok(self
            .permissions
            .get(&(tenant_id, subject.to_owned()))
            .cloned()
            .unwrap_or_default())
    }

    async fn list_runtime_field_grants_for_subject(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeFieldGrant>> {
        Ok(Vec::new())
    }

    async fn find_active_temporary_permission_grant(
        &self,
        _tenant_id: TenantId,
        _subject: &str,
        _permission: Permission,
    ) -> AppResult<Option<TemporaryPermissionGrant>> {
        Ok(None)
    }
}

#[derive(Default)]
struct NoopAuditRepository;

#[async_trait]
impl AuditRepository for NoopAuditRepository {
    async fn append_event(&self, _event: AuditEvent) -> AppResult<()> {
        Ok(())
    }
}

fn service_with_memberships(
    subject: &str,
    memberships: Vec<TenantMembership>,
    defaults: HashMap<UserId, TenantId>,
    permissions: HashMap<(TenantId, String), Vec<Permission>>,
) -> TenantAccessService {
    let tenant_repository = Arc::new(FakeTenantRepository {
        memberships: Mutex::new(HashMap::from([(subject.to_owned(), memberships)])),
    });
    let user_repository = Arc::new(FakeUserRepository {
        defaults: Mutex::new(defaults),
    });
    let authorization_service = AuthorizationService::new(
        Arc::new(FakeAuthorizationRepository { permissions }),
        Arc::new(NoopAuditRepository),
    );

    TenantAccessService::new(tenant_repository, user_repository, authorization_service)
}

#[tokio::test]
async fn resolve_active_tenant_prefers_persisted_default() {
    let subject = UserId::default().to_string();
    let first_tenant = TenantId::new();
    let second_tenant = TenantId::new();
    let user_id = UserId::from_uuid(uuid::Uuid::parse_str(subject.as_str()).unwrap_or_default());
    let service = service_with_memberships(
        subject.as_str(),
        vec![
            TenantMembership {
                tenant_id: first_tenant,
                tenant_name: "Alpha".to_owned(),
                display_name: "Alice".to_owned(),
                email: Some("alice@example.com".to_owned()),
            },
            TenantMembership {
                tenant_id: second_tenant,
                tenant_name: "Bravo".to_owned(),
                display_name: "Alice".to_owned(),
                email: Some("alice@example.com".to_owned()),
            },
        ],
        HashMap::from([(user_id, second_tenant)]),
        HashMap::from([
            (
                (first_tenant, subject.clone()),
                vec![Permission::MetadataFieldRead],
            ),
            (
                (second_tenant, subject.clone()),
                vec![Permission::SecurityRoleManage],
            ),
        ]),
    );

    let active = service
        .resolve_active_tenant(subject.as_str())
        .await
        .unwrap_or_else(|_| unreachable!())
        .unwrap_or_else(|| unreachable!());

    assert_eq!(active.tenant_id, second_tenant);
    assert!(active.is_default);
    assert_eq!(active.accessible_surfaces, vec!["admin".to_owned()]);
}

#[tokio::test]
async fn resolve_active_tenant_persists_deterministic_fallback_for_user_subjects() {
    let user_id = UserId::default();
    let subject = user_id.to_string();
    let tenant_id = TenantId::new();
    let service = service_with_memberships(
        subject.as_str(),
        vec![TenantMembership {
            tenant_id,
            tenant_name: "Alpha".to_owned(),
            display_name: "Alice".to_owned(),
            email: Some("alice@example.com".to_owned()),
        }],
        HashMap::new(),
        HashMap::from([(
            (tenant_id, subject.clone()),
            vec![Permission::MetadataFieldRead],
        )]),
    );

    let active = service
        .resolve_active_tenant(subject.as_str())
        .await
        .unwrap_or_else(|_| unreachable!())
        .unwrap_or_else(|| unreachable!());

    assert_eq!(active.tenant_id, tenant_id);
    assert!(active.is_default);

    let switched = service
        .list_subject_tenants(subject.as_str())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(switched[0].is_default);
}

#[tokio::test]
async fn switch_active_tenant_rejects_non_membership() {
    let subject = UserId::default().to_string();
    let service =
        service_with_memberships(subject.as_str(), Vec::new(), HashMap::new(), HashMap::new());

    let result = service
        .switch_active_tenant(subject.as_str(), TenantId::new())
        .await;
    assert!(matches!(result, Err(qryvanta_core::AppError::Forbidden(_))));
}
