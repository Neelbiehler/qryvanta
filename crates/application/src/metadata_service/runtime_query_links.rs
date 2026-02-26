use super::*;

impl MetadataService {
    pub(super) async fn resolve_runtime_query_links(
        &self,
        actor: &UserIdentity,
        root_entity_logical_name: &str,
        query: &mut RuntimeRecordQuery,
        schema_cache: &mut BTreeMap<String, PublishedEntitySchema>,
    ) -> AppResult<BTreeMap<String, String>> {
        let mut alias_entities = BTreeMap::new();

        for link in &mut query.links {
            if link.alias.trim().is_empty() {
                return Err(AppError::Validation(
                    "runtime query link alias cannot be empty".to_owned(),
                ));
            }

            if alias_entities.contains_key(link.alias.as_str()) {
                return Err(AppError::Validation(format!(
                    "duplicate runtime query link alias '{}'",
                    link.alias
                )));
            }

            let parent_entity_logical_name = match link.parent_alias.as_deref() {
                Some(parent_alias) if !parent_alias.trim().is_empty() => alias_entities
                    .get(parent_alias)
                    .map(String::as_str)
                    .ok_or_else(|| {
                        AppError::Validation(format!(
                            "unknown runtime query parent alias '{}'",
                            parent_alias
                        ))
                    })?,
                Some(_) => {
                    return Err(AppError::Validation(
                        "runtime query link parent_alias cannot be empty".to_owned(),
                    ));
                }
                None => root_entity_logical_name,
            };

            let parent_schema = self
                .load_runtime_query_schema(
                    actor.tenant_id(),
                    parent_entity_logical_name,
                    schema_cache,
                )
                .await?;

            let relation_field_name = link.relation_field_logical_name.trim();
            if relation_field_name.is_empty() {
                return Err(AppError::Validation(
                    "runtime query link relation_field_logical_name cannot be empty".to_owned(),
                ));
            }

            let Some(relation_field) = parent_schema
                .fields()
                .iter()
                .find(|field| field.logical_name().as_str() == relation_field_name)
            else {
                return Err(AppError::Validation(format!(
                    "unknown relation field '{}' for parent entity '{}'",
                    relation_field_name, parent_entity_logical_name
                )));
            };

            if relation_field.field_type() != FieldType::Relation {
                return Err(AppError::Validation(format!(
                    "link relation field '{}' on entity '{}' must be of type 'relation'",
                    relation_field_name, parent_entity_logical_name
                )));
            }

            let Some(target_entity) = relation_field.relation_target_entity() else {
                return Err(AppError::Validation(format!(
                    "relation field '{}' on entity '{}' is missing relation target metadata",
                    relation_field_name, parent_entity_logical_name
                )));
            };

            self.load_runtime_query_schema(actor.tenant_id(), target_entity.as_str(), schema_cache)
                .await?;

            if !link.target_entity_logical_name.is_empty()
                && link.target_entity_logical_name.as_str() != target_entity.as_str()
            {
                return Err(AppError::Validation(format!(
                    "runtime query link alias '{}' target entity mismatch: expected '{}', got '{}'",
                    link.alias,
                    target_entity.as_str(),
                    link.target_entity_logical_name
                )));
            }

            link.target_entity_logical_name = target_entity.as_str().to_owned();
            link.relation_field_logical_name = relation_field_name.to_owned();
            alias_entities.insert(link.alias.clone(), target_entity.as_str().to_owned());
        }

        Ok(alias_entities)
    }

    pub(super) async fn load_runtime_query_schema(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        schema_cache: &mut BTreeMap<String, PublishedEntitySchema>,
    ) -> AppResult<PublishedEntitySchema> {
        if let Some(schema) = schema_cache.get(entity_logical_name) {
            return Ok(schema.clone());
        }

        let schema = self
            .published_schema_for_runtime(tenant_id, entity_logical_name)
            .await?;
        schema_cache.insert(entity_logical_name.to_owned(), schema.clone());
        Ok(schema)
    }

    pub(super) fn resolve_query_field_definition<'a>(
        root_entity_logical_name: &str,
        alias_entities: &BTreeMap<String, String>,
        schema_cache: &'a BTreeMap<String, PublishedEntitySchema>,
        scope_alias: Option<&str>,
        field_logical_name: &str,
        context: &str,
    ) -> AppResult<&'a EntityFieldDefinition> {
        let scope_entity = match scope_alias {
            Some(alias) => alias_entities
                .get(alias)
                .map(String::as_str)
                .ok_or_else(|| {
                    AppError::Validation(format!("unknown runtime query scope alias '{}'", alias))
                })?,
            None => root_entity_logical_name,
        };

        let schema = schema_cache.get(scope_entity).ok_or_else(|| {
            AppError::Internal(format!(
                "runtime query schema cache missing entity '{}'",
                scope_entity
            ))
        })?;

        let field = schema
            .fields()
            .iter()
            .find(|field| field.logical_name().as_str() == field_logical_name)
            .ok_or_else(|| match scope_alias {
                Some(alias) => AppError::Validation(format!(
                    "unknown {} field '{}' for alias '{}'",
                    context, field_logical_name, alias
                )),
                None => AppError::Validation(format!(
                    "unknown {} field '{}' for entity '{}'",
                    context, field_logical_name, root_entity_logical_name
                )),
            })?;

        Ok(field)
    }
}
