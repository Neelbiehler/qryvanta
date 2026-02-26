use super::*;

impl MetadataService {
    /// Saves or updates a business rule definition.
    pub async fn save_business_rule(
        &self,
        actor: &UserIdentity,
        input: SaveBusinessRuleInput,
    ) -> AppResult<BusinessRuleDefinition> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;
        self.require_entity_exists(actor.tenant_id(), input.entity_logical_name.as_str())
            .await?;

        let business_rule = BusinessRuleDefinition::new(
            input.entity_logical_name,
            input.logical_name,
            input.display_name,
            BusinessRuleDefinitionInput {
                scope: input.scope,
                form_logical_name: input.form_logical_name,
                conditions: input.conditions,
                actions: input.actions,
                is_active: input.is_active,
            },
        )?;

        if let Some(form_logical_name) = business_rule.form_logical_name() {
            let form_exists = self
                .repository
                .find_form(
                    actor.tenant_id(),
                    business_rule.entity_logical_name().as_str(),
                    form_logical_name.as_str(),
                )
                .await?
                .is_some();
            if !form_exists {
                return Err(AppError::NotFound(format!(
                    "form '{}.{}' does not exist for business rule '{}'",
                    business_rule.entity_logical_name().as_str(),
                    form_logical_name.as_str(),
                    business_rule.logical_name().as_str()
                )));
            }
        }

        let schema = self
            .published_schema_for_runtime(
                actor.tenant_id(),
                business_rule.entity_logical_name().as_str(),
            )
            .await?;

        for condition in business_rule.conditions() {
            let field_exists = schema.fields().iter().any(|field| {
                field.logical_name().as_str() == condition.field_logical_name().as_str()
            });
            if !field_exists {
                return Err(AppError::Validation(format!(
                    "business rule condition references unknown field '{}.{}'",
                    business_rule.entity_logical_name().as_str(),
                    condition.field_logical_name().as_str()
                )));
            }
        }

        for action in business_rule.actions() {
            if let Some(target_field_logical_name) = action.target_field_logical_name() {
                let field_exists = schema.fields().iter().any(|field| {
                    field.logical_name().as_str() == target_field_logical_name.as_str()
                });
                if !field_exists {
                    return Err(AppError::Validation(format!(
                        "business rule action references unknown field '{}.{}'",
                        business_rule.entity_logical_name().as_str(),
                        target_field_logical_name.as_str()
                    )));
                }
            }
        }

        self.repository
            .save_business_rule(actor.tenant_id(), business_rule.clone())
            .await?;
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_business_rule_definition".to_owned(),
                resource_id: format!(
                    "{}.{}",
                    business_rule.entity_logical_name().as_str(),
                    business_rule.logical_name().as_str()
                ),
                detail: Some(format!(
                    "saved business rule '{}' on entity '{}'",
                    business_rule.logical_name().as_str(),
                    business_rule.entity_logical_name().as_str()
                )),
            })
            .await?;

        Ok(business_rule)
    }

    /// Lists business rules for an entity.
    pub async fn list_business_rules(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
    ) -> AppResult<Vec<BusinessRuleDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;

        self.repository
            .list_business_rules(actor.tenant_id(), entity_logical_name)
            .await
    }

    /// Finds a business rule by logical name.
    pub async fn find_business_rule(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<Option<BusinessRuleDefinition>> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldRead,
            )
            .await?;

        self.repository
            .find_business_rule(
                actor.tenant_id(),
                entity_logical_name,
                business_rule_logical_name,
            )
            .await
    }

    /// Deletes a business rule definition.
    pub async fn delete_business_rule(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        business_rule_logical_name: &str,
    ) -> AppResult<()> {
        self.authorization_service
            .require_permission(
                actor.tenant_id(),
                actor.subject(),
                Permission::MetadataFieldWrite,
            )
            .await?;

        self.repository
            .delete_business_rule(
                actor.tenant_id(),
                entity_logical_name,
                business_rule_logical_name,
            )
            .await?;
        self.audit_repository
            .append_event(AuditEvent {
                tenant_id: actor.tenant_id(),
                subject: actor.subject().to_owned(),
                action: AuditAction::MetadataFieldSaved,
                resource_type: "entity_business_rule_definition".to_owned(),
                resource_id: format!("{entity_logical_name}.{business_rule_logical_name}"),
                detail: Some(format!(
                    "deleted business rule '{}' on entity '{}'",
                    business_rule_logical_name, entity_logical_name
                )),
            })
            .await?;

        Ok(())
    }
}
