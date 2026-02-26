use qryvanta_core::{AppError, AppResult, TenantId};

use super::payload::build_contact_payload;
use super::{CONTACT_ENTITY_LOGICAL_NAME, ContactBootstrapService};

impl ContactBootstrapService {
    /// Ensures the tenant has a default `contact` schema and the subject has a mapped contact row.
    pub async fn ensure_subject_contact(
        &self,
        tenant_id: TenantId,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> AppResult<String> {
        if subject.trim().is_empty() {
            return Err(AppError::Validation(
                "subject is required for contact bootstrap".to_owned(),
            ));
        }

        if display_name.trim().is_empty() {
            return Err(AppError::Validation(
                "display_name is required for contact bootstrap".to_owned(),
            ));
        }

        self.ensure_contact_schema(tenant_id, subject).await?;

        if let Some(contact_record_id) = self
            .tenant_repository
            .contact_record_for_subject(tenant_id, subject)
            .await?
            && self
                .metadata_repository
                .runtime_record_exists(
                    tenant_id,
                    CONTACT_ENTITY_LOGICAL_NAME,
                    contact_record_id.as_str(),
                )
                .await?
        {
            return Ok(contact_record_id);
        }

        let payload = build_contact_payload(subject, display_name, email);
        let created_record = self
            .metadata_repository
            .create_runtime_record(
                tenant_id,
                CONTACT_ENTITY_LOGICAL_NAME,
                payload,
                Vec::new(),
                subject,
            )
            .await?;

        let contact_record_id = created_record.record_id().as_str().to_owned();
        self.tenant_repository
            .save_contact_record_for_subject(tenant_id, subject, contact_record_id.as_str())
            .await?;

        Ok(contact_record_id)
    }
}
