use qryvanta_core::{AppError, TenantId};

use crate::api_config::PhysicalIsolationMode;

pub(super) fn parse_physical_isolation_mode(
    value: &str,
) -> Result<PhysicalIsolationMode, AppError> {
    if value.eq_ignore_ascii_case("shared") {
        return Ok(PhysicalIsolationMode::Shared);
    }

    if value.eq_ignore_ascii_case("tenant_per_schema") {
        return Ok(PhysicalIsolationMode::TenantPerSchema);
    }

    if value.eq_ignore_ascii_case("tenant_per_database") {
        return Ok(PhysicalIsolationMode::TenantPerDatabase);
    }

    Err(AppError::Validation(format!(
        "PHYSICAL_ISOLATION_MODE must be one of 'shared', 'tenant_per_schema', or 'tenant_per_database', got '{value}'"
    )))
}

pub(super) fn validate_physical_isolation_config(
    mode: PhysicalIsolationMode,
    tenant_id: Option<TenantId>,
    schema_template: Option<&str>,
    database_url_template: Option<&str>,
) -> Result<(), AppError> {
    match mode {
        PhysicalIsolationMode::Shared => Ok(()),
        PhysicalIsolationMode::TenantPerSchema => {
            if tenant_id.is_none() {
                return Err(AppError::Validation(
                    "PHYSICAL_ISOLATION_TENANT_ID is required when PHYSICAL_ISOLATION_MODE=tenant_per_schema"
                        .to_owned(),
                ));
            }

            let template = schema_template.ok_or_else(|| {
                AppError::Validation(
                    "PHYSICAL_ISOLATION_SCHEMA_TEMPLATE is required when PHYSICAL_ISOLATION_MODE=tenant_per_schema"
                        .to_owned(),
                )
            })?;
            validate_tenant_template("PHYSICAL_ISOLATION_SCHEMA_TEMPLATE", template)
        }
        PhysicalIsolationMode::TenantPerDatabase => {
            if tenant_id.is_none() {
                return Err(AppError::Validation(
                    "PHYSICAL_ISOLATION_TENANT_ID is required when PHYSICAL_ISOLATION_MODE=tenant_per_database"
                        .to_owned(),
                ));
            }

            let template = database_url_template.ok_or_else(|| {
                AppError::Validation(
                    "PHYSICAL_ISOLATION_DATABASE_URL_TEMPLATE is required when PHYSICAL_ISOLATION_MODE=tenant_per_database"
                        .to_owned(),
                )
            })?;
            validate_tenant_template("PHYSICAL_ISOLATION_DATABASE_URL_TEMPLATE", template)
        }
    }
}

fn validate_tenant_template(name: &str, value: &str) -> Result<(), AppError> {
    if !value.contains("{tenant_id}") {
        return Err(AppError::Validation(format!(
            "{name} must include '{{tenant_id}}' placeholder"
        )));
    }

    Ok(())
}
