use std::sync::Arc;

use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId, UserIdentity};
use qryvanta_domain::{AuditAction, EntityDefinition, Permission, RegistrationMode};

use crate::AuthorizationService;

/// Repository port for metadata persistence.
#[async_trait]
pub trait MetadataRepository: Send + Sync {
    /// Saves an entity definition.
    async fn save_entity(&self, tenant_id: TenantId, entity: EntityDefinition) -> AppResult<()>;

    /// Lists all entity definitions.
    async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>>;
}

/// Repository port for append-only audit events.
#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Appends a single audit event.
    async fn append_event(&self, event: AuditEvent) -> AppResult<()>;
}

/// Canonical audit event payload emitted by application use-cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    /// Tenant partition key for the event.
    pub tenant_id: TenantId,
    /// Subject that performed the action.
    pub subject: String,
    /// Stable action identifier.
    pub action: AuditAction,
    /// Resource kind targeted by the action.
    pub resource_type: String,
    /// Stable resource identifier.
    pub resource_id: String,
    /// Optional human-readable detail payload.
    pub detail: Option<String>,
}

/// Repository port for subject-to-tenant resolution.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Finds the tenant associated with the provided subject claim.
    async fn find_tenant_for_subject(&self, subject: &str) -> AppResult<Option<TenantId>>;

    /// Returns the active registration mode for a tenant.
    async fn registration_mode_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<RegistrationMode>;

    /// Adds a membership for the subject inside a tenant.
    async fn create_membership(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<()>;

    /// Ensures the subject can be resolved to a tenant membership and returns that tenant.
    async fn ensure_membership_for_subject(
        &self,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
        preferred_tenant_id: Option<TenantId>,
    ) -> AppResult<TenantId>;
}

/// Application service for metadata operations.
#[derive(Clone)]
pub struct MetadataService {
    repository: Arc<dyn MetadataRepository>,
    authorization_service: AuthorizationService,
    audit_repository: Arc<dyn AuditRepository>,
}

impl MetadataService {
    /// Creates a new metadata service from a repository implementation.
    #[must_use]
    pub fn new(
        repository: Arc<dyn MetadataRepository>,
        authorization_service: AuthorizationService,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            repository,
            authorization_service,
            audit_repository,
        }
    }

    /// Registers a new entity definition.
    pub async fn register_entity(
        &self,
        actor: &UserIdentity,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> AppResult<EntityDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataEntityCreate,
            )
            .await?;

        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        let entity = EntityDefinition::new(logical_name, display_name)?;
        self.repository
            .save_entity(actor.tenant_id(), entity.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataEntityCreated,
                resource_type: "entity_definition".to_owned(),
                resource_id: entity.logical_name().as_str().to_owned(),
                detail: Some(format!(
                    "created metadata entity '{}'",
                    entity.logical_name().as_str()
                )),
            })
            .await?;

        Ok(entity)
    }

    /// Returns every known entity definition.
    pub async fn list_entities(&self, actor: &UserIdentity) -> AppResult<Vec<EntityDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataEntityRead,
            )
            .await?;

        self.repository.list_entities(actor.tenant_id()).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
    use qryvanta_domain::{AuditAction, EntityDefinition, Permission};
    use tokio::sync::Mutex;

    use crate::{AuthorizationRepository, AuthorizationService};

    use super::{AuditEvent, AuditRepository, MetadataRepository, MetadataService};

    struct FakeRepository {
        entities: Mutex<HashMap<(TenantId, String), EntityDefinition>>,
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                entities: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl MetadataRepository for FakeRepository {
        async fn save_entity(
            &self,
            tenant_id: TenantId,
            entity: EntityDefinition,
        ) -> AppResult<()> {
            let key = (tenant_id, entity.logical_name().as_str().to_owned());
            let mut entities = self.entities.lock().await;

            if entities.contains_key(&key) {
                return Err(AppError::Conflict(format!(
                    "entity '{}' already exists for tenant '{}'",
                    key.1, key.0
                )));
            }

            entities.insert(key, entity);
            Ok(())
        }

        async fn list_entities(&self, tenant_id: TenantId) -> AppResult<Vec<EntityDefinition>> {
            let entities = self.entities.lock().await;
            let mut listed: Vec<EntityDefinition> = entities
                .iter()
                .filter_map(|((stored_tenant_id, _), entity)| {
                    (stored_tenant_id == &tenant_id).then_some(entity.clone())
                })
                .collect();
            listed.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });
            Ok(listed)
        }
    }

    #[derive(Default)]
    struct FakeAuditRepository {
        events: Mutex<Vec<AuditEvent>>,
    }

    #[async_trait]
    impl AuditRepository for FakeAuditRepository {
        async fn append_event(&self, event: AuditEvent) -> AppResult<()> {
            self.events.lock().await.push(event);
            Ok(())
        }
    }

    struct FakeAuthorizationRepository {
        grants: HashMap<(TenantId, String), Vec<Permission>>,
    }

    #[async_trait]
    impl AuthorizationRepository for FakeAuthorizationRepository {
        async fn list_permissions_for_subject(
            &self,
            tenant_id: TenantId,
            subject: &str,
        ) -> AppResult<Vec<Permission>> {
            Ok(self
                .grants
                .get(&(tenant_id, subject.to_owned()))
                .cloned()
                .unwrap_or_default())
        }
    }

    fn actor(tenant_id: TenantId, subject: &str) -> UserIdentity {
        UserIdentity::new(subject, subject, None, tenant_id)
    }

    fn build_service(
        grants: HashMap<(TenantId, String), Vec<Permission>>,
    ) -> (MetadataService, Arc<FakeAuditRepository>) {
        let authorization_service =
            AuthorizationService::new(Arc::new(FakeAuthorizationRepository { grants }));
        let audit_repository = Arc::new(FakeAuditRepository::default());
        let service = MetadataService::new(
            Arc::new(FakeRepository::new()),
            authorization_service,
            audit_repository.clone(),
        );
        (service, audit_repository)
    }

    #[tokio::test]
    async fn register_entity_persists_data_and_writes_audit_event() {
        let tenant_id = TenantId::new();
        let subject = "alice";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![
                Permission::MetadataEntityCreate,
                Permission::MetadataEntityRead,
                Permission::MetadataFieldWrite,
            ],
        )]);
        let (service, audit_repository) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let created = service.register_entity(&actor, "contact", "Contact").await;
        assert!(created.is_ok());

        let entities = service.list_entities(&actor).await;
        assert!(entities.is_ok());
        assert_eq!(entities.unwrap_or_default().len(), 1);

        let events = audit_repository.events.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, AuditAction::MetadataEntityCreated);
        assert_eq!(events[0].resource_id, "contact");
    }

    #[tokio::test]
    async fn register_entity_requires_create_permission() {
        let tenant_id = TenantId::new();
        let (service, _) = build_service(HashMap::new());
        let actor = actor(tenant_id, "bob");

        let result = service.register_entity(&actor, "account", "Account").await;
        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn register_entity_requires_field_write_permission() {
        let tenant_id = TenantId::new();
        let subject = "bob";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![Permission::MetadataEntityCreate],
        )]);
        let (service, _) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let result = service.register_entity(&actor, "account", "Account").await;
        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn list_entities_requires_read_permission() {
        let tenant_id = TenantId::new();
        let subject = "carol";
        let grants = HashMap::from([(
            (tenant_id, subject.to_owned()),
            vec![Permission::MetadataEntityCreate],
        )]);
        let (service, _) = build_service(grants);
        let actor = actor(tenant_id, subject);

        let result = service.list_entities(&actor).await;
        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }
}
