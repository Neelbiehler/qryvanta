use std::collections::HashSet;
use std::str::FromStr;

use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};

/// Supported model-driven form types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormType {
    /// Full primary form.
    Main,
    /// Compact create form.
    QuickCreate,
    /// Read-focused side form.
    QuickView,
}

impl FormType {
    /// Returns stable storage value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::QuickCreate => "quick_create",
            Self::QuickView => "quick_view",
        }
    }
}

impl FromStr for FormType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "main" => Ok(Self::Main),
            "quick_create" => Ok(Self::QuickCreate),
            "quick_view" => Ok(Self::QuickView),
            _ => Err(AppError::Validation(format!("unknown form type '{value}'"))),
        }
    }
}

/// Field placement in a section column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormFieldPlacement {
    field_logical_name: NonEmptyString,
    column: i32,
    position: i32,
    visible: bool,
    read_only: bool,
    required_override: Option<bool>,
    label_override: Option<String>,
}

impl FormFieldPlacement {
    /// Creates a validated field placement.
    pub fn new(
        field_logical_name: impl Into<String>,
        column: i32,
        position: i32,
        visible: bool,
        read_only: bool,
        required_override: Option<bool>,
        label_override: Option<String>,
    ) -> AppResult<Self> {
        if column < 0 {
            return Err(AppError::Validation(
                "form field placement column must be greater than or equal to zero".to_owned(),
            ));
        }

        Ok(Self {
            field_logical_name: NonEmptyString::new(field_logical_name)?,
            column,
            position,
            visible,
            read_only,
            required_override,
            label_override: label_override.and_then(|value| {
                let trimmed = value.trim().to_owned();
                (!trimmed.is_empty()).then_some(trimmed)
            }),
        })
    }

    /// Returns field logical name.
    #[must_use]
    pub fn field_logical_name(&self) -> &NonEmptyString {
        &self.field_logical_name
    }

    /// Returns zero-indexed column.
    #[must_use]
    pub fn column(&self) -> i32 {
        self.column
    }
}

/// Section inside a form tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormSection {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    position: i32,
    visible: bool,
    columns: i32,
    fields: Vec<FormFieldPlacement>,
}

impl FormSection {
    /// Creates a validated form section.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        position: i32,
        visible: bool,
        columns: i32,
        fields: Vec<FormFieldPlacement>,
    ) -> AppResult<Self> {
        if !(1..=3).contains(&columns) {
            return Err(AppError::Validation(
                "form section columns must be one of 1, 2, or 3".to_owned(),
            ));
        }

        for field in &fields {
            if field.column() >= columns {
                return Err(AppError::Validation(format!(
                    "field '{}' uses column '{}' but section has '{}' columns",
                    field.field_logical_name().as_str(),
                    field.column(),
                    columns
                )));
            }
        }

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            visible,
            columns,
            fields,
        })
    }

    /// Returns section logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns field placements in this section.
    #[must_use]
    pub fn fields(&self) -> &[FormFieldPlacement] {
        &self.fields
    }
}

/// Tab inside a form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormTab {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    position: i32,
    visible: bool,
    sections: Vec<FormSection>,
}

impl FormTab {
    /// Creates a validated form tab.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        position: i32,
        visible: bool,
        sections: Vec<FormSection>,
    ) -> AppResult<Self> {
        if sections.is_empty() {
            return Err(AppError::Validation(
                "form tabs must include at least one section".to_owned(),
            ));
        }

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            visible,
            sections,
        })
    }

    /// Returns tab logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns sections in this tab.
    #[must_use]
    pub fn sections(&self) -> &[FormSection] {
        &self.sections
    }
}

/// Standalone entity form definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormDefinition {
    entity_logical_name: NonEmptyString,
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    form_type: FormType,
    tabs: Vec<FormTab>,
    header_fields: Vec<String>,
}

impl FormDefinition {
    /// Creates a validated form definition.
    pub fn new(
        entity_logical_name: impl Into<String>,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        form_type: FormType,
        tabs: Vec<FormTab>,
        header_fields: Vec<String>,
    ) -> AppResult<Self> {
        if tabs.is_empty() {
            return Err(AppError::Validation(
                "forms must include at least one tab".to_owned(),
            ));
        }

        if form_type == FormType::QuickCreate {
            if tabs.len() != 1 || tabs[0].sections().len() != 1 {
                return Err(AppError::Validation(
                    "quick_create forms must contain exactly one tab and one section".to_owned(),
                ));
            }
        }

        let mut seen_field_placements = HashSet::new();
        for tab in &tabs {
            for section in tab.sections() {
                for field in section.fields() {
                    if !seen_field_placements.insert(field.field_logical_name().as_str().to_owned())
                    {
                        return Err(AppError::Validation(format!(
                            "duplicate field placement '{}' in form",
                            field.field_logical_name().as_str()
                        )));
                    }
                }
            }
        }

        let mut normalized_header_fields = Vec::with_capacity(header_fields.len());
        let mut seen_header_fields = HashSet::new();
        for field_name in header_fields {
            let trimmed = field_name.trim().to_owned();
            if trimmed.is_empty() {
                return Err(AppError::Validation(
                    "header_fields cannot contain empty field names".to_owned(),
                ));
            }
            if !seen_header_fields.insert(trimmed.clone()) {
                return Err(AppError::Validation(format!(
                    "duplicate header field '{}'",
                    trimmed
                )));
            }
            normalized_header_fields.push(trimmed);
        }

        Ok(Self {
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            form_type,
            tabs,
            header_fields: normalized_header_fields,
        })
    }

    /// Returns parent entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns form logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns form display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns form type.
    #[must_use]
    pub fn form_type(&self) -> FormType {
        self.form_type
    }

    /// Returns tabs.
    #[must_use]
    pub fn tabs(&self) -> &[FormTab] {
        &self.tabs
    }

    /// Returns header fields.
    #[must_use]
    pub fn header_fields(&self) -> &[String] {
        &self.header_fields
    }
}
