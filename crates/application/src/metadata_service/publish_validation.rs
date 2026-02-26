use super::*;

impl MetadataService {
    fn published_field_names(schema: &PublishedEntitySchema) -> BTreeSet<String> {
        schema
            .fields()
            .iter()
            .map(|field| field.logical_name().as_str().to_owned())
            .collect()
    }

    pub(super) async fn validate_form_definition(
        &self,
        tenant_id: TenantId,
        schema: &PublishedEntitySchema,
        form: &FormDefinition,
    ) -> AppResult<()> {
        let field_names = Self::published_field_names(schema);
        for header_field in form.header_fields() {
            if !field_names.contains(header_field) {
                return Err(AppError::Validation(format!(
                    "form header field '{}' does not exist in published schema for entity '{}'",
                    header_field,
                    form.entity_logical_name().as_str()
                )));
            }
        }
        for tab in form.tabs() {
            for section in tab.sections() {
                for field in section.fields() {
                    if !field_names.contains(field.field_logical_name().as_str()) {
                        return Err(AppError::Validation(format!(
                            "form field '{}' does not exist in published schema for entity '{}'",
                            field.field_logical_name().as_str(),
                            form.entity_logical_name().as_str()
                        )));
                    }
                }

                for subgrid in section.subgrids() {
                    let target_schema = self
                        .repository
                        .latest_published_schema(
                            tenant_id,
                            subgrid.target_entity_logical_name().as_str(),
                        )
                        .await?
                        .ok_or_else(|| {
                            AppError::Validation(format!(
                                "sub-grid '{}' target entity '{}' does not have a published schema",
                                subgrid.logical_name().as_str(),
                                subgrid.target_entity_logical_name().as_str()
                            ))
                        })?;

                    let relation_field = target_schema
                        .fields()
                        .iter()
                        .find(|field| {
                            field.logical_name().as_str()
                                == subgrid.relation_field_logical_name().as_str()
                        })
                        .ok_or_else(|| {
                            AppError::Validation(format!(
                                "sub-grid '{}' relation field '{}.{}' does not exist",
                                subgrid.logical_name().as_str(),
                                subgrid.target_entity_logical_name().as_str(),
                                subgrid.relation_field_logical_name().as_str()
                            ))
                        })?;

                    if relation_field.field_type() != FieldType::Relation {
                        return Err(AppError::Validation(format!(
                            "sub-grid '{}' relation field '{}.{}' must use relation field type",
                            subgrid.logical_name().as_str(),
                            subgrid.target_entity_logical_name().as_str(),
                            subgrid.relation_field_logical_name().as_str()
                        )));
                    }

                    let Some(relation_target) = relation_field.relation_target_entity() else {
                        return Err(AppError::Validation(format!(
                            "sub-grid '{}' relation field '{}.{}' must define relation target",
                            subgrid.logical_name().as_str(),
                            subgrid.target_entity_logical_name().as_str(),
                            subgrid.relation_field_logical_name().as_str()
                        )));
                    };

                    if relation_target.as_str() != form.entity_logical_name().as_str() {
                        return Err(AppError::Validation(format!(
                            "sub-grid '{}' relation field '{}.{}' must target parent entity '{}', got '{}'",
                            subgrid.logical_name().as_str(),
                            subgrid.target_entity_logical_name().as_str(),
                            subgrid.relation_field_logical_name().as_str(),
                            form.entity_logical_name().as_str(),
                            relation_target.as_str()
                        )));
                    }

                    let target_field_names: BTreeSet<&str> = target_schema
                        .fields()
                        .iter()
                        .map(|field| field.logical_name().as_str())
                        .collect();

                    for column in subgrid.columns() {
                        if !target_field_names.contains(column.as_str()) {
                            return Err(AppError::Validation(format!(
                                "sub-grid '{}' column '{}.{}' does not exist in published schema",
                                subgrid.logical_name().as_str(),
                                subgrid.target_entity_logical_name().as_str(),
                                column
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_view_definition(
        schema: &PublishedEntitySchema,
        view: &ViewDefinition,
    ) -> AppResult<()> {
        let field_names = Self::published_field_names(schema);
        for column in view.columns() {
            if !field_names.contains(column.field_logical_name().as_str()) {
                return Err(AppError::Validation(format!(
                    "view column '{}' does not exist in published schema for entity '{}'",
                    column.field_logical_name().as_str(),
                    view.entity_logical_name().as_str()
                )));
            }
        }
        if let Some(default_sort) = view.default_sort()
            && !field_names.contains(default_sort.field_logical_name().as_str())
        {
            return Err(AppError::Validation(format!(
                "view default sort field '{}' does not exist in published schema for entity '{}'",
                default_sort.field_logical_name().as_str(),
                view.entity_logical_name().as_str()
            )));
        }
        if let Some(filter_group) = view.filter_criteria() {
            for condition in filter_group.conditions() {
                if !field_names.contains(condition.field_logical_name().as_str()) {
                    return Err(AppError::Validation(format!(
                        "view filter field '{}' does not exist in published schema for entity '{}'",
                        condition.field_logical_name().as_str(),
                        view.entity_logical_name().as_str()
                    )));
                }
            }
        }
        Ok(())
    }

    pub(super) async fn collect_publish_validation_errors(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        fields: &[EntityFieldDefinition],
        allowed_unpublished_entity_logical_names: &[String],
    ) -> AppResult<Vec<String>> {
        let mut errors = Vec::new();
        let allowed_unpublished_entities: HashSet<&str> = allowed_unpublished_entity_logical_names
            .iter()
            .map(String::as_str)
            .collect();

        if fields.is_empty() {
            errors.push(format!(
                "entity '{}' requires at least one field before publishing",
                entity_logical_name
            ));
            return Ok(errors);
        }

        let field_names: HashSet<&str> = fields
            .iter()
            .map(|field| field.logical_name().as_str())
            .collect();

        for field in fields {
            if field.field_type() != FieldType::Relation {
                continue;
            }

            let Some(target_entity) = field.relation_target_entity() else {
                errors.push(format!(
                    "relation field '{}' must define relation target entity",
                    field.logical_name().as_str()
                ));
                continue;
            };

            let target_exists = self
                .repository
                .find_entity(tenant_id, target_entity.as_str())
                .await?
                .is_some();
            if !target_exists {
                errors.push(format!(
                    "relation field '{}' target entity '{}' does not exist",
                    field.logical_name().as_str(),
                    target_entity.as_str()
                ));
                continue;
            }

            let has_published_target = self
                .repository
                .latest_published_schema(tenant_id, target_entity.as_str())
                .await?
                .is_some();
            let target_is_self = target_entity.as_str() == entity_logical_name;
            if !has_published_target
                && !target_is_self
                && !allowed_unpublished_entities.contains(target_entity.as_str())
            {
                errors.push(format!(
                    "dependency check failed: entity '{}' relation field '{}' -> entity '{}' requires a published schema or inclusion in this publish selection",
                    entity_logical_name,
                    field.logical_name().as_str(),
                    target_entity.as_str()
                ));
            }
        }

        let forms = self
            .repository
            .list_forms(tenant_id, entity_logical_name)
            .await?;
        for form in &forms {
            for header_field in form.header_fields() {
                if !field_names.contains(header_field.as_str()) {
                    errors.push(format!(
                        "form '{}' header field '{}' does not exist in draft fields",
                        form.logical_name().as_str(),
                        header_field
                    ));
                }
            }

            for tab in form.tabs() {
                for section in tab.sections() {
                    for (field_index, field_placement) in section.fields().iter().enumerate() {
                        if !field_names.contains(field_placement.field_logical_name().as_str()) {
                            errors.push(format!(
                                "form '{}' field placement '{}.{}.{}' references missing draft field '{}'",
                                form.logical_name().as_str(),
                                tab.logical_name().as_str(),
                                section.logical_name().as_str(),
                                field_index,
                                field_placement.field_logical_name().as_str(),
                            ));
                        }
                    }
                }
            }
        }

        let views = self
            .repository
            .list_views(tenant_id, entity_logical_name)
            .await?;
        for view in &views {
            for column in view.columns() {
                if !field_names.contains(column.field_logical_name().as_str()) {
                    errors.push(format!(
                        "view '{}' column '{}' does not exist in draft fields",
                        view.logical_name().as_str(),
                        column.field_logical_name().as_str(),
                    ));
                }
            }

            if let Some(default_sort) = view.default_sort()
                && !field_names.contains(default_sort.field_logical_name().as_str())
            {
                errors.push(format!(
                    "view '{}' default sort field '{}' does not exist in draft fields",
                    view.logical_name().as_str(),
                    default_sort.field_logical_name().as_str(),
                ));
            }

            if let Some(filter_group) = view.filter_criteria() {
                for condition in filter_group.conditions() {
                    if !field_names.contains(condition.field_logical_name().as_str()) {
                        errors.push(format!(
                            "view '{}' filter field '{}' does not exist in draft fields",
                            view.logical_name().as_str(),
                            condition.field_logical_name().as_str(),
                        ));
                    }
                }
            }
        }

        Ok(errors)
    }

    pub(super) fn format_publish_validation_errors(
        entity_logical_name: &str,
        errors: &[String],
    ) -> String {
        let mut message = format!(
            "publish checks failed for entity '{}':",
            entity_logical_name
        );
        for error in errors {
            message.push_str("\n- ");
            message.push_str(error);
        }
        message
    }
}
