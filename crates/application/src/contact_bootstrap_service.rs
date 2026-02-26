use std::sync::Arc;

use crate::{MetadataRepository, TenantRepository};

const CONTACT_ENTITY_LOGICAL_NAME: &str = "contact";
const CONTACT_ENTITY_DISPLAY_NAME: &str = "Contact";

const SUBJECT_FIELD_LOGICAL_NAME: &str = "subject";
const SUBJECT_FIELD_DISPLAY_NAME: &str = "Subject";

const DISPLAY_NAME_FIELD_LOGICAL_NAME: &str = "display_name";
const DISPLAY_NAME_FIELD_DISPLAY_NAME: &str = "Display Name";

const EMAIL_FIELD_LOGICAL_NAME: &str = "email";
const EMAIL_FIELD_DISPLAY_NAME: &str = "Email";

#[derive(Clone)]
struct ContactFieldSpec {
    logical_name: &'static str,
    display_name: &'static str,
    is_required: bool,
}

const CONTACT_FIELD_SPECS: [ContactFieldSpec; 3] = [
    ContactFieldSpec {
        logical_name: SUBJECT_FIELD_LOGICAL_NAME,
        display_name: SUBJECT_FIELD_DISPLAY_NAME,
        is_required: true,
    },
    ContactFieldSpec {
        logical_name: DISPLAY_NAME_FIELD_LOGICAL_NAME,
        display_name: DISPLAY_NAME_FIELD_DISPLAY_NAME,
        is_required: true,
    },
    ContactFieldSpec {
        logical_name: EMAIL_FIELD_LOGICAL_NAME,
        display_name: EMAIL_FIELD_DISPLAY_NAME,
        is_required: false,
    },
];

/// Ensures a default contact schema exists and maps authenticated subjects to runtime contacts.
#[derive(Clone)]
pub struct ContactBootstrapService {
    metadata_repository: Arc<dyn MetadataRepository>,
    tenant_repository: Arc<dyn TenantRepository>,
}

impl ContactBootstrapService {
    /// Creates a new subject-contact bootstrap service.
    #[must_use]
    pub fn new(
        metadata_repository: Arc<dyn MetadataRepository>,
        tenant_repository: Arc<dyn TenantRepository>,
    ) -> Self {
        Self {
            metadata_repository,
            tenant_repository,
        }
    }
}

mod bootstrap;
mod payload;
mod schema;

#[cfg(test)]
mod tests;
