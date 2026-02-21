use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityAction, AppEntityBinding, AppEntityRolePermission, AuditAction,
    Permission, PublishedEntitySchema, RuntimeRecord,
};
use serde_json::Value;

use crate::app_ports::{
    AppRepository, BindAppEntityInput, CreateAppInput, RuntimeRecordService,
    SaveAppRoleEntityPermissionInput, SubjectEntityPermission,
};
use crate::{
    AuditEvent, AuditRepository, AuthorizationService, MetadataService, RecordListQuery,
    RuntimeRecordQuery,
};

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

    async fn query_runtime_records_unchecked(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.query_runtime_records_unchecked(actor, entity_logical_name, query)
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

    /// Queries runtime records in app scope.
    pub async fn query_records(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        entity_logical_name: &str,
        query: RuntimeRecordQuery,
    ) -> AppResult<Vec<RuntimeRecord>> {
        self.require_entity_action(
            actor,
            app_logical_name,
            entity_logical_name,
            AppEntityAction::Read,
        )
        .await?;

        self.runtime_record_service
            .query_runtime_records_unchecked(actor, entity_logical_name, query)
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
mod tests;
