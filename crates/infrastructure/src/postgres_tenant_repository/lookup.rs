use qryvanta_core::AppError;

use super::*;
use crate::postgres_tenant_rls::begin_membership_subject_lookup_transaction;

impl PostgresTenantRepository {
    pub(super) async fn find_tenant_for_subject_impl(
        &self,
        subject: &str,
    ) -> AppResult<Option<TenantId>> {
        let mut transaction =
            begin_membership_subject_lookup_transaction(&self.pool, subject).await?;
        let tenant_id: Option<uuid::Uuid> = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            SELECT tenant_id
            FROM tenant_memberships
            WHERE subject = $1
            LIMIT 1
            "#,
        )
        .bind(subject)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to resolve tenant membership: {error}"))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit subject-scoped membership lookup transaction: {error}"
            ))
        })?;

        Ok(tenant_id.map(TenantId::from_uuid))
    }

    pub(super) async fn list_memberships_for_subject_impl(
        &self,
        subject: &str,
    ) -> AppResult<Vec<qryvanta_application::TenantMembership>> {
        #[derive(sqlx::FromRow)]
        struct MembershipRow {
            tenant_id: uuid::Uuid,
            tenant_name: String,
            display_name: String,
            email: Option<String>,
        }

        let mut transaction =
            begin_membership_subject_lookup_transaction(&self.pool, subject).await?;
        let rows = sqlx::query_as::<_, MembershipRow>(
            r#"
            SELECT
                memberships.tenant_id,
                tenants.name AS tenant_name,
                memberships.display_name,
                memberships.email
            FROM tenant_memberships memberships
            INNER JOIN tenants
                ON tenants.id = memberships.tenant_id
            WHERE memberships.subject = $1
            ORDER BY LOWER(tenants.name), memberships.tenant_id
            "#,
        )
        .bind(subject)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to list subject tenant memberships: {error}"
            ))
        })?;
        transaction.commit().await.map_err(|error| {
            AppError::Internal(format!(
                "failed to commit subject membership list transaction: {error}"
            ))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| qryvanta_application::TenantMembership {
                tenant_id: TenantId::from_uuid(row.tenant_id),
                tenant_name: row.tenant_name,
                display_name: row.display_name,
                email: row.email,
            })
            .collect())
    }

    pub(super) async fn registration_mode_for_tenant_impl(
        &self,
        tenant_id: TenantId,
    ) -> AppResult<RegistrationMode> {
        let stored_mode = sqlx::query_scalar::<_, String>(
            r#"
            SELECT registration_mode
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve tenant registration mode: {error}"
            ))
        })?
        .ok_or_else(|| AppError::NotFound(format!("tenant '{}' not found", tenant_id)))?;

        RegistrationMode::parse(stored_mode.as_str())
    }
}
