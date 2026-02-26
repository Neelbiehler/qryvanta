use super::*;

impl MetadataService {
    pub(super) async fn evaluate_entity_business_rule_effects(
        &self,
        tenant_id: TenantId,
        entity_logical_name: &str,
        normalized_data: &Value,
    ) -> AppResult<EntityBusinessRuleEffects> {
        let rules = self
            .repository
            .list_business_rules(tenant_id, entity_logical_name)
            .await?;

        let mut effects = EntityBusinessRuleEffects::default();
        let normalized_object = normalized_data.as_object();

        for rule in rules {
            if !rule.is_active() || rule.scope() != BusinessRuleScope::Entity {
                continue;
            }

            if !Self::business_rule_matches(&rule, normalized_data) {
                continue;
            }

            for action in rule.actions() {
                match action.action_type() {
                    BusinessRuleActionType::ShowField => {
                        if let Some(target_field) = action.target_field_logical_name() {
                            effects
                                .visibility_overrides
                                .insert(target_field.as_str().to_owned(), true);
                        }
                    }
                    BusinessRuleActionType::HideField => {
                        if let Some(target_field) = action.target_field_logical_name() {
                            effects
                                .visibility_overrides
                                .insert(target_field.as_str().to_owned(), false);
                        }
                    }
                    BusinessRuleActionType::SetRequired => {
                        if let Some(target_field) = action.target_field_logical_name() {
                            effects
                                .required_overrides
                                .insert(target_field.as_str().to_owned(), true);
                        }
                    }
                    BusinessRuleActionType::SetOptional => {
                        if let Some(target_field) = action.target_field_logical_name() {
                            effects
                                .required_overrides
                                .insert(target_field.as_str().to_owned(), false);
                        }
                    }
                    BusinessRuleActionType::SetDefaultValue => {
                        let Some(target_field) = action.target_field_logical_name() else {
                            continue;
                        };
                        let Some(value) = action.value() else {
                            continue;
                        };

                        let is_empty = Self::business_rule_default_target_is_empty(
                            normalized_object.and_then(|object| object.get(target_field.as_str())),
                        );
                        if is_empty {
                            effects
                                .value_patches
                                .insert(target_field.as_str().to_owned(), value.clone());
                        }
                    }
                    BusinessRuleActionType::SetFieldValue => {
                        let Some(target_field) = action.target_field_logical_name() else {
                            continue;
                        };
                        let Some(value) = action.value() else {
                            continue;
                        };

                        effects
                            .value_patches
                            .insert(target_field.as_str().to_owned(), value.clone());
                    }
                    BusinessRuleActionType::LockField => {
                        if let Some(target_field) = action.target_field_logical_name() {
                            effects
                                .lock_overrides
                                .insert(target_field.as_str().to_owned(), true);
                        }
                    }
                    BusinessRuleActionType::UnlockField => {
                        if let Some(target_field) = action.target_field_logical_name() {
                            effects
                                .lock_overrides
                                .insert(target_field.as_str().to_owned(), false);
                        }
                    }
                    BusinessRuleActionType::ShowError => {
                        if let Some(error_message) = action.error_message() {
                            effects
                                .error_messages
                                .push(error_message.as_str().to_owned());
                        }
                    }
                }
            }
        }

        Ok(effects)
    }

    fn business_rule_default_target_is_empty(value: Option<&Value>) -> bool {
        match value {
            None => true,
            Some(Value::Null) => true,
            Some(Value::String(text)) => text.trim().is_empty(),
            Some(_) => false,
        }
    }

    fn business_rule_matches(rule: &BusinessRuleDefinition, normalized_data: &Value) -> bool {
        let Some(data) = normalized_data.as_object() else {
            return false;
        };

        rule.conditions().iter().all(|condition| {
            let value = data
                .get(condition.field_logical_name().as_str())
                .unwrap_or(&Value::Null);

            Self::business_rule_condition_matches(value, condition)
        })
    }

    fn business_rule_condition_matches(value: &Value, condition: &BusinessRuleCondition) -> bool {
        match condition.operator() {
            BusinessRuleOperator::Eq => value == condition.value(),
            BusinessRuleOperator::Neq => value != condition.value(),
            BusinessRuleOperator::Gt => {
                Self::compare_business_rule_values(value, condition.value())
                    .is_some_and(|ordering| ordering.is_gt())
            }
            BusinessRuleOperator::Gte => {
                Self::compare_business_rule_values(value, condition.value()).is_some_and(
                    |ordering| {
                        matches!(
                            ordering,
                            std::cmp::Ordering::Greater | std::cmp::Ordering::Equal
                        )
                    },
                )
            }
            BusinessRuleOperator::Lt => {
                Self::compare_business_rule_values(value, condition.value())
                    .is_some_and(|ordering| ordering.is_lt())
            }
            BusinessRuleOperator::Lte => {
                Self::compare_business_rule_values(value, condition.value()).is_some_and(
                    |ordering| {
                        matches!(
                            ordering,
                            std::cmp::Ordering::Less | std::cmp::Ordering::Equal
                        )
                    },
                )
            }
            BusinessRuleOperator::Contains => {
                let left = value.as_str().unwrap_or_default().to_lowercase();
                let right = condition
                    .value()
                    .as_str()
                    .unwrap_or_default()
                    .to_lowercase();

                left.contains(right.as_str())
            }
        }
    }

    fn compare_business_rule_values(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
        if let (Some(left_number), Some(right_number)) = (
            Self::business_rule_value_as_f64(left),
            Self::business_rule_value_as_f64(right),
        ) {
            return left_number.partial_cmp(&right_number);
        }

        if let (Some(left_text), Some(right_text)) = (left.as_str(), right.as_str()) {
            return Some(left_text.cmp(right_text));
        }

        None
    }

    fn business_rule_value_as_f64(value: &Value) -> Option<f64> {
        if let Some(number) = value.as_f64() {
            return Some(number);
        }

        value.as_str().and_then(|raw| raw.parse::<f64>().ok())
    }
}
