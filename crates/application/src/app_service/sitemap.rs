use super::*;

impl AppService {
    pub(crate) fn derive_sitemap_from_bindings(
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

    pub(crate) fn normalize_sitemap_order(sitemap: &AppSitemap) -> AppResult<AppSitemap> {
        let mut sorted_areas = sitemap.areas().to_vec();
        sorted_areas.sort_by(|left, right| {
            left.position().cmp(&right.position()).then_with(|| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            })
        });

        let mut normalized_areas = Vec::with_capacity(sorted_areas.len());
        for area in sorted_areas {
            let mut sorted_groups = area.groups().to_vec();
            sorted_groups.sort_by(|left, right| {
                left.position().cmp(&right.position()).then_with(|| {
                    left.logical_name()
                        .as_str()
                        .cmp(right.logical_name().as_str())
                })
            });

            let mut normalized_groups = Vec::with_capacity(sorted_groups.len());
            for group in sorted_groups {
                let mut sorted_sub_areas = group.sub_areas().to_vec();
                sorted_sub_areas.sort_by(|left, right| {
                    left.position().cmp(&right.position()).then_with(|| {
                        left.logical_name()
                            .as_str()
                            .cmp(right.logical_name().as_str())
                    })
                });

                normalized_groups.push(SitemapGroup::new(
                    group.logical_name().as_str(),
                    group.display_name().as_str(),
                    group.position(),
                    sorted_sub_areas,
                )?);
            }

            normalized_areas.push(SitemapArea::new(
                area.logical_name().as_str(),
                area.display_name().as_str(),
                area.position(),
                area.icon().map(ToOwned::to_owned),
                normalized_groups,
            )?);
        }

        AppSitemap::new(sitemap.app_logical_name().as_str(), normalized_areas)
    }

    pub(crate) fn filter_sitemap_by_permissions(
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
