use qryvanta_core::AppError;

pub(super) const VALIDATION_GENERIC: &str = "validation.generic";
pub(super) const VALIDATION_PUBLISH_CHECKS_FAILED: &str = "validation.publish.checks_failed";
pub(super) const VALIDATION_RUNTIME_PAYLOAD_NOT_OBJECT: &str =
    "validation.runtime.payload.not_object";
pub(super) const VALIDATION_RUNTIME_PAYLOAD_UNKNOWN_FIELD: &str =
    "validation.runtime.payload.unknown_field";
pub(super) const VALIDATION_RUNTIME_PAYLOAD_REQUIRED_FIELD_MISSING: &str =
    "validation.runtime.payload.required_field_missing";
pub(super) const VALIDATION_RUNTIME_PAYLOAD_CALCULATED_FIELD_READ_ONLY: &str =
    "validation.runtime.payload.calculated_field_read_only";
pub(super) const VALIDATION_RUNTIME_RELATION_TARGET_MISSING: &str =
    "validation.runtime.relation.target_missing";
pub(super) const VALIDATION_RUNTIME_BUSINESS_RULE_LOCKED_FIELD: &str =
    "validation.runtime.business_rule.locked_field";
pub(super) const VALIDATION_RUNTIME_QUERY_LIMIT_INVALID: &str =
    "validation.runtime.query.limit_invalid";
pub(super) const VALIDATION_RUNTIME_QUERY_WHERE_EMPTY: &str =
    "validation.runtime.query.where_empty";
pub(super) const VALIDATION_RUNTIME_QUERY_DUPLICATE_SORT_FIELD: &str =
    "validation.runtime.query.duplicate_sort_field";
pub(super) const VALIDATION_RUNTIME_QUERY_ALIAS_EMPTY: &str =
    "validation.runtime.query.alias_empty";
pub(super) const VALIDATION_RUNTIME_QUERY_ALIAS_DUPLICATE: &str =
    "validation.runtime.query.alias_duplicate";
pub(super) const VALIDATION_RUNTIME_QUERY_PARENT_ALIAS_UNKNOWN: &str =
    "validation.runtime.query.parent_alias_unknown";
pub(super) const VALIDATION_RUNTIME_QUERY_PARENT_ALIAS_EMPTY: &str =
    "validation.runtime.query.parent_alias_empty";
pub(super) const VALIDATION_RUNTIME_QUERY_RELATION_FIELD_EMPTY: &str =
    "validation.runtime.query.relation_field_empty";
pub(super) const VALIDATION_RUNTIME_QUERY_SCOPE_ALIAS_UNKNOWN: &str =
    "validation.runtime.query.scope_alias_unknown";
pub(super) const VALIDATION_RUNTIME_QUERY_FIELD_UNKNOWN: &str =
    "validation.runtime.query.field_unknown";
pub(super) const VALIDATION_RUNTIME_QUERY_FIELD_TYPE_MISMATCH: &str =
    "validation.runtime.query.field_type_mismatch";
pub(super) const VALIDATION_RUNTIME_QUERY_OPERATOR_INVALID: &str =
    "validation.runtime.query.operator_invalid";
pub(super) const VALIDATION_RUNTIME_QUERY_SORT_UNSUPPORTED: &str =
    "validation.runtime.query.sort_unsupported";
pub(super) const VALIDATION_RUNTIME_QUERY_LINK_INVALID: &str =
    "validation.runtime.query.link_invalid";
pub(super) const NOT_FOUND: &str = "not_found";
pub(super) const CONFLICT: &str = "conflict";
pub(super) const UNAUTHORIZED: &str = "unauthorized";
pub(super) const FORBIDDEN: &str = "forbidden";
pub(super) const FORBIDDEN_STEP_UP_REQUIRED: &str = "forbidden.step_up_required";
pub(super) const RATE_LIMITED: &str = "rate_limited";
pub(super) const INTERNAL_ERROR: &str = "internal_error";

pub(super) fn error_code_for(error: &AppError) -> &'static str {
    match error {
        AppError::Validation(detail) => validation_code_for(detail.as_str()),
        AppError::NotFound(_) => NOT_FOUND,
        AppError::Conflict(_) => CONFLICT,
        AppError::Unauthorized(_) => UNAUTHORIZED,
        AppError::Forbidden(detail) => forbidden_code_for(detail.as_str()),
        AppError::RateLimited(_) => RATE_LIMITED,
        AppError::Internal(_) => INTERNAL_ERROR,
    }
}

fn forbidden_code_for(detail: &str) -> &'static str {
    if detail == "step-up authentication required for this action" {
        return FORBIDDEN_STEP_UP_REQUIRED;
    }

    FORBIDDEN
}

fn validation_code_for(detail: &str) -> &'static str {
    if detail.starts_with("publish checks failed for entity '") {
        return VALIDATION_PUBLISH_CHECKS_FAILED;
    }

    if detail == "runtime record payload must be a JSON object" {
        return VALIDATION_RUNTIME_PAYLOAD_NOT_OBJECT;
    }
    if detail.starts_with("unknown field '") && detail.contains(" for entity '") {
        return VALIDATION_RUNTIME_PAYLOAD_UNKNOWN_FIELD;
    }
    if detail.starts_with("missing required field '") {
        return VALIDATION_RUNTIME_PAYLOAD_REQUIRED_FIELD_MISSING;
    }
    if detail.starts_with("calculated field '") && detail.ends_with(" cannot be set directly") {
        return VALIDATION_RUNTIME_PAYLOAD_CALCULATED_FIELD_READ_ONLY;
    }
    if detail.starts_with("relation field '") && detail.contains("references missing record") {
        return VALIDATION_RUNTIME_RELATION_TARGET_MISSING;
    }
    if detail.starts_with("business rule lock prevents updating field '") {
        return VALIDATION_RUNTIME_BUSINESS_RULE_LOCKED_FIELD;
    }

    if detail == "runtime record query limit must be greater than zero" {
        return VALIDATION_RUNTIME_QUERY_LIMIT_INVALID;
    }
    if detail == "runtime query where clause must include at least one condition or nested group" {
        return VALIDATION_RUNTIME_QUERY_WHERE_EMPTY;
    }
    if detail.starts_with("duplicate runtime query sort field '") {
        return VALIDATION_RUNTIME_QUERY_DUPLICATE_SORT_FIELD;
    }
    if detail == "runtime query link alias cannot be empty" {
        return VALIDATION_RUNTIME_QUERY_ALIAS_EMPTY;
    }
    if detail.starts_with("duplicate runtime query link alias '") {
        return VALIDATION_RUNTIME_QUERY_ALIAS_DUPLICATE;
    }
    if detail.starts_with("unknown runtime query parent alias '") {
        return VALIDATION_RUNTIME_QUERY_PARENT_ALIAS_UNKNOWN;
    }
    if detail == "runtime query link parent_alias cannot be empty" {
        return VALIDATION_RUNTIME_QUERY_PARENT_ALIAS_EMPTY;
    }
    if detail == "runtime query link relation_field_logical_name cannot be empty" {
        return VALIDATION_RUNTIME_QUERY_RELATION_FIELD_EMPTY;
    }
    if detail.starts_with("unknown runtime query scope alias '") {
        return VALIDATION_RUNTIME_QUERY_SCOPE_ALIAS_UNKNOWN;
    }
    if detail.starts_with("unknown filter field '")
        || detail.starts_with("unknown sort field '")
        || detail.starts_with("unknown relation field '")
    {
        return VALIDATION_RUNTIME_QUERY_FIELD_UNKNOWN;
    }
    if detail.starts_with("query filter field type mismatch for '")
        || detail.starts_with("query sort field type mismatch for '")
    {
        return VALIDATION_RUNTIME_QUERY_FIELD_TYPE_MISMATCH;
    }
    if detail.starts_with("operator '") {
        return VALIDATION_RUNTIME_QUERY_OPERATOR_INVALID;
    }
    if detail.starts_with("sorting is not supported for json field '") {
        return VALIDATION_RUNTIME_QUERY_SORT_UNSUPPORTED;
    }
    if detail.starts_with("link relation field '")
        || detail.starts_with("relation field '")
        || detail.starts_with("runtime query link alias '")
    {
        return VALIDATION_RUNTIME_QUERY_LINK_INVALID;
    }

    VALIDATION_GENERIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_publish_validation_errors() {
        let code = error_code_for(&AppError::Validation(
            "publish checks failed for entity 'contact':\n- entity 'contact' requires at least one field before publishing"
                .to_owned(),
        ));

        assert_eq!(code, VALIDATION_PUBLISH_CHECKS_FAILED);
    }

    #[test]
    fn classifies_runtime_payload_and_query_validation_errors() {
        let payload_code = error_code_for(&AppError::Validation(
            "runtime record payload must be a JSON object".to_owned(),
        ));
        assert_eq!(payload_code, VALIDATION_RUNTIME_PAYLOAD_NOT_OBJECT);

        let query_code = error_code_for(&AppError::Validation(
            "runtime record query limit must be greater than zero".to_owned(),
        ));
        assert_eq!(query_code, VALIDATION_RUNTIME_QUERY_LIMIT_INVALID);
    }

    #[test]
    fn falls_back_to_generic_validation_code() {
        let code = error_code_for(&AppError::Validation(
            "some future validation message".to_owned(),
        ));

        assert_eq!(code, VALIDATION_GENERIC);
    }

    #[test]
    fn classifies_step_up_forbidden_errors() {
        let code = error_code_for(&AppError::Forbidden(
            "step-up authentication required for this action".to_owned(),
        ));

        assert_eq!(code, FORBIDDEN_STEP_UP_REQUIRED);
    }
}
