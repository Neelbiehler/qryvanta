use std::collections::HashSet;

use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};

/// Supported chart visualizations for metadata-driven dashboards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    /// Single KPI-style card.
    Kpi,
    /// Bar chart.
    Bar,
    /// Line chart.
    Line,
    /// Pie chart.
    Pie,
}

/// Supported aggregation operations for chart values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartAggregation {
    /// Record count.
    Count,
    /// Sum aggregation.
    Sum,
    /// Average aggregation.
    Avg,
    /// Minimum aggregation.
    Min,
    /// Maximum aggregation.
    Max,
}

/// Chart metadata model rendered inside a dashboard widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartDefinition {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    entity_logical_name: NonEmptyString,
    view_logical_name: Option<NonEmptyString>,
    chart_type: ChartType,
    aggregation: ChartAggregation,
    category_field_logical_name: Option<NonEmptyString>,
    value_field_logical_name: Option<NonEmptyString>,
}

impl ChartDefinition {
    /// Creates a validated chart definition.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        entity_logical_name: impl Into<String>,
        view_logical_name: Option<String>,
        chart_type: ChartType,
        aggregation: ChartAggregation,
        category_field_logical_name: Option<String>,
        value_field_logical_name: Option<String>,
    ) -> AppResult<Self> {
        let value_field_logical_name = value_field_logical_name
            .map(NonEmptyString::new)
            .transpose()?;

        if !matches!(aggregation, ChartAggregation::Count) && value_field_logical_name.is_none() {
            return Err(AppError::Validation(
                "non-count chart aggregations require value_field_logical_name".to_owned(),
            ));
        }

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            view_logical_name: view_logical_name.map(NonEmptyString::new).transpose()?,
            chart_type,
            aggregation,
            category_field_logical_name: category_field_logical_name
                .map(NonEmptyString::new)
                .transpose()?,
            value_field_logical_name,
        })
    }

    /// Returns chart logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns chart display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns source entity logical name.
    #[must_use]
    pub fn entity_logical_name(&self) -> &NonEmptyString {
        &self.entity_logical_name
    }

    /// Returns optional source view logical name.
    #[must_use]
    pub fn view_logical_name(&self) -> Option<&NonEmptyString> {
        self.view_logical_name.as_ref()
    }

    /// Returns chart visualization type.
    #[must_use]
    pub fn chart_type(&self) -> ChartType {
        self.chart_type
    }

    /// Returns aggregation mode.
    #[must_use]
    pub fn aggregation(&self) -> ChartAggregation {
        self.aggregation
    }

    /// Returns optional category field logical name.
    #[must_use]
    pub fn category_field_logical_name(&self) -> Option<&NonEmptyString> {
        self.category_field_logical_name.as_ref()
    }

    /// Returns optional value field logical name.
    #[must_use]
    pub fn value_field_logical_name(&self) -> Option<&NonEmptyString> {
        self.value_field_logical_name.as_ref()
    }
}

/// One dashboard grid widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardWidget {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    position: i32,
    width: i32,
    height: i32,
    chart: ChartDefinition,
}

impl DashboardWidget {
    /// Creates a validated dashboard widget.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        position: i32,
        width: i32,
        height: i32,
        chart: ChartDefinition,
    ) -> AppResult<Self> {
        if position < 0 {
            return Err(AppError::Validation(
                "dashboard widget position must be non-negative".to_owned(),
            ));
        }

        if width <= 0 || height <= 0 {
            return Err(AppError::Validation(
                "dashboard widget width and height must be positive".to_owned(),
            ));
        }

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            width,
            height,
            chart,
        })
    }

    /// Returns widget logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns widget display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns widget position.
    #[must_use]
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Returns widget width.
    #[must_use]
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Returns widget height.
    #[must_use]
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Returns chart definition.
    #[must_use]
    pub fn chart(&self) -> &ChartDefinition {
        &self.chart
    }
}

/// Dashboard metadata definition containing ordered chart widgets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardDefinition {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    widgets: Vec<DashboardWidget>,
}

impl DashboardDefinition {
    /// Creates a validated dashboard definition.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        widgets: Vec<DashboardWidget>,
    ) -> AppResult<Self> {
        let mut seen_widget_names = HashSet::new();
        let mut seen_positions = HashSet::new();
        for widget in &widgets {
            if !seen_widget_names.insert(widget.logical_name().as_str().to_owned()) {
                return Err(AppError::Validation(format!(
                    "duplicate dashboard widget logical name '{}'",
                    widget.logical_name().as_str()
                )));
            }

            if !seen_positions.insert(widget.position()) {
                return Err(AppError::Validation(format!(
                    "duplicate dashboard widget position '{}'",
                    widget.position()
                )));
            }
        }

        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            widgets,
        })
    }

    /// Returns dashboard logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns dashboard display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns dashboard widgets.
    #[must_use]
    pub fn widgets(&self) -> &[DashboardWidget] {
        &self.widgets
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChartAggregation, ChartDefinition, ChartType, DashboardDefinition, DashboardWidget,
    };

    #[test]
    fn chart_requires_value_field_for_non_count_aggregation() {
        let chart = ChartDefinition::new(
            "revenue_sum",
            "Revenue",
            "invoice",
            Some("main_view".to_owned()),
            ChartType::Bar,
            ChartAggregation::Sum,
            Some("month".to_owned()),
            None,
        );
        assert!(chart.is_err());
    }

    #[test]
    fn dashboard_rejects_duplicate_widget_positions() {
        let chart = ChartDefinition::new(
            "count",
            "Count",
            "account",
            None,
            ChartType::Kpi,
            ChartAggregation::Count,
            None,
            None,
        )
        .unwrap_or_else(|_| unreachable!());

        let dashboard = DashboardDefinition::new(
            "overview",
            "Overview",
            vec![
                DashboardWidget::new("first", "First", 0, 4, 3, chart.clone())
                    .unwrap_or_else(|_| unreachable!()),
                DashboardWidget::new("second", "Second", 0, 4, 3, chart)
                    .unwrap_or_else(|_| unreachable!()),
            ],
        );

        assert!(dashboard.is_err());
    }
}
