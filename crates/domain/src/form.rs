use std::collections::{BTreeMap, HashSet};
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

/// Related-record sub-grid control configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormSubgrid {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    target_entity_logical_name: NonEmptyString,
    relation_field_logical_name: NonEmptyString,
    position: i32,
    #[serde(default)]
    columns: Vec<String>,
}

impl FormSubgrid {
    /// Creates a validated sub-grid control.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        target_entity_logical_name: impl Into<String>,
        relation_field_logical_name: impl Into<String>,
        position: i32,
        columns: Vec<String>,
    ) -> AppResult<Self> {
        if position < 0 {
            return Err(AppError::Validation(
                "sub-grid position must be greater than or equal to zero".to_owned(),
            ));
        }

        let mut normalized_columns = Vec::with_capacity(columns.len());
        let mut seen_columns = HashSet::new();
        for column in columns {
            let trimmed = column.trim().to_owned();
            if trimmed.is_empty() {
                continue;
            }
            if seen_columns.insert(trimmed.clone()) {
                normalized_columns.push(trimmed);
            }
        }

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            target_entity_logical_name: NonEmptyString::new(target_entity_logical_name)?,
            relation_field_logical_name: NonEmptyString::new(relation_field_logical_name)?,
            position,
            columns: normalized_columns,
        })
    }

    /// Returns sub-grid logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns target entity logical name.
    #[must_use]
    pub fn target_entity_logical_name(&self) -> &NonEmptyString {
        &self.target_entity_logical_name
    }

    /// Returns child relation field logical name.
    #[must_use]
    pub fn relation_field_logical_name(&self) -> &NonEmptyString {
        &self.relation_field_logical_name
    }

    /// Returns configured column list.
    #[must_use]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
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

        if position < 0 {
            return Err(AppError::Validation(
                "form field placement position must be greater than or equal to zero".to_owned(),
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
    #[serde(default)]
    subgrids: Vec<FormSubgrid>,
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
        subgrids: Vec<FormSubgrid>,
    ) -> AppResult<Self> {
        if position < 0 {
            return Err(AppError::Validation(
                "form section position must be greater than or equal to zero".to_owned(),
            ));
        }

        if !(1..=3).contains(&columns) {
            return Err(AppError::Validation(
                "form section columns must be one of 1, 2, or 3".to_owned(),
            ));
        }

        let mut field_positions_by_column: BTreeMap<i32, HashSet<i32>> = BTreeMap::new();
        for field in &fields {
            if field.column() >= columns {
                return Err(AppError::Validation(format!(
                    "field '{}' uses column '{}' but section has '{}' columns",
                    field.field_logical_name().as_str(),
                    field.column(),
                    columns
                )));
            }

            let positions = field_positions_by_column.entry(field.column).or_default();
            if !positions.insert(field.position) {
                return Err(AppError::Validation(format!(
                    "duplicate field position '{}' in column '{}' for section",
                    field.position, field.column
                )));
            }
        }

        for (column, positions) in field_positions_by_column {
            if !positions_are_contiguous(positions.into_iter().collect()) {
                return Err(AppError::Validation(format!(
                    "field positions in column '{}' must form contiguous sequence starting at zero",
                    column
                )));
            }
        }

        let mut seen_subgrids = HashSet::new();
        let mut subgrid_positions = HashSet::new();
        for subgrid in &subgrids {
            if !seen_subgrids.insert(subgrid.logical_name().as_str().to_owned()) {
                return Err(AppError::Validation(format!(
                    "duplicate sub-grid logical name '{}' in section",
                    subgrid.logical_name().as_str()
                )));
            }

            if !subgrid_positions.insert(subgrid.position) {
                return Err(AppError::Validation(format!(
                    "duplicate sub-grid position '{}' in section",
                    subgrid.position
                )));
            }
        }

        if !positions_are_contiguous(subgrid_positions.into_iter().collect()) {
            return Err(AppError::Validation(
                "sub-grid positions in section must form contiguous sequence starting at zero"
                    .to_owned(),
            ));
        }

        let mut sorted_fields = fields;
        sorted_fields.sort_by(|left, right| {
            left.column
                .cmp(&right.column)
                .then_with(|| left.position.cmp(&right.position))
                .then_with(|| {
                    left.field_logical_name
                        .as_str()
                        .cmp(right.field_logical_name.as_str())
                })
        });

        let mut sorted_subgrids = subgrids;
        sorted_subgrids.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then_with(|| left.logical_name.as_str().cmp(right.logical_name.as_str()))
        });

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            visible,
            columns,
            fields: sorted_fields,
            subgrids: sorted_subgrids,
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

    /// Returns sub-grid controls in this section.
    #[must_use]
    pub fn subgrids(&self) -> &[FormSubgrid] {
        &self.subgrids
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
        if position < 0 {
            return Err(AppError::Validation(
                "form tab position must be greater than or equal to zero".to_owned(),
            ));
        }

        if sections.is_empty() {
            return Err(AppError::Validation(
                "form tabs must include at least one section".to_owned(),
            ));
        }

        let mut section_positions = HashSet::new();
        for section in &sections {
            if !section_positions.insert(section.position) {
                return Err(AppError::Validation(format!(
                    "duplicate form section position '{}' in tab",
                    section.position
                )));
            }
        }

        if !positions_are_contiguous(section_positions.into_iter().collect()) {
            return Err(AppError::Validation(
                "form section positions in tab must form contiguous sequence starting at zero"
                    .to_owned(),
            ));
        }

        let mut sorted_sections = sections;
        sorted_sections.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then_with(|| left.logical_name.as_str().cmp(right.logical_name.as_str()))
        });

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            visible,
            sections: sorted_sections,
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

        if form_type == FormType::QuickCreate && (tabs.len() != 1 || tabs[0].sections().len() != 1)
        {
            return Err(AppError::Validation(
                "quick_create forms must contain exactly one tab and one section".to_owned(),
            ));
        }

        let mut tab_positions = HashSet::new();
        for tab in &tabs {
            if !tab_positions.insert(tab.position) {
                return Err(AppError::Validation(format!(
                    "duplicate form tab position '{}'",
                    tab.position
                )));
            }
        }

        if !positions_are_contiguous(tab_positions.into_iter().collect()) {
            return Err(AppError::Validation(
                "form tab positions must form contiguous sequence starting at zero".to_owned(),
            ));
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

        let mut sorted_tabs = tabs;
        sorted_tabs.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then_with(|| left.logical_name.as_str().cmp(right.logical_name.as_str()))
        });

        Ok(Self {
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            form_type,
            tabs: sorted_tabs,
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

fn positions_are_contiguous(mut positions: Vec<i32>) -> bool {
    positions.sort_unstable();
    positions.iter().enumerate().all(|(index, position)| {
        let Ok(expected) = i32::try_from(index) else {
            return false;
        };
        *position == expected
    })
}

#[cfg(test)]
mod tests {
    use super::{FormDefinition, FormFieldPlacement, FormSection, FormSubgrid, FormTab, FormType};

    #[test]
    fn form_field_placement_rejects_negative_position() {
        let placement = FormFieldPlacement::new("name", 0, -1, true, false, None, None);
        assert!(placement.is_err());
    }

    #[test]
    fn form_section_rejects_sparse_field_positions_per_column() {
        let first = FormFieldPlacement::new("name", 0, 0, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());
        let second = FormFieldPlacement::new("email", 0, 2, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());

        let section = FormSection::new(
            "main_section",
            "Main Section",
            0,
            true,
            1,
            vec![first, second],
            Vec::new(),
        );

        assert!(section.is_err());
    }

    #[test]
    fn form_section_rejects_duplicate_subgrid_positions() {
        let subgrid_a =
            FormSubgrid::new("accounts", "Accounts", "account", "owner_id", 0, Vec::new())
                .unwrap_or_else(|_| unreachable!());
        let subgrid_b =
            FormSubgrid::new("contacts", "Contacts", "contact", "owner_id", 0, Vec::new())
                .unwrap_or_else(|_| unreachable!());

        let section = FormSection::new(
            "main_section",
            "Main Section",
            0,
            true,
            1,
            Vec::new(),
            vec![subgrid_a, subgrid_b],
        );

        assert!(section.is_err());
    }

    #[test]
    fn form_section_normalizes_field_order_by_column_then_position() {
        let contact_phone = FormFieldPlacement::new("phone", 1, 0, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());
        let contact_email = FormFieldPlacement::new("email", 0, 1, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());
        let contact_name = FormFieldPlacement::new("name", 0, 0, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());

        let section = FormSection::new(
            "main_section",
            "Main Section",
            0,
            true,
            2,
            vec![contact_phone, contact_email, contact_name],
            Vec::new(),
        )
        .unwrap_or_else(|_| unreachable!());

        let field_order: Vec<&str> = section
            .fields()
            .iter()
            .map(|field| field.field_logical_name().as_str())
            .collect();
        assert_eq!(field_order, vec!["name", "email", "phone"]);
    }

    #[test]
    fn form_section_normalizes_subgrid_order_by_position() {
        let late = FormSubgrid::new("late", "Late", "account", "owner_id", 1, Vec::new())
            .unwrap_or_else(|_| unreachable!());
        let early = FormSubgrid::new("early", "Early", "contact", "owner_id", 0, Vec::new())
            .unwrap_or_else(|_| unreachable!());

        let section = FormSection::new(
            "main_section",
            "Main Section",
            0,
            true,
            1,
            Vec::new(),
            vec![late, early],
        )
        .unwrap_or_else(|_| unreachable!());

        let subgrid_order: Vec<&str> = section
            .subgrids()
            .iter()
            .map(|subgrid| subgrid.logical_name().as_str())
            .collect();
        assert_eq!(subgrid_order, vec!["early", "late"]);
    }

    #[test]
    fn form_tab_normalizes_section_order_by_position() {
        let late_section =
            FormSection::new("late_section", "Late", 1, true, 1, Vec::new(), Vec::new())
                .unwrap_or_else(|_| unreachable!());
        let early_section =
            FormSection::new("early_section", "Early", 0, true, 1, Vec::new(), Vec::new())
                .unwrap_or_else(|_| unreachable!());

        let tab = FormTab::new(
            "main_tab",
            "Main",
            0,
            true,
            vec![late_section, early_section],
        )
        .unwrap_or_else(|_| unreachable!());

        let section_order: Vec<&str> = tab
            .sections()
            .iter()
            .map(|section| section.logical_name().as_str())
            .collect();
        assert_eq!(section_order, vec!["early_section", "late_section"]);
    }

    #[test]
    fn form_definition_rejects_sparse_tab_positions() {
        let name = FormFieldPlacement::new("name", 0, 0, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());
        let email = FormFieldPlacement::new("email", 0, 0, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());

        let first_section = FormSection::new(
            "summary_section",
            "Summary",
            0,
            true,
            1,
            vec![name],
            Vec::new(),
        )
        .unwrap_or_else(|_| unreachable!());
        let second_section = FormSection::new(
            "details_section",
            "Details",
            0,
            true,
            1,
            vec![email],
            Vec::new(),
        )
        .unwrap_or_else(|_| unreachable!());

        let tab_a = FormTab::new("summary", "Summary", 0, true, vec![first_section])
            .unwrap_or_else(|_| unreachable!());
        let tab_b = FormTab::new("details", "Details", 2, true, vec![second_section])
            .unwrap_or_else(|_| unreachable!());

        let form = FormDefinition::new(
            "contact",
            "main_form",
            "Main Form",
            FormType::Main,
            vec![tab_a, tab_b],
            Vec::new(),
        );

        assert!(form.is_err());
    }

    #[test]
    fn form_definition_normalizes_tab_order_by_position() {
        let title = FormFieldPlacement::new("title", 0, 0, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());
        let name = FormFieldPlacement::new("name", 0, 0, true, false, None, None)
            .unwrap_or_else(|_| unreachable!());

        let late_section =
            FormSection::new("late_section", "Late", 0, true, 1, vec![title], Vec::new())
                .unwrap_or_else(|_| unreachable!());
        let early_section =
            FormSection::new("early_section", "Early", 0, true, 1, vec![name], Vec::new())
                .unwrap_or_else(|_| unreachable!());

        let late_tab = FormTab::new("late", "Late", 1, true, vec![late_section])
            .unwrap_or_else(|_| unreachable!());
        let early_tab = FormTab::new("early", "Early", 0, true, vec![early_section])
            .unwrap_or_else(|_| unreachable!());

        let form = FormDefinition::new(
            "contact",
            "main_form",
            "Main Form",
            FormType::Main,
            vec![late_tab, early_tab],
            Vec::new(),
        )
        .unwrap_or_else(|_| unreachable!());

        let tab_order: Vec<&str> = form
            .tabs()
            .iter()
            .map(|tab| tab.logical_name().as_str())
            .collect();
        assert_eq!(tab_order, vec!["early", "late"]);
    }
}
