use super::*;

pub(super) type ScopeFieldTypes = std::collections::BTreeMap<
    String,
    std::collections::BTreeMap<String, qryvanta_domain::FieldType>,
>;

pub(super) fn normalize_scope_alias(
    scope_alias: Option<String>,
    context: &str,
) -> Result<Option<String>, AppError> {
    match scope_alias {
        Some(alias) => {
            let trimmed = alias.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation(format!(
                    "runtime query {context} scope_alias cannot be empty"
                )));
            }

            Ok(Some(trimmed.to_owned()))
        }
        None => Ok(None),
    }
}
