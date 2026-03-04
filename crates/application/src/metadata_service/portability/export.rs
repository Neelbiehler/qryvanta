use super::*;

impl MetadataService {
    /// Exports a deterministic workspace portability package.
    pub async fn export_workspace_bundle(
        &self,
        actor: &UserIdentity,
        options: ExportWorkspaceBundleOptions,
    ) -> AppResult<WorkspacePortableBundle> {
        if !options.include_metadata && !options.include_runtime_data {
            return Err(AppError::Validation(
                "at least one of include_metadata/include_runtime_data must be true".to_owned(),
            ));
        }

        if options.include_metadata {
            self.authorization_service
                .require_permission(
                    actor.tenant_id(),
                    actor.subject(),
                    Permission::MetadataEntityRead,
                )
                .await?;
            self.authorization_service
                .require_permission(
                    actor.tenant_id(),
                    actor.subject(),
                    Permission::MetadataFieldRead,
                )
                .await?;
        }

        if options.include_runtime_data {
            self.authorization_service
                .require_permission(
                    actor.tenant_id(),
                    actor.subject(),
                    Permission::RuntimeRecordRead,
                )
                .await?;
        }

        let entities = self.repository.list_entities(actor.tenant_id()).await?;
        let mut payload_entities = Vec::with_capacity(entities.len());

        for entity in entities {
            let entity_logical_name = entity.logical_name().as_str().to_owned();

            let mut fields = if options.include_metadata {
                self.repository
                    .list_fields(actor.tenant_id(), entity_logical_name.as_str())
                    .await?
            } else {
                Vec::new()
            };
            fields.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });

            let mut option_sets = if options.include_metadata {
                self.repository
                    .list_option_sets(actor.tenant_id(), entity_logical_name.as_str())
                    .await?
            } else {
                Vec::new()
            };
            option_sets.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });

            let mut forms = if options.include_metadata {
                self.repository
                    .list_forms(actor.tenant_id(), entity_logical_name.as_str())
                    .await?
            } else {
                Vec::new()
            };
            forms.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });

            let mut views = if options.include_metadata {
                self.repository
                    .list_views(actor.tenant_id(), entity_logical_name.as_str())
                    .await?
            } else {
                Vec::new()
            };
            views.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });

            let mut business_rules = if options.include_metadata {
                self.repository
                    .list_business_rules(actor.tenant_id(), entity_logical_name.as_str())
                    .await?
            } else {
                Vec::new()
            };
            business_rules.sort_by(|left, right| {
                left.logical_name()
                    .as_str()
                    .cmp(right.logical_name().as_str())
            });

            let published_schema = if options.include_metadata {
                self.repository
                    .latest_published_schema(actor.tenant_id(), entity_logical_name.as_str())
                    .await?
            } else {
                None
            };

            let mut runtime_records = if options.include_runtime_data {
                self.list_all_runtime_records_for_export(
                    actor.tenant_id(),
                    entity_logical_name.as_str(),
                )
                .await?
                .into_iter()
                .map(|record| PortableRuntimeRecord {
                    record_id: record.record_id().as_str().to_owned(),
                    data: Self::canonicalize_json_value(record.data().clone()),
                })
                .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            runtime_records.sort_by(|left, right| left.record_id.cmp(&right.record_id));

            payload_entities.push(PortableEntityBundle {
                entity_logical_name,
                entity: options.include_metadata.then_some(entity),
                fields,
                option_sets,
                forms,
                views,
                business_rules,
                published_schema,
                runtime_records,
            });
        }

        payload_entities
            .sort_by(|left, right| left.entity_logical_name.cmp(&right.entity_logical_name));

        let payload = WorkspacePortablePayload {
            tenant_id: actor.tenant_id().to_string(),
            entities: payload_entities,
            include_metadata: options.include_metadata,
            include_runtime_data: options.include_runtime_data,
        };

        let payload_sha256 = Self::payload_sha256(&payload)?;

        Ok(WorkspacePortableBundle {
            package_format: PORTABLE_PACKAGE_FORMAT.to_owned(),
            package_version: PORTABLE_PACKAGE_VERSION,
            exported_at: chrono::Utc::now(),
            payload_sha256,
            payload,
        })
    }

    async fn list_all_runtime_records_for_export(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
    ) -> AppResult<Vec<RuntimeRecord>> {
        let mut offset = 0_usize;
        let page_limit = 200_usize;
        let mut records = Vec::new();

        loop {
            let page = self
                .repository
                .list_runtime_records(
                    tenant_id,
                    entity_logical_name,
                    RecordListQuery {
                        limit: page_limit,
                        offset,
                        owner_subject: None,
                    },
                )
                .await?;

            if page.is_empty() {
                break;
            }

            offset = offset.saturating_add(page.len());
            records.extend(page);

            if records.len() % page_limit != 0 {
                break;
            }
        }

        records.sort_by(|left, right| left.record_id().as_str().cmp(right.record_id().as_str()));

        Ok(records)
    }
}
