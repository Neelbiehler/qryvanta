use qryvanta_domain::AppEntityAction;

/// Effective subject permissions for an entity in an app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectEntityPermission {
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Read access.
    pub can_read: bool,
    /// Create access.
    pub can_create: bool,
    /// Update access.
    pub can_update: bool,
    /// Delete access.
    pub can_delete: bool,
}

impl SubjectEntityPermission {
    /// Returns whether an action is allowed by this capability.
    #[must_use]
    pub fn allows(&self, action: AppEntityAction) -> bool {
        match action {
            AppEntityAction::Read => self.can_read,
            AppEntityAction::Create => self.can_create,
            AppEntityAction::Update => self.can_update,
            AppEntityAction::Delete => self.can_delete,
        }
    }
}
