use super::*;

impl PostgresAppRepository {
    pub(super) async fn save_sitemap_impl(
        &self,
        tenant_id: TenantId,
        sitemap: AppSitemap,
    ) -> AppResult<()> {
        let definition_json = serde_json::to_value(&sitemap).map_err(|error| {
            AppError::Internal(format!(
                "failed to serialize sitemap for app '{}' in tenant '{}': {error}",
                sitemap.app_logical_name().as_str(),
                tenant_id
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO app_sitemaps (tenant_id, app_logical_name, definition_json, updated_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (tenant_id, app_logical_name)
            DO UPDATE SET
                definition_json = EXCLUDED.definition_json,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(sitemap.app_logical_name().as_str())
        .bind(definition_json)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to save sitemap for app '{}' in tenant '{}': {error}",
                sitemap.app_logical_name().as_str(),
                tenant_id
            ))
        })?;

        Ok(())
    }

    pub(super) async fn get_sitemap_impl(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppSitemap>> {
        let row = sqlx::query_as::<_, AppSitemapRow>(
            r#"
            SELECT definition_json
            FROM app_sitemaps
            WHERE tenant_id = $1 AND app_logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load sitemap for app '{}' in tenant '{}': {error}",
                app_logical_name, tenant_id
            ))
        })?;

        row.map(|value| {
            serde_json::from_value::<AppSitemap>(value.definition_json).map_err(|error| {
                AppError::Internal(format!(
                    "persisted sitemap for app '{}' in tenant '{}' is invalid: {error}",
                    app_logical_name, tenant_id
                ))
            })
        })
        .transpose()
    }
}
