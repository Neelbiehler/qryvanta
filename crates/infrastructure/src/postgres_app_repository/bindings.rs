use super::*;

impl PostgresAppRepository {
    pub(super) async fn save_app_entity_binding_impl(
        &self,
        tenant_id: TenantId,
        binding: AppEntityBinding,
    ) -> AppResult<()> {
        let forms = Json(
            binding
                .forms()
                .iter()
                .map(|form| AppEntityFormDocument {
                    logical_name: form.logical_name().as_str().to_owned(),
                    display_name: form.display_name().as_str().to_owned(),
                    field_logical_names: form.field_logical_names().to_vec(),
                })
                .collect::<Vec<_>>(),
        );
        let list_views = Json(
            binding
                .list_views()
                .iter()
                .map(|view| AppEntityViewDocument {
                    logical_name: view.logical_name().as_str().to_owned(),
                    display_name: view.display_name().as_str().to_owned(),
                    field_logical_names: view.field_logical_names().to_vec(),
                })
                .collect::<Vec<_>>(),
        );
        let default_view_mode = binding.default_view_mode().as_str();

        sqlx::query(
            r#"
            INSERT INTO app_entity_bindings (
                tenant_id,
                app_logical_name,
                entity_logical_name,
                navigation_label,
                navigation_order,
                forms,
                list_views,
                default_form_logical_name,
                default_list_view_logical_name,
                default_view_mode,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (tenant_id, app_logical_name, entity_logical_name)
            DO UPDATE SET
                navigation_label = EXCLUDED.navigation_label,
                navigation_order = EXCLUDED.navigation_order,
                forms = EXCLUDED.forms,
                list_views = EXCLUDED.list_views,
                default_form_logical_name = EXCLUDED.default_form_logical_name,
                default_list_view_logical_name = EXCLUDED.default_list_view_logical_name,
                default_view_mode = EXCLUDED.default_view_mode,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(binding.app_logical_name().as_str())
        .bind(binding.entity_logical_name().as_str())
        .bind(binding.navigation_label())
        .bind(binding.navigation_order())
        .bind(forms)
        .bind(list_views)
        .bind(binding.default_form_logical_name().as_str())
        .bind(binding.default_list_view_logical_name().as_str())
        .bind(default_view_mode)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save app entity binding '{}.{}' for tenant '{}': {error}",
                binding.app_logical_name().as_str(),
                binding.entity_logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_app_entity_bindings_impl(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Vec<AppEntityBinding>> {
        let rows = sqlx::query_as::<_, AppEntityBindingRow>(
            r#"
            SELECT
                app_logical_name,
                entity_logical_name,
                navigation_label,
                navigation_order,
                COALESCE(
                    NULLIF(forms, '[]'::jsonb),
                    jsonb_build_array(
                        jsonb_build_object(
                            'logical_name', 'main_form',
                            'display_name', 'Main Form',
                            'field_logical_names', form_field_logical_names
                        )
                    )
                ) AS forms,
                COALESCE(
                    NULLIF(list_views, '[]'::jsonb),
                    jsonb_build_array(
                        jsonb_build_object(
                            'logical_name', 'main_view',
                            'display_name', 'Main View',
                            'field_logical_names', list_field_logical_names
                        )
                    )
                ) AS list_views,
                COALESCE(default_form_logical_name, 'main_form') AS default_form_logical_name,
                COALESCE(default_list_view_logical_name, 'main_view') AS default_list_view_logical_name,
                default_view_mode
            FROM app_entity_bindings
            WHERE tenant_id = $1 AND app_logical_name = $2
            ORDER BY navigation_order, entity_logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list app entity bindings for app '{}' and tenant '{}': {error}",
                app_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                AppEntityBinding::new(
                    row.app_logical_name,
                    row.entity_logical_name,
                    row.navigation_label,
                    row.navigation_order,
                    row.forms
                        .0
                        .into_iter()
                        .map(|form| {
                            AppEntityForm::new(
                                form.logical_name,
                                form.display_name,
                                form.field_logical_names,
                            )
                        })
                        .collect::<AppResult<Vec<_>>>()?,
                    row.list_views
                        .0
                        .into_iter()
                        .map(|view| {
                            AppEntityView::new(
                                view.logical_name,
                                view.display_name,
                                view.field_logical_names,
                            )
                        })
                        .collect::<AppResult<Vec<_>>>()?,
                    row.default_form_logical_name,
                    row.default_list_view_logical_name,
                    app_entity_view_mode_from_str(row.default_view_mode.as_str())?,
                )
            })
            .collect()
    }
}
