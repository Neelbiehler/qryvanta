use super::*;

use super::scope::{ScopeFieldTypes, normalize_scope_alias};

pub(super) async fn runtime_record_links_from_request(
    metadata_service: &qryvanta_application::MetadataService,
    actor: &UserIdentity,
    root_entity_logical_name: &str,
    root_schema: &qryvanta_domain::PublishedEntitySchema,
    link_entities: Option<Vec<crate::dto::RuntimeRecordQueryLinkEntityRequest>>,
    scope_field_types: &mut ScopeFieldTypes,
) -> Result<Vec<qryvanta_application::RuntimeRecordLink>, AppError> {
    let mut links = Vec::new();
    let mut scope_entities = std::collections::BTreeMap::new();
    scope_entities.insert(String::new(), root_entity_logical_name.to_owned());
    let mut schema_cache = std::collections::BTreeMap::new();
    schema_cache.insert(root_entity_logical_name.to_owned(), root_schema.clone());

    for entry in link_entities.unwrap_or_default() {
        let alias = entry.alias.trim().to_owned();
        if alias.is_empty() {
            return Err(AppError::Validation(
                "runtime query link alias cannot be empty".to_owned(),
            ));
        }

        if alias == "root" {
            return Err(AppError::Validation(
                "runtime query link alias 'root' is reserved".to_owned(),
            ));
        }

        if scope_entities.contains_key(alias.as_str()) {
            return Err(AppError::Validation(format!(
                "duplicate runtime query link alias '{}'",
                alias
            )));
        }

        let parent_alias = normalize_scope_alias(entry.parent_alias, "link")?;
        let parent_scope_key = parent_alias.clone().unwrap_or_default();
        let Some(parent_entity_logical_name) = scope_entities.get(parent_scope_key.as_str()) else {
            return Err(AppError::Validation(format!(
                "unknown runtime query parent alias '{}'",
                parent_scope_key
            )));
        };

        let parent_schema = load_published_schema_for_entity(
            metadata_service,
            actor,
            parent_entity_logical_name.as_str(),
            &mut schema_cache,
        )
        .await?;

        let relation_field_name = entry.relation_field_logical_name.trim().to_owned();
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
                relation_field_name,
                parent_schema.entity().logical_name().as_str()
            )));
        };

        if relation_field.field_type() != qryvanta_domain::FieldType::Relation {
            return Err(AppError::Validation(format!(
                "link relation field '{}' on entity '{}' must be of type 'relation'",
                relation_field_name,
                parent_schema.entity().logical_name().as_str()
            )));
        }

        let Some(target_entity) = relation_field.relation_target_entity() else {
            return Err(AppError::Validation(format!(
                "relation field '{}' on entity '{}' is missing relation target metadata",
                relation_field_name,
                parent_schema.entity().logical_name().as_str()
            )));
        };

        let join_type = entry
            .join_type
            .as_deref()
            .map(qryvanta_application::RuntimeRecordJoinType::parse_transport)
            .transpose()?
            .unwrap_or(qryvanta_application::RuntimeRecordJoinType::Inner);

        let target_schema = load_published_schema_for_entity(
            metadata_service,
            actor,
            target_entity.as_str(),
            &mut schema_cache,
        )
        .await?;

        scope_field_types.insert(
            alias.clone(),
            target_schema
                .fields()
                .iter()
                .map(|field| (field.logical_name().as_str().to_owned(), field.field_type()))
                .collect(),
        );

        links.push(qryvanta_application::RuntimeRecordLink {
            alias: alias.clone(),
            parent_alias,
            relation_field_logical_name: relation_field_name,
            target_entity_logical_name: target_entity.as_str().to_owned(),
            join_type,
        });
        scope_entities.insert(alias, target_entity.as_str().to_owned());
    }

    Ok(links)
}

async fn load_published_schema_for_entity(
    metadata_service: &qryvanta_application::MetadataService,
    actor: &UserIdentity,
    entity_logical_name: &str,
    schema_cache: &mut std::collections::BTreeMap<String, qryvanta_domain::PublishedEntitySchema>,
) -> Result<qryvanta_domain::PublishedEntitySchema, AppError> {
    if let Some(schema) = schema_cache.get(entity_logical_name) {
        return Ok(schema.clone());
    }

    let schema = metadata_service
        .latest_published_schema_unchecked(actor, entity_logical_name)
        .await?
        .ok_or_else(|| {
            AppError::Validation(format!(
                "entity '{}' must be published before runtime query links can be used",
                entity_logical_name
            ))
        })?;

    schema_cache.insert(entity_logical_name.to_owned(), schema.clone());
    Ok(schema)
}
