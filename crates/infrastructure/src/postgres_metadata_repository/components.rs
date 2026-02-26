use super::*;

impl PostgresMetadataRepository {
    pub(super) async fn save_option_set_impl(
        &self,
        tenant_id: TenantId,
        option_set: OptionSetDefinition,
    ) -> AppResult<()> {
        let items_json = serde_json::to_value(option_set.options()).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize option set '{}.{}' items: {error}",
                option_set.entity_logical_name().as_str(),
                option_set.logical_name().as_str()
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO entity_option_sets (
                tenant_id,
                entity_logical_name,
                logical_name,
                display_name,
                items_json,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (tenant_id, entity_logical_name, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                items_json = EXCLUDED.items_json,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(option_set.entity_logical_name().as_str())
        .bind(option_set.logical_name().as_str())
        .bind(option_set.display_name().as_str())
        .bind(items_json)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save option set '{}.{}' in tenant '{}': {error}",
                option_set.entity_logical_name().as_str(),
                option_set.logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_option_sets_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<OptionSetDefinition>> {
        let rows = sqlx::query_as::<_, OptionSetRow>(
            r#"
            SELECT entity_logical_name, logical_name, display_name, items_json
            FROM entity_option_sets
            WHERE tenant_id = $1 AND entity_logical_name = $2
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list option sets for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                let options = serde_json::from_value(row.items_json).map_err(|error| {
                    AppError::Internal(format!(
                        "persisted option set '{}.{}' items are invalid: {error}",
                        row.entity_logical_name, row.logical_name
                    ))
                })?;
                OptionSetDefinition::new(
                    row.entity_logical_name,
                    row.logical_name,
                    row.display_name,
                    options,
                )
            })
            .collect()
    }

    pub(super) async fn find_option_set_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<Option<OptionSetDefinition>> {
        let row = sqlx::query_as::<_, OptionSetRow>(
            r#"
            SELECT entity_logical_name, logical_name, display_name, items_json
            FROM entity_option_sets
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(option_set_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find option set '{}.{}' in tenant '{}': {error}",
                entity_logical_name, option_set_logical_name, tenant_id
            ))
        })?;

        row.map(|row| {
            let options = serde_json::from_value(row.items_json).map_err(|error| {
                AppError::Internal(format!(
                    "persisted option set '{}.{}' items are invalid: {error}",
                    row.entity_logical_name, row.logical_name
                ))
            })?;
            OptionSetDefinition::new(
                row.entity_logical_name,
                row.logical_name,
                row.display_name,
                options,
            )
        })
        .transpose()
    }

    pub(super) async fn delete_option_set_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        option_set_logical_name: &str,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM entity_option_sets
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(option_set_logical_name)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to delete option set '{}.{}' in tenant '{}': {error}",
                entity_logical_name, option_set_logical_name, tenant_id
            ))
        })?;

        if result.rows_affected() == 0 {
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
        let definition_json = serde_json::to_value(&form).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize form '{}.{}': {error}",
                form.entity_logical_name().as_str(),
                form.logical_name().as_str()
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO entity_forms (
                tenant_id,
                entity_logical_name,
                logical_name,
                display_name,
                form_type,
                definition_json,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            ON CONFLICT (tenant_id, entity_logical_name, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                form_type = EXCLUDED.form_type,
                definition_json = EXCLUDED.definition_json,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(form.entity_logical_name().as_str())
        .bind(form.logical_name().as_str())
        .bind(form.display_name().as_str())
        .bind(form.form_type().as_str())
        .bind(definition_json)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save form '{}.{}' in tenant '{}': {error}",
                form.entity_logical_name().as_str(),
                form.logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_forms_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<FormDefinition>> {
        let rows = sqlx::query_as::<_, FormRow>(
            r#"
            SELECT definition_json
            FROM entity_forms
            WHERE tenant_id = $1 AND entity_logical_name = $2
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list forms for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<FormDefinition>(row.definition_json).map_err(|error| {
                    AppError::Internal(format!(
                        "persisted form definition is invalid for entity '{}' in tenant '{}': {error}",
                        entity_logical_name, tenant_id
                    ))
                })
            })
            .collect()
    }

    pub(super) async fn find_form_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<Option<FormDefinition>> {
        let row = sqlx::query_as::<_, FormRow>(
            r#"
            SELECT definition_json
            FROM entity_forms
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(form_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find form '{}.{}' in tenant '{}': {error}",
                entity_logical_name, form_logical_name, tenant_id
            ))
        })?;

        row.map(|row| {
            serde_json::from_value::<FormDefinition>(row.definition_json).map_err(|error| {
                AppError::Internal(format!(
                    "persisted form definition '{}.{}' is invalid in tenant '{}': {error}",
                    entity_logical_name, form_logical_name, tenant_id
                ))
            })
        })
        .transpose()
    }

    pub(super) async fn delete_form_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        form_logical_name: &str,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM entity_forms
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(form_logical_name)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to delete form '{}.{}' in tenant '{}': {error}",
                entity_logical_name, form_logical_name, tenant_id
            ))
        })?;

        if result.rows_affected() == 0 {
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
        let definition_json = serde_json::to_value(&view).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize view '{}.{}': {error}",
                view.entity_logical_name().as_str(),
                view.logical_name().as_str()
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO entity_views (
                tenant_id,
                entity_logical_name,
                logical_name,
                display_name,
                view_type,
                is_default,
                definition_json,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, now())
            ON CONFLICT (tenant_id, entity_logical_name, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                view_type = EXCLUDED.view_type,
                is_default = EXCLUDED.is_default,
                definition_json = EXCLUDED.definition_json,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(view.entity_logical_name().as_str())
        .bind(view.logical_name().as_str())
        .bind(view.display_name().as_str())
        .bind(view.view_type().as_str())
        .bind(view.is_default())
        .bind(definition_json)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save view '{}.{}' in tenant '{}': {error}",
                view.entity_logical_name().as_str(),
                view.logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_views_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<ViewDefinition>> {
        let rows = sqlx::query_as::<_, ViewRow>(
            r#"
            SELECT definition_json
            FROM entity_views
            WHERE tenant_id = $1 AND entity_logical_name = $2
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list views for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<ViewDefinition>(row.definition_json).map_err(|error| {
                    AppError::Internal(format!(
                        "persisted view definition is invalid for entity '{}' in tenant '{}': {error}",
                        entity_logical_name, tenant_id
                    ))
                })
            })
            .collect()
    }

    pub(super) async fn find_view_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<Option<ViewDefinition>> {
        let row = sqlx::query_as::<_, ViewRow>(
            r#"
            SELECT definition_json
            FROM entity_views
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(view_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find view '{}.{}' in tenant '{}': {error}",
                entity_logical_name, view_logical_name, tenant_id
            ))
        })?;

        row.map(|row| {
            serde_json::from_value::<ViewDefinition>(row.definition_json).map_err(|error| {
                AppError::Internal(format!(
                    "persisted view definition '{}.{}' is invalid in tenant '{}': {error}",
                    entity_logical_name, view_logical_name, tenant_id
                ))
            })
        })
        .transpose()
    }

    pub(super) async fn delete_view_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        view_logical_name: &str,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM entity_views
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(view_logical_name)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to delete view '{}.{}' in tenant '{}': {error}",
                entity_logical_name, view_logical_name, tenant_id
            ))
        })?;

        if result.rows_affected() == 0 {
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
        let definition_json = serde_json::to_value(&business_rule).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize business rule '{}.{}': {error}",
                business_rule.entity_logical_name().as_str(),
                business_rule.logical_name().as_str()
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO entity_business_rules (
                tenant_id,
                entity_logical_name,
                logical_name,
                display_name,
                scope,
                definition_json,
                is_active,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, now())
            ON CONFLICT (tenant_id, entity_logical_name, logical_name)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                scope = EXCLUDED.scope,
                definition_json = EXCLUDED.definition_json,
                is_active = EXCLUDED.is_active,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(business_rule.entity_logical_name().as_str())
        .bind(business_rule.logical_name().as_str())
        .bind(business_rule.display_name().as_str())
        .bind(business_rule.scope().as_str())
        .bind(definition_json)
        .bind(business_rule.is_active())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save business rule '{}.{}' in tenant '{}': {error}",
                business_rule.entity_logical_name().as_str(),
                business_rule.logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn list_business_rules_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<BusinessRuleDefinition>> {
        let rows = sqlx::query_as::<_, BusinessRuleRow>(
            r#"
            SELECT definition_json
            FROM entity_business_rules
            WHERE tenant_id = $1 AND entity_logical_name = $2
            ORDER BY logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list business rules for entity '{}' in tenant '{}': {error}",
                entity_logical_name, tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<BusinessRuleDefinition>(row.definition_json).map_err(
                    |error| {
                        AppError::Internal(format!(
                            "persisted business rule definition is invalid for entity '{}' in tenant '{}': {error}",
                            entity_logical_name, tenant_id
                        ))
                    },
                )
            })
            .collect()
    }

    pub(super) async fn find_business_rule_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<Option<BusinessRuleDefinition>> {
        let row = sqlx::query_as::<_, BusinessRuleRow>(
            r#"
            SELECT definition_json
            FROM entity_business_rules
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(business_rule_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find business rule '{}.{}' in tenant '{}': {error}",
                entity_logical_name, business_rule_logical_name, tenant_id
            ))
        })?;

        row.map(|row| {
            serde_json::from_value::<BusinessRuleDefinition>(row.definition_json).map_err(|error| {
                AppError::Internal(format!(
                    "persisted business rule definition '{}.{}' is invalid in tenant '{}': {error}",
                    entity_logical_name, business_rule_logical_name, tenant_id
                ))
            })
        })
        .transpose()
    }

    pub(super) async fn delete_business_rule_impl(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM entity_business_rules
            WHERE tenant_id = $1 AND entity_logical_name = $2 AND logical_name = $3
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(entity_logical_name)
        .bind(business_rule_logical_name)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to delete business rule '{}.{}' in tenant '{}': {error}",
                entity_logical_name, business_rule_logical_name, tenant_id
            ))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "business rule '{}.{}' does not exist for tenant '{}'",
                entity_logical_name, business_rule_logical_name, tenant_id
            )));
        }

        Ok(())
    }
}
