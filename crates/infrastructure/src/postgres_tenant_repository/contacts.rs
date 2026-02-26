use qryvanta_core::AppError;
use uuid::Uuid;

use super::*;

impl PostgresTenantRepository {
    pub(super) async fn contact_record_for_subject_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
    ) -> AppResult<Option<String>> {
        let record_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT contact_record_id
            FROM tenant_subject_contacts
            WHERE tenant_id = $1 AND subject = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to resolve contact mapping for tenant '{}' and subject '{}': {error}",
                tenant_id, subject
            ))
        })?;

        Ok(record_id.map(|value| value.to_string()))
    }

    pub(super) async fn save_contact_record_for_subject_impl(
        &self,
        tenant_id: TenantId,
        subject: &str,
        contact_record_id: &str,
    ) -> AppResult<()> {
        let contact_record_uuid = parse_contact_record_uuid(contact_record_id)?;

        let is_tenant_contact_record = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM runtime_records
                WHERE tenant_id = $1
                  AND entity_logical_name = 'contact'
                  AND id = $2
            )
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(contact_record_uuid)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to validate contact record '{}' in tenant '{}': {error}",
                contact_record_id, tenant_id
            ))
        })?;

        if !is_tenant_contact_record {
            return Err(AppError::NotFound(format!(
                "contact runtime record '{}' does not exist in tenant '{}'",
                contact_record_id, tenant_id
            )));
        }

        sqlx::query(
            r#"
            INSERT INTO tenant_subject_contacts (tenant_id, subject, contact_record_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (tenant_id, subject)
            DO UPDATE SET
                contact_record_id = EXCLUDED.contact_record_id,
                updated_at = now()
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(subject)
        .bind(contact_record_uuid)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to persist contact mapping for tenant '{}' and subject '{}': {error}",
                tenant_id, subject
            ))
        })?;

        Ok(())
    }
}

fn parse_contact_record_uuid(contact_record_id: &str) -> AppResult<Uuid> {
    Uuid::parse_str(contact_record_id).map_err(|error| {
        AppError::Validation(format!(
            "invalid contact record id '{}': {error}",
            contact_record_id
        ))
    })
}
