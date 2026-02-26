use std::collections::HashSet;

use qryvanta_core::{AppError, AppResult, TenantId};
use qryvanta_domain::{EntityDefinition, EntityFieldDefinition, FieldType};

use super::{
    CONTACT_ENTITY_DISPLAY_NAME, CONTACT_ENTITY_LOGICAL_NAME, CONTACT_FIELD_SPECS,
    ContactBootstrapService,
};

impl ContactBootstrapService {
    pub(super) async fn ensure_contact_schema(
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
