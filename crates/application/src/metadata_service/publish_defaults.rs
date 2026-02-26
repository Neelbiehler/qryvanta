use super::*;

impl MetadataService {
    /// Auto-generates a default main form if none exists for the entity.
    pub(super) async fn auto_generate_default_form(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        fields: &[EntityFieldDefinition],
    ) -> AppResult<()> {
        let existing_forms = self
            .repository
            .list_forms(tenant_id, entity_logical_name)
            .await?;

        let has_main_form = existing_forms
            .iter()
            .any(|form| form.form_type() == FormType::Main);
        if has_main_form {
            return Ok(());
        }

        let placements: Vec<FormFieldPlacement> = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let position = i32::try_from(index).unwrap_or(i32::MAX);
                FormFieldPlacement::new(
                    field.logical_name().as_str(),
                    0,
                    position,
                    true,
                    false,
                    None,
                    None,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let section = FormSection::new("general", "General", 0, true, 2, placements, vec![])?;
        let tab = FormTab::new("general", "General", 0, true, vec![section])?;
        let form = FormDefinition::new(
            entity_logical_name,
            "main_form",
            "Main Form",
            FormType::Main,
            vec![tab],
            Vec::new(),
        )?;

        self.repository.save_form(tenant_id, form).await?;
        Ok(())
    }

    /// Auto-generates a default "All Records" grid view if none exists for the entity.
    pub(super) async fn auto_generate_default_view(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        fields: &[EntityFieldDefinition],
    ) -> AppResult<()> {
        let existing_views = self
            .repository
            .list_views(tenant_id, entity_logical_name)
            .await?;

        let has_default_view = existing_views.iter().any(|view| view.is_default());
        if has_default_view {
            return Ok(());
        }

        let columns: Vec<ViewColumn> = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let position = i32::try_from(index).unwrap_or(i32::MAX);
                ViewColumn::new(field.logical_name().as_str(), position, None, None)
            })
            .collect::<Result<Vec<_>, _>>()?;

        if columns.is_empty() {
            return Ok(());
        }

        let default_sort = fields
            .first()
            .map(|field| ViewSort::new(field.logical_name().as_str(), SortDirection::Asc))
            .transpose()?;

        let view = ViewDefinition::new(
            entity_logical_name,
            "all_records",
            "All Records",
            ViewType::Grid,
            columns,
            default_sort,
            None,
            true,
        )?;

        self.repository.save_view(tenant_id, view).await?;
        Ok(())
    }
}
