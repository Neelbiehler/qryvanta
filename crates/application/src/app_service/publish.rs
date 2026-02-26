use std::collections::HashSet;

use super::*;

impl AppService {
    /// Runs app-level publish checks without mutating metadata.
    pub async fn publish_checks(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
    ) -> AppResult<Vec<String>> {
        self.publish_checks_with_allowed_unpublished_entities(actor, app_logical_name, &[])
            .await
    }

    /// Runs app-level publish checks while allowing selected unpublished entities
    /// to satisfy app dependency validation in the same publish transaction.
    pub async fn publish_checks_with_allowed_unpublished_entities(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        allowed_unpublished_entity_logical_names: &[String],
    ) -> AppResult<Vec<String>> {
        self.require_admin(actor).await?;
        self.require_app_exists(actor.tenant_id(), app_logical_name)
            .await?;

        let allowed_unpublished_entities: HashSet<&str> = allowed_unpublished_entity_logical_names
            .iter()
            .map(String::as_str)
            .collect();

        let bindings = self
            .repository
            .list_app_entity_bindings(actor.tenant_id(), app_logical_name)
            .await?;

        let mut errors = Vec::new();
        if bindings.is_empty() {
            errors.push(format!("app '{}' has no entity bindings", app_logical_name));
        }

        for binding in &bindings {
            let entity_logical_name = binding.entity_logical_name().as_str();

            let published_schema = self
                .runtime_record_service
                .latest_published_schema_unchecked(actor, entity_logical_name)
                .await?;
            if published_schema.is_none()
                && !allowed_unpublished_entities.contains(entity_logical_name)
            {
                errors.push(format!(
                    "dependency check failed: app '{}' -> entity '{}' requires a published schema or inclusion in this publish selection",
                    app_logical_name, entity_logical_name
                ));
            }

            let default_form = binding.default_form_logical_name().as_str();
            if self
                .runtime_record_service
                .find_form_unchecked(actor, entity_logical_name, default_form)
                .await?
                .is_none()
            {
                errors.push(format!(
                    "app entity '{}' default form '{}' was not found",
                    entity_logical_name, default_form
                ));
            }

            let default_view = binding.default_list_view_logical_name().as_str();
            if self
                .runtime_record_service
                .find_view_unchecked(actor, entity_logical_name, default_view)
                .await?
                .is_none()
            {
                errors.push(format!(
                    "app entity '{}' default view '{}' was not found",
                    entity_logical_name, default_view
                ));
            }
        }

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

        errors.extend(Self::collect_sitemap_structure_errors(&sitemap));

        errors.extend(
            self.collect_sitemap_target_errors(actor, app_logical_name, &sitemap, &bindings)
                .await?,
        );

        Ok(errors)
    }

    pub(super) async fn validate_sitemap_targets(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        sitemap: &AppSitemap,
    ) -> AppResult<()> {
        let mut errors = Self::collect_sitemap_structure_errors(sitemap);

        let bindings = self
            .repository
            .list_app_entity_bindings(actor.tenant_id(), app_logical_name)
            .await?;
        errors.extend(
            self.collect_sitemap_target_errors(actor, app_logical_name, sitemap, &bindings)
                .await?,
        );
        if let Some(first_error) = errors.into_iter().next() {
            return Err(AppError::Validation(first_error));
        }

        Ok(())
    }

    pub(super) fn collect_sitemap_structure_errors(sitemap: &AppSitemap) -> Vec<String> {
        let mut errors = Vec::new();

        let mut area_names = HashSet::new();
        let mut area_positions = HashSet::new();
        let mut area_positions_are_valid = true;
        for area in sitemap.areas() {
            if area.position() < 0 {
                errors.push(format!(
                    "sitemap area '{}' has negative position '{}'",
                    area.logical_name().as_str(),
                    area.position()
                ));
                area_positions_are_valid = false;
            }

            if !area_names.insert(area.logical_name().as_str().to_owned()) {
                errors.push(format!(
                    "duplicate sitemap area logical name '{}'",
                    area.logical_name().as_str()
                ));
            }

            if !area_positions.insert(area.position()) {
                errors.push(format!(
                    "duplicate sitemap area position '{}'",
                    area.position()
                ));
                area_positions_are_valid = false;
            }

            let mut group_names = HashSet::new();
            let mut group_positions = HashSet::new();
            let mut group_positions_are_valid = true;
            for group in area.groups() {
                if group.position() < 0 {
                    errors.push(format!(
                        "sitemap group '{}.{}' has negative position '{}'",
                        area.logical_name().as_str(),
                        group.logical_name().as_str(),
                        group.position()
                    ));
                    group_positions_are_valid = false;
                }

                if !group_names.insert(group.logical_name().as_str().to_owned()) {
                    errors.push(format!(
                        "duplicate sitemap group logical name '{}' in area '{}'",
                        group.logical_name().as_str(),
                        area.logical_name().as_str()
                    ));
                }

                if !group_positions.insert(group.position()) {
                    errors.push(format!(
                        "duplicate sitemap group position '{}' in area '{}'",
                        group.position(),
                        area.logical_name().as_str()
                    ));
                    group_positions_are_valid = false;
                }

                let mut sub_area_names = HashSet::new();
                let mut sub_area_positions = HashSet::new();
                let mut sub_area_positions_are_valid = true;
                for sub_area in group.sub_areas() {
                    if sub_area.position() < 0 {
                        errors.push(format!(
                            "sitemap sub area '{}.{}.{}' has negative position '{}'",
                            area.logical_name().as_str(),
                            group.logical_name().as_str(),
                            sub_area.logical_name().as_str(),
                            sub_area.position()
                        ));
                        sub_area_positions_are_valid = false;
                    }

                    if !sub_area_names.insert(sub_area.logical_name().as_str().to_owned()) {
                        errors.push(format!(
                            "duplicate sitemap sub area logical name '{}' in group '{}.{}'",
                            sub_area.logical_name().as_str(),
                            area.logical_name().as_str(),
                            group.logical_name().as_str()
                        ));
                    }

                    if !sub_area_positions.insert(sub_area.position()) {
                        errors.push(format!(
                            "duplicate sitemap sub area position '{}' in group '{}.{}'",
                            sub_area.position(),
                            area.logical_name().as_str(),
                            group.logical_name().as_str()
                        ));
                        sub_area_positions_are_valid = false;
                    }
                }

                if sub_area_positions_are_valid
                    && !Self::positions_are_contiguous(&sub_area_positions)
                {
                    errors.push(format!(
                        "sitemap sub area positions in group '{}.{}' must form contiguous sequence starting at zero",
                        area.logical_name().as_str(),
                        group.logical_name().as_str()
                    ));
                }
            }

            if group_positions_are_valid && !Self::positions_are_contiguous(&group_positions) {
                errors.push(format!(
                    "sitemap group positions in area '{}' must form contiguous sequence starting at zero",
                    area.logical_name().as_str()
                ));
            }
        }

        if area_positions_are_valid && !Self::positions_are_contiguous(&area_positions) {
            errors.push(
                "sitemap area positions must form contiguous sequence starting at zero".to_owned(),
            );
        }

        errors
    }

    fn positions_are_contiguous(positions: &HashSet<i32>) -> bool {
        let mut sorted_positions: Vec<i32> = positions.iter().copied().collect();
        sorted_positions.sort_unstable();

        sorted_positions
            .iter()
            .enumerate()
            .all(|(index, position)| {
                let Ok(expected) = i32::try_from(index) else {
                    return false;
                };
                *position == expected
            })
    }

    async fn collect_sitemap_target_errors(
        &self,
        actor: &UserIdentity,
        app_logical_name: &str,
        sitemap: &AppSitemap,
        bindings: &[AppEntityBinding],
    ) -> AppResult<Vec<String>> {
        let mut errors = Vec::new();
        let bound_entities: HashSet<&str> = bindings
            .iter()
            .map(|binding| binding.entity_logical_name().as_str())
            .collect();

        for area in sitemap.areas() {
            for group in area.groups() {
                for sub_area in group.sub_areas() {
                    let SitemapTarget::Entity {
                        entity_logical_name,
                        default_form,
                        default_view,
                    } = sub_area.target()
                    else {
                        continue;
                    };

                    if !bound_entities.contains(entity_logical_name.as_str()) {
                        errors.push(format!(
                            "sitemap target '{}.{}' references unbound entity '{}' in app '{}'",
                            group.logical_name().as_str(),
                            sub_area.logical_name().as_str(),
                            entity_logical_name,
                            app_logical_name,
                        ));
                    }

                    if let Some(form_logical_name) = default_form
                        && self
                            .runtime_record_service
                            .find_form_unchecked(
                                actor,
                                entity_logical_name.as_str(),
                                form_logical_name.as_str(),
                            )
                            .await?
                            .is_none()
                    {
                        errors.push(format!(
                            "sitemap target '{}.{}' default form '{}' was not found for entity '{}'",
                            group.logical_name().as_str(),
                            sub_area.logical_name().as_str(),
                            form_logical_name,
                            entity_logical_name
                        ));
                    }

                    if let Some(view_logical_name) = default_view
                        && self
                            .runtime_record_service
                            .find_view_unchecked(
                                actor,
                                entity_logical_name.as_str(),
                                view_logical_name.as_str(),
                            )
                            .await?
                            .is_none()
                    {
                        errors.push(format!(
                            "sitemap target '{}.{}' default view '{}' was not found for entity '{}'",
                            group.logical_name().as_str(),
                            sub_area.logical_name().as_str(),
                            view_logical_name,
                            entity_logical_name
                        ));
                    }
                }
            }
        }

        Ok(errors)
    }
}
