use super::*;

impl PostgresAppRepository {
    pub(super) async fn create_app_impl(
        &self,
        tenant_id: TenantId,
        app: AppDefinition,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO app_definitions (tenant_id, logical_name, display_name, description)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app.logical_name().as_str())
        .bind(app.display_name().as_str())
        .bind(app.description())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                if let sqlx::Error::Database(database_error) = &error
                    && database_error.code().as_deref() == Some("23505")
                {
                    return Err(AppError::Conflict(format!(
                        "app '{}' already exists for tenant '{}'",
                        app.logical_name().as_str(),
                        tenant_id
                    )));
                }

                Err(AppError::Internal(format!(
                    "failed to create app '{}' for tenant '{}': {error}",
                    app.logical_name().as_str(),
                    tenant_id
                )))
            }
        }
    }

    pub(super) async fn list_apps_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<Vec<AppDefinition>> {
        let rows = sqlx::query_as::<_, AppRow>(
            r#"
            SELECT logical_name, display_name, description
            FROM app_definitions
            WHERE tenant_id = $1
            ORDER BY display_name, logical_name
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list apps for tenant '{}': {error}",
                tenant_id
            ))
        })?;

        rows.into_iter()
            .map(|row| AppDefinition::new(row.logical_name, row.display_name, row.description))
            .collect()
    }

    pub(super) async fn find_app_impl(
        &self,
        tenant_id: TenantId,
        app_logical_name: &str,
    ) -> AppResult<Option<AppDefinition>> {
        let row = sqlx::query_as::<_, AppRow>(
            r#"
            SELECT logical_name, display_name, description
            FROM app_definitions
            WHERE tenant_id = $1 AND logical_name = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(app_logical_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to find app '{}' for tenant '{}': {error}",
                app_logical_name, tenant_id
            ))
        })?;

        row.map(|value| {
            AppDefinition::new(value.logical_name, value.display_name, value.description)
        })
        .transpose()
    }
}
