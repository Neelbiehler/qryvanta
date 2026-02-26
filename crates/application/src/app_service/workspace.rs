use super::*;

impl AppService {
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

        let sitemap = Self::normalize_sitemap_order(&sitemap)?;

        Self::filter_sitemap_by_permissions(sitemap, permissions)
    }

    /// Returns a minimal metadata-driven dashboard surface for worker users.
    pub async fn get_dashboard_for_subject(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        dashboard_logical_name: &str,
    ) -> AppResult<DashboardDefinition> {
        self.ensure_subject_can_access_app(actor, app_logical_name)
            .await?;

        let bindings = self
            .repository
            .list_app_entity_bindings(actor.tenant_id(), app_logical_name)
            .await?;
        let sitemap = if let Some(sitemap) = self
            .repository
            .get_sitemap(actor.tenant_id(), app_logical_name)
            .await?
        {
            sitemap
        } else {
            Self::derive_sitemap_from_bindings(app_logical_name, bindings.clone())?
        };
        let sitemap = Self::normalize_sitemap_order(&sitemap)?;

        let Some(display_name) = sitemap
            .areas()
            .iter()
            .flat_map(SitemapArea::groups)
            .flat_map(SitemapGroup::sub_areas)
            .find_map(|sub_area| match sub_area.target() {
                SitemapTarget::Dashboard {
                    dashboard_logical_name: logical_name,
                } if logical_name == dashboard_logical_name => {
                    Some(sub_area.display_name().as_str().to_owned())
                }
                _ => None,
            })
        else {
            return Err(AppError::NotFound(format!(
                "dashboard '{}' does not exist in app '{}' sitemap",
                dashboard_logical_name, app_logical_name
            )));
        };

        let widgets: AppResult<Vec<DashboardWidget>> = bindings
            .iter()
            .take(6)
            .enumerate()
            .map(|(index, binding)| {
                let entity_logical_name = binding.entity_logical_name().as_str();
                let widget_position = i32::try_from(index).map_err(|_| {
                    AppError::Validation("dashboard widget position exceeded i32 range".to_owned())
                })?;
                let display_label = binding
                    .navigation_label()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| entity_logical_name.to_owned());

                let chart = ChartDefinition::new(
                    format!("{entity_logical_name}_record_count"),
                    format!("{display_label} Records"),
                    entity_logical_name,
                    Some(binding.default_list_view_logical_name().as_str().to_owned()),
                    ChartType::Kpi,
                    ChartAggregation::Count,
                    None,
                    None,
                )?;

                DashboardWidget::new(
                    format!("{entity_logical_name}_widget"),
                    display_label,
                    widget_position,
                    4,
                    3,
                    chart,
                )
            })
            .collect();

        DashboardDefinition::new(dashboard_logical_name, display_name, widgets?)
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

        Self::normalize_sitemap_order(&sitemap)
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

        self.validate_sitemap_targets(actor, input.app_logical_name.as_str(), &input.sitemap)
            .await?;

        let normalized_sitemap = Self::normalize_sitemap_order(&input.sitemap)?;

        self.repository
            .save_sitemap(actor.tenant_id(), normalized_sitemap.clone())
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

        Ok(normalized_sitemap)
    }
}
