use qryvanta_core::{AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};

/// Metadata definition for a business entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityDefinition {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
}

impl EntityDefinition {
    /// Creates a new entity definition with validated fields.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
        })
    }

    /// Returns the logical (stable) name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns the display (human-friendly) name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }
}

#[cfg(test)]
mod tests {
    use super::EntityDefinition;

    #[test]
    fn entity_requires_non_empty_fields() {
        let result = EntityDefinition::new("", "Contact");
        assert!(result.is_err());
    }
}
