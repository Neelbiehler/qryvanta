use super::*;

impl MetadataService {
    pub(crate) async fn validate_runtime_query(
        &self,
        actor: &UserIdentity,
        root_entity_logical_name: &str,
        root_schema: &PublishedEntitySchema,
        query: &mut RuntimeRecordQuery,
        root_field_access: Option<&crate::RuntimeFieldAccess>,
    ) -> AppResult<()> {
        if query.limit == 0 {
            return Err(AppError::Validation(
                "runtime record query limit must be greater than zero".to_owned(),
            ));
        }

        let mut schema_cache = BTreeMap::new();
        schema_cache.insert(root_entity_logical_name.to_owned(), root_schema.clone());
        let alias_entities = self
            .resolve_runtime_query_links(actor, root_entity_logical_name, query, &mut schema_cache)
            .await?;

        let mut scope_field_access = BTreeMap::new();
        if let Some(access) = root_field_access {
            scope_field_access.insert(String::new(), access.clone());
        }

        let mut entity_field_access_cache = BTreeMap::new();
        for entity_logical_name in alias_entities.values() {
            if entity_field_access_cache.contains_key(entity_logical_name) {
                continue;
            }

            let field_access = self
                .runtime_field_access_for_actor(actor, entity_logical_name)
                .await?;
            entity_field_access_cache.insert(entity_logical_name.clone(), field_access);
        }

        for (alias, entity_logical_name) in &alias_entities {
            let Some(field_access) = entity_field_access_cache
                .get(entity_logical_name)
                .and_then(Option::as_ref)
            else {
                continue;
            };

            scope_field_access.insert(alias.clone(), field_access.clone());
        }

        Self::enforce_query_readable_fields(query, &scope_field_access)?;

        for filter in &query.filters {
            let field = Self::resolve_query_field_definition(
                root_entity_logical_name,
                &alias_entities,
                &schema_cache,
                filter.scope_alias.as_deref(),
                filter.field_logical_name.as_str(),
                "filter",
            )?;
            Self::validate_runtime_query_filter(field, filter)?;
        }

        if let Some(where_clause) = &query.where_clause {
            Self::validate_runtime_query_group(
                root_entity_logical_name,
                &alias_entities,
                &schema_cache,
                where_clause,
            )?;
        }

        let mut seen_sort_fields = BTreeSet::new();
        for sort in &query.sort {
            let sort_scope_key = sort.scope_alias.clone().unwrap_or_default();
            if !seen_sort_fields.insert((sort_scope_key.clone(), sort.field_logical_name.clone())) {
                return Err(AppError::Validation(format!(
                    "duplicate runtime query sort field '{}' in scope '{}'",
                    sort.field_logical_name,
                    if sort_scope_key.is_empty() {
                        root_entity_logical_name
                    } else {
                        sort_scope_key.as_str()
                    }
                )));
            }

            let field = Self::resolve_query_field_definition(
                root_entity_logical_name,
                &alias_entities,
                &schema_cache,
                sort.scope_alias.as_deref(),
                sort.field_logical_name.as_str(),
                "sort",
            )?;
            Self::validate_runtime_query_sort(field, sort)?;
        }

        Ok(())
    }
}
