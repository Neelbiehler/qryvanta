use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};

/// Metadata-driven application definition used to group worker experiences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDefinition {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    description: Option<String>,
}

impl AppDefinition {
    /// Creates a validated app definition.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        description: Option<String>,
    ) -> AppResult<Self> {
        let description = description.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            description,
        })
    }

    /// Returns the stable app logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns the app display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns an optional app description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Entity navigation binding inside an app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEntityBinding {
    app_logical_name: NonEmptyString,
    entity_logical_name: NonEmptyString,
    navigation_label: Option<String>,
    navigation_order: i32,
}

impl AppEntityBinding {
    /// Creates a validated app entity binding.
    pub fn new(
        app_logical_name: impl Into<String>,
        entity_logical_name: impl Into<String>,
        navigation_label: Option<String>,
        navigation_order: i32,
    ) -> AppResult<Self> {
        if navigation_order < 0 {
            return Err(AppError::Validation(
                "navigation_order must be greater than or equal to zero".to_owned(),
            ));
        }

        let navigation_label = navigation_label.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(Self {
            app_logical_name: NonEmptyString::new(app_logical_name)?,
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            navigation_label,
            navigation_order,
        })
    }

    /// Returns the parent app logical name.
    #[must_use]
    pub fn app_logical_name(&self) -> &NonEmptyString {
        &self.app_logical_name
    }

    /// Returns the bound entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns optional navigation label override.
    #[must_use]
    pub fn navigation_label(&self) -> Option<&str> {
        self.navigation_label.as_deref()
    }

    /// Returns navigation ordering value.
    #[must_use]
    pub fn navigation_order(&self) -> i32 {
        self.navigation_order
    }
}

/// App-scoped entity action permissions assigned to a role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEntityRolePermission {
    app_logical_name: NonEmptyString,
    role_name: NonEmptyString,
    entity_logical_name: NonEmptyString,
    can_read: bool,
    can_create: bool,
    can_update: bool,
    can_delete: bool,
}

impl AppEntityRolePermission {
    /// Creates a validated app-scoped role permission entry.
    pub fn new(
        app_logical_name: impl Into<String>,
        role_name: impl Into<String>,
        entity_logical_name: impl Into<String>,
        can_read: bool,
        can_create: bool,
        can_update: bool,
        can_delete: bool,
    ) -> AppResult<Self> {
        Ok(Self {
            app_logical_name: NonEmptyString::new(app_logical_name)?,
            role_name: NonEmptyString::new(role_name)?,
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            can_read,
            can_create,
            can_update,
            can_delete,
        })
    }

    /// Returns app logical name.
    #[must_use]
    pub fn app_logical_name(&self) -> &NonEmptyString {
        &self.app_logical_name
    }

    /// Returns role name.
    #[must_use]
    pub fn role_name(&self) -> &NonEmptyString {
        &self.role_name
    }

    /// Returns entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns whether the role can read records.
    #[must_use]
    pub fn can_read(&self) -> bool {
        self.can_read
    }

    /// Returns whether the role can create records.
    #[must_use]
    pub fn can_create(&self) -> bool {
        self.can_create
    }

    /// Returns whether the role can update records.
    #[must_use]
    pub fn can_update(&self) -> bool {
        self.can_update
    }

    /// Returns whether the role can delete records.
    #[must_use]
    pub fn can_delete(&self) -> bool {
        self.can_delete
    }
}

/// Runtime action applied to app-scoped records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEntityAction {
    /// Read record/list operations.
    Read,
    /// Create operation.
    Create,
    /// Update operation.
    Update,
    /// Delete operation.
    Delete,
}

impl AppEntityAction {
    /// Returns stable action name for error messages.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppDefinition, AppEntityBinding};

    #[test]
    fn app_definition_requires_non_empty_values() {
        let app = AppDefinition::new("", "Sales", None);
        assert!(app.is_err());
    }

    #[test]
    fn app_entity_binding_rejects_negative_navigation_order() {
        let binding = AppEntityBinding::new("sales", "account", None, -1);
        assert!(binding.is_err());
    }
}
