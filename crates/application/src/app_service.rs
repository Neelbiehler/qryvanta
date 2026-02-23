use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::{AppError, AppResult, TenantId, UserIdentity};
use qryvanta_domain::{
    AppDefinition, AppEntityAction, AppEntityBinding, AppEntityForm, AppEntityRolePermission,
    AppEntityView, AppEntityViewMode, AppSitemap, AuditAction, Permission, PublishedEntitySchema,
    RuntimeRecord, SitemapArea, SitemapGroup, SitemapSubArea, SitemapTarget,
};
use serde_json::Value;

use crate::app_ports::{
    AppRepository, BindAppEntityInput, CreateAppInput, RuntimeRecordService,
    SaveAppRoleEntityPermissionInput, SaveAppSitemapInput, SubjectEntityPermission,
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

        let forms = resolve_forms(&input)?;
        let list_views = resolve_list_views(&input)?;
        let default_form_logical_name = input
            .default_form_logical_name
            .clone()
            .unwrap_or_else(|| "main_form".to_owned());
        let default_list_view_logical_name = input
            .default_list_view_logical_name
            .clone()
            .unwrap_or_else(|| "main_view".to_owned());

        let binding = AppEntityBinding::new(
            input.app_logical_name,
            input.entity_logical_name,
            input.navigation_label,
            input.navigation_order,
            forms,
            list_views,
            default_form_logical_name,
            default_list_view_logical_name,
            input.default_view_mode.unwrap_or(AppEntityViewMode::Grid),
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
    ) -> AppResult<AppSitemap> {
        self.ensure_subject_can_access_app(actor, app_logical_name)
            .await?;

        let permissions = self
            .repository
            .list_subject_entity_permissions(actor.tenant_id(), actor.subject(), app_logical_name)
            .await?;

        let sitemap = if let Some(sitemap) = self
            .repository
            .get_sitemap(actor.tenant_id(), app_logical_name)
            .await?
        {
            sitemap
        } else {
            let bindings = self
                .repository
                .list_app_entity_bindings(actor.tenant_id(), app_logical_name)
                .await?;
            Self::derive_sitemap_from_bindings(app_logical_name, bindings)?
        };

        Self::filter_sitemap_by_permissions(sitemap, permissions)
    }

    /// Returns app sitemap in admin scope (without subject filtering).
    pub async fn get_sitemap(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
    ) -> AppResult<AppSitemap> {
        self.require_admin(actor).await?;
        self.require_app_exists(actor.tenant_id(), app_logical_name)
            .await?;
        if let Some(sitemap) = self
            .repository
            .get_sitemap(actor.tenant_id(), app_logical_name)
            .await?
        {
            return Ok(sitemap);
        }

        let bindings = self
            .repository
            .list_app_entity_bindings(actor.tenant_id(), app_logical_name)
            .await?;
        Self::derive_sitemap_from_bindings(app_logical_name, bindings)
    }

    /// Saves app sitemap in admin scope.
    pub async fn save_sitemap(
        &self,
        actor: &UserIdentity,
        input: SaveAppSitemapInput,
    ) -> AppResult<AppSitemap> {
        self.require_admin(actor).await?;
        self.require_app_exists(actor.tenant_id(), input.app_logical_name.as_str())
            .await?;

        if input.sitemap.app_logical_name().as_str() != input.app_logical_name.as_str() {
            return Err(AppError::Validation(format!(
                "sitemap app '{}' must match path app '{}'",
                input.sitemap.app_logical_name().as_str(),
                input.app_logical_name
            )));
        }

        self.repository
            .save_sitemap(actor.tenant_id(), input.sitemap.clone())
            .await?;
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::AppEntityBound,
                resource_type: "app_sitemap".to_owned(),
                resource_id: input.app_logical_name.clone(),
                detail: Some(format!(
                    "saved sitemap for app '{}'",
                    input.app_logical_name
                )),
            })
            .await?;

        Ok(input.sitemap)
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

        for link in &query.links {
            self.require_entity_action(
                actor,
                app_logical_name,
                link.target_entity_logical_name.as_str(),
                AppEntityAction::Read,
            )
            .await?;
        }

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

    fn derive_sitemap_from_bindings(
        app_logical_name: &str,
        bindings: Vec<AppEntityBinding>,
    ) -> AppResult<AppSitemap> {
        let mut sorted_bindings = bindings;
        sorted_bindings.sort_by(|left, right| {
            left.navigation_order()
                .cmp(&right.navigation_order())
                .then_with(|| {
                    left.entity_logical_name()
                        .as_str()
                        .cmp(right.entity_logical_name().as_str())
                })
        });

        let mut sub_areas = Vec::with_capacity(sorted_bindings.len());
        for binding in sorted_bindings {
            let entity_logical_name = binding.entity_logical_name().as_str().to_owned();
            let display_name = binding
                .navigation_label()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| entity_logical_name.clone());

            sub_areas.push(SitemapSubArea::new(
                entity_logical_name.clone(),
                display_name,
                binding.navigation_order(),
                SitemapTarget::Entity {
                    entity_logical_name,
                    default_form: Some(binding.default_form_logical_name().as_str().to_owned()),
                    default_view: Some(
                        binding.default_list_view_logical_name().as_str().to_owned(),
                    ),
                },
                None,
            )?);
        }

        let area = SitemapArea::new(
            "main_area",
            "Main",
            0,
            None,
            vec![SitemapGroup::new("main_group", "Main", 0, sub_areas)?],
        )?;

        AppSitemap::new(app_logical_name, vec![area])
    }

    fn filter_sitemap_by_permissions(
        sitemap: AppSitemap,
        permissions: Vec<SubjectEntityPermission>,
    ) -> AppResult<AppSitemap> {
        let mut filtered_areas = Vec::new();
        for area in sitemap.areas() {
            let mut filtered_groups = Vec::new();
            for group in area.groups() {
                let mut filtered_sub_areas = Vec::new();
                for sub_area in group.sub_areas() {
                    let allowed = match sub_area.target() {
                        SitemapTarget::Entity {
                            entity_logical_name,
                            ..
                        } => permissions
                            .iter()
                            .find(|permission| {
                                permission.entity_logical_name == *entity_logical_name
                            })
                            .map(|permission| permission.can_read)
                            .unwrap_or(false),
                        SitemapTarget::Dashboard { .. } | SitemapTarget::CustomPage { .. } => true,
                    };

                    if allowed {
                        filtered_sub_areas.push(sub_area.clone());
                    }
                }

                if !filtered_sub_areas.is_empty() {
                    filtered_groups.push(SitemapGroup::new(
                        group.logical_name().as_str(),
                        group.display_name().as_str(),
                        group.position(),
                        filtered_sub_areas,
                    )?);
                }
            }

            if !filtered_groups.is_empty() {
                filtered_areas.push(SitemapArea::new(
                    area.logical_name().as_str(),
                    area.display_name().as_str(),
                    area.position(),
                    area.icon().map(ToOwned::to_owned),
                    filtered_groups,
                )?);
            }
        }

        AppSitemap::new(sitemap.app_logical_name().as_str(), filtered_areas)
    }
}

fn resolve_forms(input: &BindAppEntityInput) -> AppResult<Vec<AppEntityForm>> {
    if let Some(forms) = &input.forms {
        return forms
            .iter()
            .map(|form| {
                AppEntityForm::new(
                    form.logical_name.clone(),
                    form.display_name.clone(),
                    form.field_logical_names.clone(),
                )
            })
            .collect();
    }

    Ok(vec![AppEntityForm::new(
        "main_form",
        "Main Form",
        input.form_field_logical_names.clone().unwrap_or_default(),
    )?])
}

fn resolve_list_views(input: &BindAppEntityInput) -> AppResult<Vec<AppEntityView>> {
    if let Some(list_views) = &input.list_views {
        return list_views
            .iter()
            .map(|view| {
                AppEntityView::new(
                    view.logical_name.clone(),
                    view.display_name.clone(),
                    view.field_logical_names.clone(),
                )
            })
            .collect();
    }

    Ok(vec![AppEntityView::new(
        "main_view",
        "Main View",
        input.list_field_logical_names.clone().unwrap_or_default(),
    )?])
}

#[cfg(test)]
mod tests;
