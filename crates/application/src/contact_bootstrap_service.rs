use std::collections::HashSet;
use std::sync::Arc;

use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{EntityDefinition, EntityFieldDefinition, FieldType};
use serde_json::{Map, Value};

use crate::{MetadataRepository, TenantRepository};

const CONTACT_ENTITY_LOGICAL_NAME: &str = "contact";
const CONTACT_ENTITY_DISPLAY_NAME: &str = "Contact";

const SUBJECT_FIELD_LOGICAL_NAME: &str = "subject";
const SUBJECT_FIELD_DISPLAY_NAME: &str = "Subject";

const DISPLAY_NAME_FIELD_LOGICAL_NAME: &str = "display_name";
const DISPLAY_NAME_FIELD_DISPLAY_NAME: &str = "Display Name";

const EMAIL_FIELD_LOGICAL_NAME: &str = "email";
const EMAIL_FIELD_DISPLAY_NAME: &str = "Email";

#[derive(Clone)]
struct ContactFieldSpec {
    logical_name: &'static str,
    display_name: &'static str,
    is_required: bool,
}

const CONTACT_FIELD_SPECS: [ContactFieldSpec; 3] = [
    ContactFieldSpec {
        logical_name: SUBJECT_FIELD_LOGICAL_NAME,
        display_name: SUBJECT_FIELD_DISPLAY_NAME,
        is_required: true,
    },
    ContactFieldSpec {
        logical_name: DISPLAY_NAME_FIELD_LOGICAL_NAME,
        display_name: DISPLAY_NAME_FIELD_DISPLAY_NAME,
        is_required: true,
    },
    ContactFieldSpec {
        logical_name: EMAIL_FIELD_LOGICAL_NAME,
        display_name: EMAIL_FIELD_DISPLAY_NAME,
        is_required: false,
    },
];

/// Ensures a default contact schema exists and maps authenticated subjects to runtime contacts.
#[derive(Clone)]
pub struct ContactBootstrapService {
    metadata_repository: Arc<dyn MetadataRepository>,
    tenant_repository: Arc<dyn TenantRepository>,
}

impl ContactBootstrapService {
    /// Creates a new subject-contact bootstrap service.
    #[must_use]
    pub fn new(
        metadata_repository: Arc<dyn MetadataRepository>,
        tenant_repository: Arc<dyn TenantRepository>,
    ) -> Self {
        Self {
            metadata_repository,
            tenant_repository,
        }
    }

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

    async fn ensure_contact_schema(
        &self,
        tenant_id: TenantId,
        published_by_subject: &str,
    ) -> AppResult<()> {
        if self
            .metadata_repository
            .find_entity(tenant_id, CONTACT_ENTITY_LOGICAL_NAME)
            .await?
            .is_none()
        {
            self.metadata_repository
                .save_entity(
                    tenant_id,
                    EntityDefinition::new(
                        CONTACT_ENTITY_LOGICAL_NAME,
                        CONTACT_ENTITY_DISPLAY_NAME,
                    )?,
                )
                .await?;
        }

        let existing_fields = self
            .metadata_repository
            .list_fields(tenant_id, CONTACT_ENTITY_LOGICAL_NAME)
            .await?;
        let existing_field_names: HashSet<String> = existing_fields
            .iter()
            .map(|field| field.logical_name().as_str().to_owned())
            .collect();

        for field_spec in CONTACT_FIELD_SPECS {
            if existing_field_names.contains(field_spec.logical_name) {
                continue;
            }

            self.metadata_repository
                .save_field(
                    tenant_id,
                    EntityFieldDefinition::new(
                        CONTACT_ENTITY_LOGICAL_NAME,
                        field_spec.logical_name,
                        field_spec.display_name,
                        FieldType::Text,
                        field_spec.is_required,
                        false,
                        None,
                        None,
                    )?,
                )
                .await?;
        }

        if self
            .metadata_repository
            .latest_published_schema(tenant_id, CONTACT_ENTITY_LOGICAL_NAME)
            .await?
            .is_some()
        {
            return Ok(());
        }

        let entity = self
            .metadata_repository
            .find_entity(tenant_id, CONTACT_ENTITY_LOGICAL_NAME)
            .await?
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "contact entity is missing in tenant '{}' after bootstrap",
                    tenant_id
                ))
            })?;
        let fields = self
            .metadata_repository
            .list_fields(tenant_id, CONTACT_ENTITY_LOGICAL_NAME)
            .await?;
        let option_sets = self
            .metadata_repository
            .list_option_sets(tenant_id, CONTACT_ENTITY_LOGICAL_NAME)
            .await?;

        if fields.is_empty() {
            return Err(AppError::Validation(
                "contact entity requires at least one field before publishing".to_owned(),
            ));
        }

        self.metadata_repository
            .publish_entity_schema(tenant_id, entity, fields, option_sets, published_by_subject)
            .await?;

        Ok(())
    }
}

fn build_contact_payload(subject: &str, display_name: &str, email: Option<&str>) -> Value {
    let mut payload = Map::new();
    payload.insert(
        SUBJECT_FIELD_LOGICAL_NAME.to_owned(),
        Value::String(subject.to_owned()),
    );
    payload.insert(
        DISPLAY_NAME_FIELD_LOGICAL_NAME.to_owned(),
        Value::String(display_name.to_owned()),
    );
    if let Some(address) = email.filter(|value| !value.trim().is_empty()) {
        payload.insert(
            EMAIL_FIELD_LOGICAL_NAME.to_owned(),
            Value::String(address.to_owned()),
        );
    }

    Value::Object(payload)
}

#[cfg(test)]
mod tests;
