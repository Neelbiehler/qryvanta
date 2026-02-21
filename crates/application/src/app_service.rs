use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityAction, AppEntityBinding, AppEntityRolePermission, AuditAction,
    Permission, PublishedEntitySchema, RuntimeRecord,
};
use serde_json::Value;

use crate::{AuditEvent, AuditRepository, AuthorizationService, MetadataService, RecordListQuery};

/// Input payload for app creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAppInput {
    /// Stable app logical name.
    pub logical_name: String,
    /// App display name.
    pub display_name: String,
    /// Optional app description.
    pub description: Option<String>,
}

/// Input payload for binding an entity into app navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindAppEntityInput {
    /// Parent app logical name.
    pub app_logical_name: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Optional display label in navigation.
    pub navigation_label: Option<String>,
    /// Display ordering value.
    pub navigation_order: i32,
}

/// Input payload for app role entity permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveAppRoleEntityPermissionInput {
    /// Parent app logical name.
    pub app_logical_name: String,
    /// Role name to configure.
    pub role_name: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Whether read access is granted.
    pub can_read: bool,
    /// Whether create access is granted.
    pub can_create: bool,
    /// Whether update access is granted.
    pub can_update: bool,
    /// Whether delete access is granted.
    pub can_delete: bool,
}

/// Effective subject permissions for an entity in an app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectEntityPermission {
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Read access.
    pub can_read: bool,
    /// Create access.
    pub can_create: bool,
    /// Update access.
    pub can_update: bool,
    /// Delete access.
    pub can_delete: bool,
}

impl SubjectEntityPermission {
    /// Returns whether an action is allowed by this capability.
    #[must_use]
    pub fn allows(&self, action: AppEntityAction) -> bool {
        match action {
            AppEntityAction::Read => self.can_read,
            AppEntityAction::Create => self.can_create,
            AppEntityAction::Update => self.can_update,
            AppEntityAction::Delete => self.can_delete,
        }
    }
}

/// Repository port for app definitions and app-scoped permissions.
#[async_trait]
pub trait AppRepository: Send + Sync {
    /// Creates a new app definition.
    async fn create_app(&self, tenant_id: TenantId, app: AppDefinition) -> AppResult<()>;

    /// Lists all apps for a tenant.
    async fn list_apps(&self, tenant_id: TenantId) -> AppResult<Vec<AppDefinition>>;

    /// Returns one app by logical name.
    async fn find_app(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppDefinition>>;

    /// Saves an app entity navigation binding.
    async fn save_app_entity_binding(
        &self,
        tenant_id: TenantId,
        binding: AppEntityBinding,
    ) -> AppResult<()>;

    /// Lists entities bound into an app navigation.
    async fn list_app_entity_bindings(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>>;

    /// Saves app-scoped role permissions for an entity.
    async fn save_app_role_entity_permission(
        &self,
        tenant_id: TenantId,
        permission: AppEntityRolePermission,
    ) -> AppResult<()>;

    /// Lists configured role permissions for an app.
    async fn list_app_role_entity_permissions(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityRolePermission>>;

    /// Lists apps accessible to the subject by role bindings.
    async fn list_accessible_apps(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Vec<AppDefinition>>;

    /// Returns whether the subject can access an app.
    async fn subject_can_access_app(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<bool>;

    /// Returns effective subject permissions for one entity in an app.
    async fn subject_entity_permission(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<Option<SubjectEntityPermission>>;

    /// Returns effective subject permissions for every entity in an app.
    async fn list_subject_entity_permissions(
        &self,
        tenant_id: TenantId,
        subject: &str,
        app_logical_name: &str,
    ) -> AppResult<Vec<SubjectEntityPermission>>;
}

/// Runtime record gateway used by app-scoped execution.
#[async_trait]
pub trait RuntimeRecordService: Send + Sync {
    /// Returns latest published schema for an entity.
    async fn latest_published_schema_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>>;

    /// Lists runtime records without global permission checks.
    async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>>;

    /// Fetches one runtime record without global permission checks.
    async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord>;

    /// Creates runtime record without global permission checks.
    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Updates runtime record without global permission checks.
    async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord>;

    /// Deletes runtime record without global permission checks.
    async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()>;
}

#[async_trait]
impl RuntimeRecordService for MetadataService {
    async fn latest_published_schema_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Option<PublishedEntitySchema>> {
        self.latest_published_schema_unchecked(actor, entity_logical_name)
            .await
    }

    async fn list_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.list_runtime_records_unchecked(actor, entity_logical_name, query)
            .await
    }

    async fn get_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        self.get_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }

    async fn create_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.create_runtime_record_unchecked(actor, entity_logical_name, data)
            .await
    }

    async fn update_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.update_runtime_record_unchecked(actor, entity_logical_name, record_id, data)
            .await
    }

    async fn delete_runtime_record_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.delete_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }
}

/// Application service for app builder and app-scoped runtime access.
#[derive(Clone)]
pub struct AppService {
    authorization_service: AuthorizationService,
    repository: Arc<dyn AppRepository>,
    runtime_record_service: Arc<dyn RuntimeRecordService>,
    audit_repository: Arc<dyn AuditRepository>,
}

impl AppService {
    /// Creates a new app service.
    #[must_use]
    pub fn new(
        authorization_service: AuthorizationService,
        repository: Arc<dyn AppRepository>,
        runtime_record_service: Arc<dyn RuntimeRecordService>,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            authorization_service,
            repository,
            runtime_record_service,
            audit_repository,
        }
    }

    /// Creates a new app definition.
    pub async fn create_app(
        &self,
        actor: &UserIdentity,
        input: CreateAppInput,
    ) -> AppResult<AppDefinition> {
        self.require_admin(actor).await?;

        let app = AppDefinition::new(input.logical_name, input.display_name, input.description)?;
        self.repository
            .create_app(actor.tenant_id(), app.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::AppCreated,
                resource_type: "app_definition".to_owned(),
                resource_id: app.logical_name().as_str().to_owned(),
                detail: Some(format!("created app '{}'", app.logical_name().as_str())),
            })
            .await?;

        Ok(app)
    }

    /// Lists all app definitions in tenant scope for administrators.
    pub async fn list_apps(&self, actor: &UserIdentity) -> AppResult<Vec<AppDefinition>> {
        self.require_admin(actor).await?;
        self.repository.list_apps(actor.tenant_id()).await
    }

    /// Saves app navigation binding for an entity.
    pub async fn bind_entity(
        &self,
        actor: &UserIdentity,
        input: BindAppEntityInput,
    ) -> AppResult<AppEntityBinding> {
        self.require_admin(actor).await?;
        self.require_app_exists(actor.tenant_id(), input.app_logical_name.as_str())
            .await?;

        let binding = AppEntityBinding::new(
            input.app_logical_name,
            input.entity_logical_name,
            input.navigation_label,
            input.navigation_order,
        )?;

        self.repository
            .save_app_entity_binding(actor.tenant_id(), binding.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::AppEntityBound,
                resource_type: "app_entity_binding".to_owned(),
                resource_id: format!(
                    "{}.{}",
                    binding.app_logical_name().as_str(),
                    binding.entity_logical_name().as_str()
                ),
                detail: Some(format!(
                    "bound entity '{}' in app '{}'",
                    binding.entity_logical_name().as_str(),
                    binding.app_logical_name().as_str()
                )),
            })
            .await?;

        Ok(binding)
    }

    /// Lists navigation bindings in an app.
    pub async fn list_app_entities(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>> {
        self.require_admin(actor).await?;
        self.repository
            .list_app_entity_bindings(actor.tenant_id(), app_logical_name)
            .await
    }

    /// Saves role permissions for one app entity.
    pub async fn save_role_entity_permission(
        &self,
        actor: &UserIdentity,
        input: SaveAppRoleEntityPermissionInput,
    ) -> AppResult<AppEntityRolePermission> {
        self.require_admin(actor).await?;
        self.require_app_exists(actor.tenant_id(), input.app_logical_name.as_str())
            .await?;

        let permission = AppEntityRolePermission::new(
            input.app_logical_name,
            input.role_name,
            input.entity_logical_name,
            input.can_read,
            input.can_create,
            input.can_update,
            input.can_delete,
        )?;

        self.repository
            .save_app_role_entity_permission(actor.tenant_id(), permission.clone())
            .await?;

        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::AppRoleEntityPermissionSaved,
                resource_type: "app_role_entity_permission".to_owned(),
                resource_id: format!(
                    "{}.{}.{}",
                    permission.app_logical_name().as_str(),
                    permission.role_name().as_str(),
                    permission.entity_logical_name().as_str()
                ),
                detail: Some(format!(
                    "saved app entity permissions for role '{}' on entity '{}' in app '{}'",
                    permission.role_name().as_str(),
                    permission.entity_logical_name().as_str(),
                    permission.app_logical_name().as_str()
                )),
            })
            .await?;

        Ok(permission)
    }

    /// Lists role-entity permission entries configured for an app.
    pub async fn list_role_entity_permissions(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityRolePermission>> {
        self.require_admin(actor).await?;
        self.repository
            .list_app_role_entity_permissions(actor.tenant_id(), app_logical_name)
            .await
    }

    /// Lists apps accessible to the current worker by role bindings.
    pub async fn list_accessible_apps(
        &self,
        actor: &UserIdentity,
    ) -> AppResult<Vec<AppDefinition>> {
        self.repository
            .list_accessible_apps(actor.tenant_id(), actor.subject())
            .await
    }

    /// Lists app navigation entities visible to current worker.
    pub async fn app_navigation_for_subject(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>> {
        self.ensure_subject_can_access_app(actor, app_logical_name)
            .await?;

        let bindings = self
            .repository
            .list_app_entity_bindings(actor.tenant_id(), app_logical_name)
            .await?;

        let permissions = self
            .repository
            .list_subject_entity_permissions(actor.tenant_id(), actor.subject(), app_logical_name)
            .await?;

        Ok(bindings
            .into_iter()
            .filter(|binding| {
                permissions
                    .iter()
                    .find(|permission| {
                        permission.entity_logical_name == binding.entity_logical_name().as_str()
                    })
                    .map(|permission| permission.can_read)
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Returns effective capabilities for one app entity and subject.
    pub async fn entity_capabilities_for_subject(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<SubjectEntityPermission> {
        self.ensure_subject_can_access_app(actor, app_logical_name)
            .await?;

        self.repository
            .subject_entity_permission(
                actor.tenant_id(),
                actor.subject(),
                app_logical_name,
                entity_logical_name,
            )
            .await?
            .ok_or_else(|| {
                AppError::Forbidden(format!(
                    "subject '{}' has no app capabilities for entity '{}' in app '{}'",
                    actor.subject(),
                    entity_logical_name,
                    app_logical_name
                ))
            })
    }

    /// Fetches published schema for a worker-facing app entity.
    pub async fn schema_for_subject(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
    ) -> AppResult<PublishedEntitySchema> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .latest_published_schema_unchecked(actor, entity_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "entity '{}' has no published schema",
                    entity_logical_name
                ))
            })
    }

    /// Lists runtime records in app scope.
    pub async fn list_records(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        query: RecordListQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .list_runtime_records_unchecked(actor, entity_logical_name, query)
            .await
    }

    /// Fetches one runtime record in app scope.
    pub async fn get_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<RuntimeRecord> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .get_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }

    /// Creates one runtime record in app scope.
    pub async fn create_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Create,
        )
        .await?;

        self.runtime_record_service
            .create_runtime_record_unchecked(actor, entity_logical_name, data)
            .await
    }

    /// Updates one runtime record in app scope.
    pub async fn update_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        record_id: &str,
        data: Value,
    ) -> AppResult<RuntimeRecord> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Update,
        )
        .await?;

        self.runtime_record_service
            .update_runtime_record_unchecked(actor, entity_logical_name, record_id, data)
            .await
    }

    /// Deletes one runtime record in app scope.
    pub async fn delete_record(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        record_id: &str,
    ) -> AppResult<()> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Delete,
        )
        .await?;

        self.runtime_record_service
            .delete_runtime_record_unchecked(actor, entity_logical_name, record_id)
            .await
    }

    async fn require_admin(&self, actor: &UserIdentity) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::SecurityRoleManage,
            )
            .await
    }

    async fn require_app_exists(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<()> {
        let app = self
            .repository
            .find_app(tenant_id, app_logical_name)
            .await?;
        if app.is_none() {
            return Err(AppError::NotFound(format!(
                "app '{}' does not exist for tenant '{}'",
                app_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    async fn ensure_subject_can_access_app(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
    ) -> AppResult<()> {
        let can_access = self
            .repository
            .subject_can_access_app(actor.tenant_id(), actor.subject(), app_logical_name)
            .await?;

        if !can_access {
            return Err(AppError::Forbidden(format!(
                "subject '{}' is not assigned to app '{}'",
                actor.subject(),
                app_logical_name
            )));
        }

        Ok(())
    }

    async fn require_entity_action(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        action: AppEntityAction,
    ) -> AppResult<()> {
        self.ensure_subject_can_access_app(actor, app_logical_name)
            .await?;
        let permission = self
            .repository
            .subject_entity_permission(
                actor.tenant_id(),
                actor.subject(),
                app_logical_name,
                entity_logical_name,
            )
            .await?;

        if permission
            .map(|value| value.allows(action))
            .unwrap_or(false)
        {
            return Ok(());
        }

        Err(AppError::Forbidden(format!(
            "subject '{}' is missing '{}' access for entity '{}' in app '{}'",
            actor.subject(),
            action.as_str(),
            entity_logical_name,
            app_logical_name
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use serde_json::{Value, json};
    use tokio::sync::Mutex;

    use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
    use qryvanta_domain::{
        AppDefinition, AppEntityBinding, AppEntityRolePermission, Permission, RuntimeRecord,
    };

    use crate::{
        AuditEvent, AuditRepository, AuthorizationRepository, AuthorizationService, RecordListQuery,
    };

    use super::{
        AppRepository, AppService, CreateAppInput, RuntimeRecordService, SubjectEntityPermission,
    };

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

    #[derive(Default)]
    struct FakeAppRepository {
        bindings: Mutex<HashMap<(TenantId, String), Vec<AppEntityBinding>>>,
        subject_permissions:
            Mutex<HashMap<(TenantId, String, String), Vec<SubjectEntityPermission>>>,
        subject_access: Mutex<HashMap<(TenantId, String, String), bool>>,
    }

    #[async_trait]
    impl AppRepository for FakeAppRepository {
        async fn create_app(&self, _tenant_id: TenantId, _app: AppDefinition) -> AppResult<()> {
            Ok(())
        }

        async fn list_apps(&self, _tenant_id: TenantId) -> AppResult<Vec<AppDefinition>> {
            Ok(Vec::new())
        }

        async fn find_app(
            &self,
            _tenant_id: TenantId,
            _app_logical_name: &str,
        ) -> AppResult<Option<AppDefinition>> {
            Ok(Some(AppDefinition::new("sales", "Sales", None)?))
        }

        async fn save_app_entity_binding(
            &self,
            _tenant_id: TenantId,
            _binding: AppEntityBinding,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn list_app_entity_bindings(
            &self,
            tenant_id: TenantId,
            app_logical_name: &str,
        ) -> AppResult<Vec<AppEntityBinding>> {
            Ok(self
                .bindings
                .lock()
                .await
                .get(&(tenant_id, app_logical_name.to_owned()))
                .cloned()
                .unwrap_or_default())
        }

        async fn save_app_role_entity_permission(
            &self,
            _tenant_id: TenantId,
            _permission: AppEntityRolePermission,
        ) -> AppResult<()> {
            Ok(())
        }

        async fn list_app_role_entity_permissions(
            &self,
            _tenant_id: TenantId,
            _app_logical_name: &str,
        ) -> AppResult<Vec<AppEntityRolePermission>> {
            Ok(Vec::new())
        }

        async fn list_accessible_apps(
            &self,
            _tenant_id: TenantId,
            _subject: &str,
        ) -> AppResult<Vec<AppDefinition>> {
            Ok(Vec::new())
        }

        async fn subject_can_access_app(
            &self,
            tenant_id: TenantId,
            subject: &str,
            app_logical_name: &str,
        ) -> AppResult<bool> {
            Ok(*self
                .subject_access
                .lock()
                .await
                .get(&(tenant_id, subject.to_owned(), app_logical_name.to_owned()))
                .unwrap_or(&false))
        }

        async fn subject_entity_permission(
            &self,
            tenant_id: TenantId,
            subject: &str,
            app_logical_name: &str,
            entity_logical_name: &str,
        ) -> AppResult<Option<SubjectEntityPermission>> {
            Ok(self
                .subject_permissions
                .lock()
                .await
                .get(&(tenant_id, subject.to_owned(), app_logical_name.to_owned()))
                .and_then(|permissions| {
                    permissions
                        .iter()
                        .find(|permission| permission.entity_logical_name == entity_logical_name)
                        .cloned()
                }))
        }

        async fn list_subject_entity_permissions(
            &self,
            tenant_id: TenantId,
            subject: &str,
            app_logical_name: &str,
        ) -> AppResult<Vec<SubjectEntityPermission>> {
            Ok(self
                .subject_permissions
                .lock()
                .await
                .get(&(tenant_id, subject.to_owned(), app_logical_name.to_owned()))
                .cloned()
                .unwrap_or_default())
        }
    }

    #[derive(Default)]
    struct FakeRuntimeRecordService {
        create_calls: Mutex<usize>,
    }

    #[async_trait]
    impl RuntimeRecordService for FakeRuntimeRecordService {
        async fn latest_published_schema_unchecked(
            &self,
            _actor: &UserIdentity,
            _entity_logical_name: &str,
        ) -> AppResult<Option<qryvanta_domain::PublishedEntitySchema>> {
            Ok(None)
        }

        async fn list_runtime_records_unchecked(
            &self,
            _actor: &UserIdentity,
            _entity_logical_name: &str,
            _query: RecordListQuery,
        ) -> AppResult<Vec<RuntimeRecord>> {
            Ok(Vec::new())
        }

        async fn get_runtime_record_unchecked(
            &self,
            _actor: &UserIdentity,
            entity_logical_name: &str,
            record_id: &str,
        ) -> AppResult<RuntimeRecord> {
            RuntimeRecord::new(record_id, entity_logical_name, json!({"id": record_id}))
        }

        async fn create_runtime_record_unchecked(
            &self,
            _actor: &UserIdentity,
            entity_logical_name: &str,
            data: Value,
        ) -> AppResult<RuntimeRecord> {
            let mut calls = self.create_calls.lock().await;
            *calls += 1;
            RuntimeRecord::new("record-1", entity_logical_name, data)
        }

        async fn update_runtime_record_unchecked(
            &self,
            _actor: &UserIdentity,
            entity_logical_name: &str,
            record_id: &str,
            data: Value,
        ) -> AppResult<RuntimeRecord> {
            RuntimeRecord::new(record_id, entity_logical_name, data)
        }

        async fn delete_runtime_record_unchecked(
            &self,
            _actor: &UserIdentity,
            _entity_logical_name: &str,
            _record_id: &str,
        ) -> AppResult<()> {
            Ok(())
        }
    }

    fn actor(tenant_id: TenantId, subject: &str) -> UserIdentity {
        UserIdentity::new(subject, subject, None, tenant_id)
    }

    fn build_service(
        grants: HashMap<(TenantId, String), Vec<Permission>>,
        app_repository: Arc<FakeAppRepository>,
        runtime_record_service: Arc<FakeRuntimeRecordService>,
    ) -> AppService {
        let authorization_service =
            AuthorizationService::new(Arc::new(FakeAuthorizationRepository { grants }));
        AppService::new(
            authorization_service,
            app_repository,
            runtime_record_service,
            Arc::new(FakeAuditRepository::default()),
        )
    }

    #[tokio::test]
    async fn create_app_requires_manage_permission() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "alice");
        let service = build_service(
            HashMap::new(),
            Arc::new(FakeAppRepository::default()),
            Arc::new(FakeRuntimeRecordService::default()),
        );

        let result = service
            .create_app(
                &actor,
                CreateAppInput {
                    logical_name: "sales".to_owned(),
                    display_name: "Sales".to_owned(),
                    description: None,
                },
            )
            .await;

        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn app_navigation_only_includes_readable_entities() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "worker");
        let app_repository = Arc::new(FakeAppRepository::default());
        let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
        let service = build_service(
            HashMap::new(),
            app_repository.clone(),
            runtime_record_service,
        );

        app_repository
            .subject_access
            .lock()
            .await
            .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);

        app_repository.bindings.lock().await.insert(
            (tenant_id, "sales".to_owned()),
            vec![
                AppEntityBinding::new("sales", "account", None, 0)
                    .unwrap_or_else(|_| unreachable!()),
                AppEntityBinding::new("sales", "invoice", None, 1)
                    .unwrap_or_else(|_| unreachable!()),
            ],
        );

        app_repository.subject_permissions.lock().await.insert(
            (tenant_id, "worker".to_owned(), "sales".to_owned()),
            vec![
                SubjectEntityPermission {
                    entity_logical_name: "account".to_owned(),
                    can_read: true,
                    can_create: false,
                    can_update: false,
                    can_delete: false,
                },
                SubjectEntityPermission {
                    entity_logical_name: "invoice".to_owned(),
                    can_read: false,
                    can_create: true,
                    can_update: false,
                    can_delete: false,
                },
            ],
        );

        let navigation = service.app_navigation_for_subject(&actor, "sales").await;

        assert!(navigation.is_ok());
        let navigation = navigation.unwrap_or_default();
        assert_eq!(navigation.len(), 1);
        assert_eq!(navigation[0].entity_logical_name().as_str(), "account");
    }

    #[tokio::test]
    async fn create_record_is_forbidden_without_create_capability() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "worker");
        let app_repository = Arc::new(FakeAppRepository::default());
        let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
        let service = build_service(
            HashMap::new(),
            app_repository.clone(),
            runtime_record_service.clone(),
        );

        app_repository
            .subject_access
            .lock()
            .await
            .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);
        app_repository.subject_permissions.lock().await.insert(
            (tenant_id, "worker".to_owned(), "sales".to_owned()),
            vec![SubjectEntityPermission {
                entity_logical_name: "account".to_owned(),
                can_read: true,
                can_create: false,
                can_update: false,
                can_delete: false,
            }],
        );

        let result = service
            .create_record(&actor, "sales", "account", json!({"name": "A"}))
            .await;

        assert!(matches!(result, Err(AppError::Forbidden(_))));
        assert_eq!(*runtime_record_service.create_calls.lock().await, 0);
    }

    #[tokio::test]
    async fn create_record_calls_runtime_when_create_capability_exists() {
        let tenant_id = TenantId::new();
        let actor = actor(tenant_id, "worker");
        let app_repository = Arc::new(FakeAppRepository::default());
        let runtime_record_service = Arc::new(FakeRuntimeRecordService::default());
        let service = build_service(
            HashMap::new(),
            app_repository.clone(),
            runtime_record_service.clone(),
        );

        app_repository
            .subject_access
            .lock()
            .await
            .insert((tenant_id, "worker".to_owned(), "sales".to_owned()), true);
        app_repository.subject_permissions.lock().await.insert(
            (tenant_id, "worker".to_owned(), "sales".to_owned()),
            vec![SubjectEntityPermission {
                entity_logical_name: "account".to_owned(),
                can_read: true,
                can_create: true,
                can_update: false,
                can_delete: false,
            }],
        );

        let created = service
            .create_record(&actor, "sales", "account", json!({"name": "A"}))
            .await;

        assert!(created.is_ok());
        let created = created.unwrap_or_else(|_| unreachable!());
        assert_eq!(created.entity_logical_name().as_str(), "account");
        assert_eq!(*runtime_record_service.create_calls.lock().await, 1);
    }
}
