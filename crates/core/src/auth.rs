use serde::{Deserialize, Serialize};

use crate::TenantId;

/// User information persisted in the authenticated session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserIdentity {
    subject: String,
    display_name: String,
    email: Option<String>,
    tenant_id: TenantId,
}

impl UserIdentity {
    /// Creates a user identity from authentication and tenancy data.
    #[must_use]
    pub fn new(
        subject: impl Into<String>,
        display_name: impl Into<String>,
        email: Option<String>,
        tenant_id: TenantId,
    ) -> Self {
        Self {
            subject: subject.into(),
            display_name: display_name.into(),
            email,
            tenant_id,
        }
    }

    /// Returns the stable subject claim from the identity provider.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Returns the display name for the current user.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.display_name.as_str()
    }

    /// Returns the email, if the provider returned one.
    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Returns the tenant linked to the identity.
    #[must_use]
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }
}
