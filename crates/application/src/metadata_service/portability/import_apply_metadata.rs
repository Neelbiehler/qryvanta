use super::*;

impl MetadataService {
    pub(super) async fn apply_metadata_import(
        &self,
        actor: &UserIdentity,
        payload: &WorkspacePortablePayload,
    ) -> AppResult<()> {
        for entity_bundle in &payload.entities {
            let Some(entity_definition) = &entity_bundle.entity else {
                continue;
            };

            if self
                .repository
                .find_entity(
                    actor.tenant_id(),
                    entity_bundle.entity_logical_name.as_str(),
                )
                .await?
                .is_some()
            {
                self.update_entity(
                    actor,
                    UpdateEntityInput {
                        logical_name: entity_bundle.entity_logical_name.clone(),
                        display_name: entity_definition.display_name().as_str().to_owned(),
                        description: entity_definition.description().map(ToOwned::to_owned),
                        plural_display_name: entity_definition
                            .plural_display_name()
                            .map(|value| value.as_str().to_owned()),
                        icon: entity_definition.icon().map(ToOwned::to_owned),
                    },
                )
                .await?;
            } else {
                self.register_entity_with_details(
                    actor,
                    entity_bundle.entity_logical_name.clone(),
                    entity_definition.display_name().as_str().to_owned(),
                    entity_definition.description().map(ToOwned::to_owned),
                    entity_definition
                        .plural_display_name()
                        .map(|value| value.as_str().to_owned()),
                    entity_definition.icon().map(ToOwned::to_owned),
                )
                .await?;
            }
        }

        for entity_bundle in &payload.entities {
            for option_set in &entity_bundle.option_sets {
                self.save_option_set(
                    actor,
                    SaveOptionSetInput {
                        entity_logical_name: entity_bundle.entity_logical_name.clone(),
                        logical_name: option_set.logical_name().as_str().to_owned(),
                        display_name: option_set.display_name().as_str().to_owned(),
                        options: option_set.options().to_vec(),
                    },
                )
                .await?;
            }
        }

        for entity_bundle in &payload.entities {
            for field in &entity_bundle.fields {
                self.save_field(
                    actor,
                    SaveFieldInput {
                        entity_logical_name: entity_bundle.entity_logical_name.clone(),
                        logical_name: field.logical_name().as_str().to_owned(),
                        display_name: field.display_name().as_str().to_owned(),
                        field_type: field.field_type(),
                        is_required: field.is_required(),
                        is_unique: field.is_unique(),
                        default_value: field.default_value().cloned(),
                        relation_target_entity: field
                            .relation_target_entity()
                            .map(|value| value.as_str().to_owned()),
                        option_set_logical_name: field
                            .option_set_logical_name()
                            .map(|value| value.as_str().to_owned()),
                        calculation_expression: field
                            .calculation_expression()
                            .map(ToOwned::to_owned),
                    },
                )
                .await?;

                self.update_field(
                    actor,
                    UpdateFieldInput {
                        entity_logical_name: entity_bundle.entity_logical_name.clone(),
                        logical_name: field.logical_name().as_str().to_owned(),
                        display_name: field.display_name().as_str().to_owned(),
                        description: field.description().map(ToOwned::to_owned),
                        default_value: field.default_value().cloned(),
                        calculation_expression: field
                            .calculation_expression()
                            .map(ToOwned::to_owned),
                        max_length: field.max_length(),
                        min_value: field.min_value(),
                        max_value: field.max_value(),
                    },
                )
                .await?;
            }
        }

        let publish_entity_logical_names = payload
            .entities
            .iter()
            .filter(|entity_bundle| {
                entity_bundle.published_schema.is_some()
                    || !entity_bundle.forms.is_empty()
                    || !entity_bundle.views.is_empty()
                    || !entity_bundle.business_rules.is_empty()
                    || !entity_bundle.runtime_records.is_empty()
            })
            .map(|entity_bundle| entity_bundle.entity_logical_name.clone())
            .collect::<Vec<_>>();

        for entity_logical_name in &publish_entity_logical_names {
            self.publish_entity_with_allowed_unpublished_entities(
                actor,
                entity_logical_name.as_str(),
                &publish_entity_logical_names,
            )
            .await?;
        }

        for entity_bundle in &payload.entities {
            for form in &entity_bundle.forms {
                self.save_form(
                    actor,
                    SaveFormInput {
                        entity_logical_name: entity_bundle.entity_logical_name.clone(),
                        logical_name: form.logical_name().as_str().to_owned(),
                        display_name: form.display_name().as_str().to_owned(),
                        form_type: form.form_type(),
                        tabs: form.tabs().to_vec(),
                        header_fields: form
                            .header_fields()
                            .iter()
                            .map(|value| value.as_str().to_owned())
                            .collect(),
                    },
                )
                .await?;
            }

            for view in &entity_bundle.views {
                self.save_view(
                    actor,
                    SaveViewInput {
                        entity_logical_name: entity_bundle.entity_logical_name.clone(),
                        logical_name: view.logical_name().as_str().to_owned(),
                        display_name: view.display_name().as_str().to_owned(),
                        view_type: view.view_type(),
                        columns: view.columns().to_vec(),
                        default_sort: view.default_sort().cloned(),
                        filter_criteria: view.filter_criteria().cloned(),
                        is_default: view.is_default(),
                    },
                )
                .await?;
            }

            for business_rule in &entity_bundle.business_rules {
                self.save_business_rule(
                    actor,
                    SaveBusinessRuleInput {
                        entity_logical_name: entity_bundle.entity_logical_name.clone(),
                        logical_name: business_rule.logical_name().as_str().to_owned(),
                        display_name: business_rule.display_name().as_str().to_owned(),
                        scope: business_rule.scope(),
                        form_logical_name: business_rule
                            .form_logical_name()
                            .map(|value| value.as_str().to_owned()),
                        conditions: business_rule.conditions().to_vec(),
                        actions: business_rule.actions().to_vec(),
                        is_active: business_rule.is_active(),
                    },
                )
                .await?;
            }
        }

        Ok(())
    }
}
