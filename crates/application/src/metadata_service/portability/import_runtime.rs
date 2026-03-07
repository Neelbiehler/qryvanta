use std::collections::HashMap;

use super::*;

impl MetadataService {
    pub(super) async fn plan_runtime_record_import(
        &self,
        tenant_id: TenantId,
        payload: &WorkspacePortablePayload,
        remap_record_ids: bool,
    ) -> AppResult<Vec<PlannedRuntimeRecordImport>> {
        let mut record_id_map: HashMap<(String, String), String> = HashMap::new();
        let mut entity_fields: HashMap<String, HashMap<String, String>> = HashMap::new();

        for entity_bundle in &payload.entities {
            let relation_fields = entity_bundle
                .fields
                .iter()
                .filter(|field| field.field_type() == FieldType::Relation)
                .filter_map(|field| {
                    field.relation_target_entity().map(|target| {
                        (
                            field.logical_name().as_str().to_owned(),
                            target.as_str().to_owned(),
                        )
                    })
                })
                .collect::<HashMap<_, _>>();
            entity_fields.insert(entity_bundle.entity_logical_name.clone(), relation_fields);

            for runtime_record in &entity_bundle.runtime_records {
                let mapped_record_id = if remap_record_ids {
                    Self::deterministic_record_id(
                        tenant_id,
                        entity_bundle.entity_logical_name.as_str(),
                        runtime_record.record_id.as_str(),
                    )
                } else {
                    runtime_record.record_id.clone()
                };
                record_id_map.insert(
                    (
                        entity_bundle.entity_logical_name.clone(),
                        runtime_record.record_id.clone(),
                    ),
                    mapped_record_id,
                );
            }
        }

        let mut plan = Vec::new();

        for entity_bundle in &payload.entities {
            let relation_fields = entity_fields
                .get(entity_bundle.entity_logical_name.as_str())
                .cloned()
                .unwrap_or_default();
            for runtime_record in &entity_bundle.runtime_records {
                let target_record_id = record_id_map
                    .get(&(
                        entity_bundle.entity_logical_name.clone(),
                        runtime_record.record_id.clone(),
                    ))
                    .cloned()
                    .ok_or_else(|| {
                        AppError::Internal(format!(
                            "missing runtime id mapping for '{}.{}'",
                            entity_bundle.entity_logical_name, runtime_record.record_id
                        ))
                    })?;

                let rewritten_data = Self::rewrite_relation_values(
                    runtime_record.data.clone(),
                    &relation_fields,
                    &record_id_map,
                )?;

                let exists = self
                    .repository
                    .find_runtime_record(
                        tenant_id,
                        entity_bundle.entity_logical_name.as_str(),
                        target_record_id.as_str(),
                    )
                    .await?
                    .is_some();

                plan.push(PlannedRuntimeRecordImport {
                    entity_logical_name: entity_bundle.entity_logical_name.clone(),
                    source_record_id: runtime_record.record_id.clone(),
                    target_record_id,
                    rewritten_data,
                    will_create: !exists,
                });
            }
        }

        Ok(plan)
    }

    pub(super) async fn apply_runtime_import(
        &self,
        actor: &UserIdentity,
        runtime_plan: Vec<PlannedRuntimeRecordImport>,
    ) -> AppResult<()> {
        let mut schema_by_entity: HashMap<String, PublishedEntitySchema> = HashMap::new();

        for plan in &runtime_plan {
            if schema_by_entity.contains_key(plan.entity_logical_name.as_str()) {
                continue;
            }

            let schema = self
                .repository
                .latest_published_schema(actor.tenant_id(), plan.entity_logical_name.as_str())
                .await?
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "entity '{}' must be published before runtime import",
                        plan.entity_logical_name
                    ))
                })?;

            schema_by_entity.insert(plan.entity_logical_name.clone(), schema);
        }

        for plan in runtime_plan {
            let schema = schema_by_entity
                .get(plan.entity_logical_name.as_str())
                .ok_or_else(|| {
                    AppError::Internal(format!(
                        "missing published schema cache for entity '{}'",
                        plan.entity_logical_name
                    ))
                })?;

            let unique_values = Self::unique_values_for_record(schema, &plan.rewritten_data)?;

            if plan.will_create {
                let created = self
                    .repository
                    .create_runtime_record_with_id(
                        actor.tenant_id(),
                        plan.entity_logical_name.as_str(),
                        plan.target_record_id.as_str(),
                        plan.rewritten_data,
                        unique_values,
                        actor.subject(),
                        None,
                    )
                    .await?;

                self.audit_repository
                    .append_event(AuditEvent {
                        tenant_id: actor.tenant_id(),
                        subject: actor.subject().to_owned(),
                        action: AuditAction::RuntimeRecordCreated,
                        resource_type: "runtime_record".to_owned(),
                        resource_id: created.record_id().as_str().to_owned(),
                        detail: Some(format!(
                            "imported runtime record '{}' for entity '{}'",
                            created.record_id().as_str(),
                            plan.entity_logical_name
                        )),
                    })
                    .await?;
            } else {
                let updated = self
                    .repository
                    .update_runtime_record(
                        actor.tenant_id(),
                        plan.entity_logical_name.as_str(),
                        plan.target_record_id.as_str(),
                        plan.rewritten_data,
                        unique_values,
                        None,
                    )
                    .await?;

                self.audit_repository
                    .append_event(AuditEvent {
                        tenant_id: actor.tenant_id(),
                        subject: actor.subject().to_owned(),
                        action: AuditAction::RuntimeRecordUpdated,
                        resource_type: "runtime_record".to_owned(),
                        resource_id: updated.record_id().as_str().to_owned(),
                        detail: Some(format!(
                            "imported runtime record update '{}' for entity '{}'",
                            updated.record_id().as_str(),
                            plan.entity_logical_name
                        )),
                    })
                    .await?;
            }
        }

        Ok(())
    }

    pub(super) fn count_relation_rewrites(
        payload: &WorkspacePortablePayload,
        entity_logical_name: &str,
        source_record_id: &str,
        rewritten_data: &Value,
    ) -> usize {
        let Some(entity_bundle) = payload
            .entities
            .iter()
            .find(|entity| entity.entity_logical_name == entity_logical_name)
        else {
            return 0;
        };
        let Some(source_record) = entity_bundle
            .runtime_records
            .iter()
            .find(|record| record.record_id == source_record_id)
        else {
            return 0;
        };

        let Some(source_object) = source_record.data.as_object() else {
            return 0;
        };
        let Some(rewritten_object) = rewritten_data.as_object() else {
            return 0;
        };

        let relation_fields = entity_bundle
            .fields
            .iter()
            .filter_map(|field| {
                (field.field_type() == FieldType::Relation)
                    .then_some(field.logical_name().as_str().to_owned())
            })
            .collect::<Vec<_>>();

        relation_fields
            .iter()
            .filter(|field| source_object.get(*field) != rewritten_object.get(*field))
            .count()
    }
}
