use std::collections::{HashMap, HashSet};

use super::*;

impl MetadataService {
    pub(super) fn validate_bundle_header(bundle: &WorkspacePortableBundle) -> AppResult<()> {
        if bundle.package_format != PORTABLE_PACKAGE_FORMAT {
            return Err(AppError::Validation(format!(
                "unsupported package_format '{}': expected '{}'",
                bundle.package_format, PORTABLE_PACKAGE_FORMAT
            )));
        }

        if bundle.package_version != PORTABLE_PACKAGE_VERSION {
            return Err(AppError::Validation(format!(
                "unsupported package_version '{}': expected '{}'",
                bundle.package_version, PORTABLE_PACKAGE_VERSION
            )));
        }

        Ok(())
    }

    pub(super) fn validate_bundle_checksum(bundle: &WorkspacePortableBundle) -> AppResult<()> {
        let computed = Self::payload_sha256(&bundle.payload)?;
        if computed != bundle.payload_sha256 {
            return Err(AppError::Validation(format!(
                "bundle checksum mismatch: expected '{}' got '{}'",
                bundle.payload_sha256, computed
            )));
        }

        Ok(())
    }

    pub(super) async fn validate_bundle_payload(
        &self,
        tenant_id: TenantId,
        payload: &WorkspacePortablePayload,
    ) -> AppResult<()> {
        let mut entity_names = HashSet::new();
        let mut runtime_record_ids_by_entity: HashMap<String, HashSet<String>> = HashMap::new();

        for entity_bundle in &payload.entities {
            if !entity_names.insert(entity_bundle.entity_logical_name.clone()) {
                return Err(AppError::Validation(format!(
                    "duplicate entity '{}' in bundle",
                    entity_bundle.entity_logical_name
                )));
            }

            if payload.include_metadata {
                let Some(entity_definition) = &entity_bundle.entity else {
                    return Err(AppError::Validation(format!(
                        "entity '{}' missing metadata definition",
                        entity_bundle.entity_logical_name
                    )));
                };
                if entity_definition.logical_name().as_str() != entity_bundle.entity_logical_name {
                    return Err(AppError::Validation(format!(
                        "entity '{}' metadata logical name mismatch",
                        entity_bundle.entity_logical_name
                    )));
                }
            }

            for field in &entity_bundle.fields {
                if field.entity_logical_name().as_str() != entity_bundle.entity_logical_name {
                    return Err(AppError::Validation(format!(
                        "field '{}.{}' is scoped to wrong entity '{}",
                        entity_bundle.entity_logical_name,
                        field.logical_name().as_str(),
                        field.entity_logical_name().as_str()
                    )));
                }
            }
            for option_set in &entity_bundle.option_sets {
                if option_set.entity_logical_name().as_str() != entity_bundle.entity_logical_name {
                    return Err(AppError::Validation(format!(
                        "option set '{}.{}' is scoped to wrong entity '{}'",
                        entity_bundle.entity_logical_name,
                        option_set.logical_name().as_str(),
                        option_set.entity_logical_name().as_str()
                    )));
                }
            }
            for form in &entity_bundle.forms {
                if form.entity_logical_name().as_str() != entity_bundle.entity_logical_name {
                    return Err(AppError::Validation(format!(
                        "form '{}.{}' is scoped to wrong entity '{}'",
                        entity_bundle.entity_logical_name,
                        form.logical_name().as_str(),
                        form.entity_logical_name().as_str()
                    )));
                }
            }
            for view in &entity_bundle.views {
                if view.entity_logical_name().as_str() != entity_bundle.entity_logical_name {
                    return Err(AppError::Validation(format!(
                        "view '{}.{}' is scoped to wrong entity '{}'",
                        entity_bundle.entity_logical_name,
                        view.logical_name().as_str(),
                        view.entity_logical_name().as_str()
                    )));
                }
            }
            for business_rule in &entity_bundle.business_rules {
                if business_rule.entity_logical_name().as_str() != entity_bundle.entity_logical_name
                {
                    return Err(AppError::Validation(format!(
                        "business rule '{}.{}' is scoped to wrong entity '{}'",
                        entity_bundle.entity_logical_name,
                        business_rule.logical_name().as_str(),
                        business_rule.entity_logical_name().as_str()
                    )));
                }
            }

            let mut record_ids = HashSet::new();
            for runtime_record in &entity_bundle.runtime_records {
                if !runtime_record.data.is_object() {
                    return Err(AppError::Validation(format!(
                        "runtime record '{}.{}' data must be a JSON object",
                        entity_bundle.entity_logical_name, runtime_record.record_id
                    )));
                }
                if !record_ids.insert(runtime_record.record_id.clone()) {
                    return Err(AppError::Validation(format!(
                        "duplicate runtime record id '{}.{}' in bundle",
                        entity_bundle.entity_logical_name, runtime_record.record_id
                    )));
                }
            }
            runtime_record_ids_by_entity
                .insert(entity_bundle.entity_logical_name.clone(), record_ids);
        }

        for entity_bundle in &payload.entities {
            for field in &entity_bundle.fields {
                if field.field_type() != FieldType::Relation {
                    continue;
                }
                let Some(target_entity_logical_name) = field.relation_target_entity() else {
                    continue;
                };

                for runtime_record in &entity_bundle.runtime_records {
                    let Some(data_object) = runtime_record.data.as_object() else {
                        continue;
                    };
                    let Some(value) = data_object.get(field.logical_name().as_str()) else {
                        continue;
                    };
                    let Some(target_record_id) = value.as_str() else {
                        continue;
                    };

                    let target_in_bundle = runtime_record_ids_by_entity
                        .get(target_entity_logical_name.as_str())
                        .map(|ids| ids.contains(target_record_id))
                        .unwrap_or(false);

                    if target_in_bundle {
                        continue;
                    }

                    let target_exists = self
                        .repository
                        .runtime_record_exists(
                            tenant_id,
                            target_entity_logical_name.as_str(),
                            target_record_id,
                        )
                        .await?;

                    if !target_exists {
                        return Err(AppError::Validation(format!(
                            "relation '{}.{}' points to missing record '{}.{}'",
                            entity_bundle.entity_logical_name,
                            field.logical_name().as_str(),
                            target_entity_logical_name.as_str(),
                            target_record_id
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}
