use std::collections::HashSet;
use std::str::FromStr;

use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Supported standalone view type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    /// Tabular grid view.
    Grid,
    /// Card-based view.
    Card,
}

impl ViewType {
    /// Returns stable storage value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::Card => "card",
        }
    }
}

impl FromStr for ViewType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "grid" => Ok(Self::Grid),
            "card" => Ok(Self::Card),
            _ => Err(AppError::Validation(format!("unknown view type '{value}'"))),
        }
    }
}

/// Sort direction for view default sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Logical mode for filter groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicalMode {
    /// All conditions must match.
    And,
    /// Any condition may match.
    Or,
}

/// Filter operator for view filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    /// Equality comparison.
    Eq,
    /// Inequality comparison.
    Neq,
    /// Greater-than comparison.
    Gt,
    /// Greater-than-or-equal comparison.
    Gte,
    /// Less-than comparison.
    Lt,
    /// Less-than-or-equal comparison.
    Lte,
    /// Substring match for text values.
    Contains,
    /// Membership in provided set.
    In,
}

/// View column definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewColumn {
    field_logical_name: NonEmptyString,
    position: i32,
    width: Option<i32>,
    label_override: Option<String>,
}

impl ViewColumn {
    /// Creates a validated view column.
    pub fn new(
        field_logical_name: impl Into<String>,
        position: i32,
        width: Option<i32>,
        label_override: Option<String>,
    ) -> AppResult<Self> {
        if let Some(width) = width
            && width <= 0
        {
            return Err(AppError::Validation(
                "view column width must be greater than zero".to_owned(),
            ));
        }

        Ok(Self {
            field_logical_name: NonEmptyString::new(field_logical_name)?,
            position,
            width,
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
}

/// Default sort definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewSort {
    field_logical_name: NonEmptyString,
    direction: SortDirection,
}

impl ViewSort {
    /// Creates a validated sort definition.
    pub fn new(field_logical_name: impl Into<String>, direction: SortDirection) -> AppResult<Self> {
        Ok(Self {
            field_logical_name: NonEmptyString::new(field_logical_name)?,
            direction,
        })
    }

    /// Returns field logical name.
    #[must_use]
    pub fn field_logical_name(&self) -> &NonEmptyString {
        &self.field_logical_name
    }
}

/// One filter condition in a view filter group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewFilterCondition {
    field_logical_name: NonEmptyString,
    operator: FilterOperator,
    value: Value,
}

impl ViewFilterCondition {
    /// Creates a validated filter condition.
    pub fn new(
        field_logical_name: impl Into<String>,
        operator: FilterOperator,
        value: Value,
    ) -> AppResult<Self> {
        Ok(Self {
            field_logical_name: NonEmptyString::new(field_logical_name)?,
            operator,
            value,
        })
    }

    /// Returns condition field logical name.
    #[must_use]
    pub fn field_logical_name(&self) -> &NonEmptyString {
        &self.field_logical_name
    }
}

/// Grouped view filter criteria.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewFilterGroup {
    logical_mode: LogicalMode,
    conditions: Vec<ViewFilterCondition>,
}

impl ViewFilterGroup {
    /// Creates a validated filter group.
    pub fn new(logical_mode: LogicalMode, conditions: Vec<ViewFilterCondition>) -> AppResult<Self> {
        if conditions.is_empty() {
            return Err(AppError::Validation(
                "view filter groups must include at least one condition".to_owned(),
            ));
        }

        Ok(Self {
            logical_mode,
            conditions,
        })
    }

    /// Returns group conditions.
    #[must_use]
    pub fn conditions(&self) -> &[ViewFilterCondition] {
        &self.conditions
    }
}

/// Standalone view definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewDefinition {
    entity_logical_name: NonEmptyString,
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    view_type: ViewType,
    columns: Vec<ViewColumn>,
    default_sort: Option<ViewSort>,
    filter_criteria: Option<ViewFilterGroup>,
    is_default: bool,
}

impl ViewDefinition {
    /// Creates a validated view definition.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_logical_name: impl Into<String>,
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        view_type: ViewType,
        columns: Vec<ViewColumn>,
        default_sort: Option<ViewSort>,
        filter_criteria: Option<ViewFilterGroup>,
        is_default: bool,
    ) -> AppResult<Self> {
        if columns.is_empty() {
            return Err(AppError::Validation(
                "views must include at least one column".to_owned(),
            ));
        }

        let mut seen_columns = HashSet::new();
        for column in &columns {
            if !seen_columns.insert(column.field_logical_name().as_str().to_owned()) {
                return Err(AppError::Validation(format!(
                    "duplicate view column '{}'",
                    column.field_logical_name().as_str()
                )));
            }
        }

        Ok(Self {
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            view_type,
            columns,
            default_sort,
            filter_criteria,
            is_default,
        })
    }

    /// Returns parent entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns view type.
    #[must_use]
    pub fn view_type(&self) -> ViewType {
        self.view_type
    }

    /// Returns view columns.
    #[must_use]
    pub fn columns(&self) -> &[ViewColumn] {
        &self.columns
    }

    /// Returns optional default sort.
    #[must_use]
    pub fn default_sort(&self) -> Option<&ViewSort> {
        self.default_sort.as_ref()
    }

    /// Returns optional filter criteria.
    #[must_use]
    pub fn filter_criteria(&self) -> Option<&ViewFilterGroup> {
        self.filter_criteria.as_ref()
    }

    /// Returns whether this view is default.
    #[must_use]
    pub fn is_default(&self) -> bool {
        self.is_default
    }
}
