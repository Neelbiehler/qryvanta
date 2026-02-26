use super::*;

impl AppService {
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
        let default_form_logical_name =
            input.default_form_logical_name.clone().unwrap_or_else(|| {
                forms
                    .first()
                    .map(|form| form.logical_name().as_str().to_owned())
                    .unwrap_or_else(|| "main_form".to_owned())
            });
        let default_list_view_logical_name = input
            .default_list_view_logical_name
            .clone()
            .unwrap_or_else(|| {
                list_views
                    .first()
                    .map(|view| view.logical_name().as_str().to_owned())
                    .unwrap_or_else(|| "main_view".to_owned())
            });

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
