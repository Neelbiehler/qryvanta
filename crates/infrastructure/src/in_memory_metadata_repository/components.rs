use super::*;

impl InMemoryMetadataRepository {
    pub(super) async fn save_option_set_impl(
        &self,
        tenant_id: TenantId,
        option_set: OptionSetDefinition,
    ) -> AppResult<()> {
        self.option_sets.write().await.insert(
            (
                tenant_id,
                option_set.entity_logical_name().as_str().to_owned(),
                option_set.logical_name().as_str().to_owned(),
            ),
            option_set,
        );
        Ok(())
    }

    pub(super) async fn list_option_sets_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<OptionSetDefinition>> {
        let option_sets = self.option_sets.read().await;
        let mut listed: Vec<OptionSetDefinition> = option_sets
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), option_set)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(option_set.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    pub(super) async fn find_option_set_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<Option<OptionSetDefinition>> {
        Ok(self
            .option_sets
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                option_set_logical_name.to_owned(),
            ))
            .cloned())
    }

    pub(super) async fn delete_option_set_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.option_sets.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            option_set_logical_name.to_owned(),
        ));
        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "option set '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, option_set_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    pub(super) async fn save_form_impl(
        &self,
        tenant_id: TenantId,
        form: FormDefinition,
    ) -> AppResult<()> {
        self.forms.write().await.insert(
            (
                tenant_id,
                form.entity_logical_name().as_str().to_owned(),
                form.logical_name().as_str().to_owned(),
            ),
            form,
        );
        Ok(())
    }

    pub(super) async fn list_forms_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        let forms = self.forms.read().await;
        let mut listed: Vec<FormDefinition> = forms
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), form)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(form.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    pub(super) async fn find_form_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        Ok(self
            .forms
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                form_logical_name.to_owned(),
            ))
            .cloned())
    }

    pub(super) async fn delete_form_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.forms.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            form_logical_name.to_owned(),
        ));
        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "form '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, form_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    pub(super) async fn save_view_impl(
        &self,
        tenant_id: TenantId,
        view: ViewDefinition,
    ) -> AppResult<()> {
        self.views.write().await.insert(
            (
                tenant_id,
                view.entity_logical_name().as_str().to_owned(),
                view.logical_name().as_str().to_owned(),
            ),
            view,
        );
        Ok(())
    }

    pub(super) async fn list_views_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        let views = self.views.read().await;
        let mut listed: Vec<ViewDefinition> = views
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), view)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(view.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    pub(super) async fn find_view_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        Ok(self
            .views
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                view_logical_name.to_owned(),
            ))
            .cloned())
    }

    pub(super) async fn delete_view_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.views.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            view_logical_name.to_owned(),
        ));
        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "view '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, view_logical_name, tenant_id
            )));
        }
        Ok(())
    }

    pub(super) async fn save_business_rule_impl(
        &self,
        tenant_id: TenantId,
        business_rule: BusinessRuleDefinition,
    ) -> AppResult<()> {
        self.business_rules.write().await.insert(
            (
                tenant_id,
                business_rule.entity_logical_name().as_str().to_owned(),
                business_rule.logical_name().as_str().to_owned(),
            ),
            business_rule,
        );
        Ok(())
    }

    pub(super) async fn list_business_rules_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<BusinessRuleDefinition>> {
        let rules = self.business_rules.read().await;
        let mut listed: Vec<BusinessRuleDefinition> = rules
            .iter()
            .filter_map(|((stored_tenant_id, stored_entity_name, _), rule)| {
                (stored_tenant_id == &tenant_id && stored_entity_name == entity_logical_name)
                    .then_some(rule.clone())
            })
            .collect();
        listed.sort_by(|left, right| {
            left.logical_name()
                .as_str()
                .cmp(right.logical_name().as_str())
        });
        Ok(listed)
    }

    pub(super) async fn find_business_rule_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<Option<BusinessRuleDefinition>> {
        Ok(self
            .business_rules
            .read()
            .await
            .get(&(
                tenant_id,
                entity_logical_name.to_owned(),
                business_rule_logical_name.to_owned(),
            ))
            .cloned())
    }

    pub(super) async fn delete_business_rule_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<()> {
        let removed = self.business_rules.write().await.remove(&(
            tenant_id,
            entity_logical_name.to_owned(),
            business_rule_logical_name.to_owned(),
        ));
        if removed.is_none() {
            return Err(AppError::NotFound(format!(
                "business rule '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, business_rule_logical_name, tenant_id
            )));
        }
        Ok(())
    }
}
