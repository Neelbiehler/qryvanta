use super::*;

impl MetadataService {
    /// Imports a workspace portability package.
    pub async fn import_workspace_bundle(
        &self,
        actor: &UserIdentity,
        bundle: WorkspacePortableBundle,
        options: ImportWorkspaceBundleOptions,
    ) -> AppResult<ImportWorkspaceBundleResult> {
        if !options.import_metadata && !options.import_runtime_data {
            return Err(AppError::Validation(
                "at least one of import_metadata/import_runtime_data must be true".to_owned(),
            ));
        }

        if options.import_metadata {
            self.authorization_service
                .require_permission(
                    actor.tenant_id(),
                    actor.subject(),
                    Permission::MetadataEntityCreate,
                )
                .await?;
            self.authorization_service
                .require_permission(
                    actor.tenant_id(),
                    actor.subject(),
                    Permission::MetadataFieldWrite,
                )
                .await?;
        }

        if options.import_runtime_data {
            self.authorization_service
                .require_permission(
                    actor.tenant_id(),
                    actor.subject(),
                    Permission::RuntimeRecordWrite,
                )
                .await?;
        }

        Self::validate_bundle_header(&bundle)?;
        Self::validate_bundle_checksum(&bundle)?;
        self.validate_bundle_payload(actor.tenant_id(), &bundle.payload)
            .await?;

        if options.import_metadata && !bundle.payload.include_metadata {
            return Err(AppError::Validation(
                "bundle does not include metadata content".to_owned(),
            ));
        }
        if options.import_runtime_data && !bundle.payload.include_runtime_data {
            return Err(AppError::Validation(
                "bundle does not include runtime data content".to_owned(),
            ));
        }
        if options.remap_record_ids && !bundle.payload.include_metadata {
            return Err(AppError::Validation(
                "remap_record_ids requires metadata content in the bundle".to_owned(),
            ));
        }

        let runtime_plan = if options.import_runtime_data {
            self.plan_runtime_record_import(
                actor.tenant_id(),
                &bundle.payload,
                options.remap_record_ids,
            )
            .await?
        } else {
            Vec::new()
        };

        let runtime_records_discovered = bundle
            .payload
            .entities
            .iter()
            .map(|entity| entity.runtime_records.len())
            .sum::<usize>();
        let runtime_records_created = runtime_plan.iter().filter(|item| item.will_create).count();
        let runtime_records_updated = runtime_plan.len().saturating_sub(runtime_records_created);
        let runtime_records_remapped = runtime_plan
            .iter()
            .filter(|item| item.source_record_id != item.target_record_id)
            .count();
        let relation_rewrites = runtime_plan
            .iter()
            .map(|item| {
                Self::count_relation_rewrites(
                    &bundle.payload,
                    item.entity_logical_name.as_str(),
                    item.source_record_id.as_str(),
                    &item.rewritten_data,
                )
            })
            .sum();

        if options.dry_run {
            return Ok(ImportWorkspaceBundleResult {
                dry_run: true,
                entities_processed: bundle.payload.entities.len(),
                runtime_records_discovered,
                runtime_records_created,
                runtime_records_updated,
                runtime_records_remapped,
                relation_rewrites,
            });
        }

        if options.import_metadata {
            self.apply_metadata_import(actor, &bundle.payload).await?;
        }

        if options.import_runtime_data {
            self.apply_runtime_import(actor, runtime_plan).await?;
        }

        Ok(ImportWorkspaceBundleResult {
            dry_run: false,
            entities_processed: bundle.payload.entities.len(),
            runtime_records_discovered,
            runtime_records_created,
            runtime_records_updated,
            runtime_records_remapped,
            relation_rewrites,
        })
    }
}
