use qryvanta_core::{AppError, AppResult, NonEmptyString};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::str::FromStr;

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
    forms: Vec<AppEntityForm>,
    list_views: Vec<AppEntityView>,
    default_form_logical_name: NonEmptyString,
    default_list_view_logical_name: NonEmptyString,
    default_view_mode: AppEntityViewMode,
}

/// App-scoped model-driven form definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEntityForm {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    field_logical_names: Vec<String>,
}

/// App-scoped model-driven list view definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEntityView {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    field_logical_names: Vec<String>,
}

/// Default worker view mode for an app entity binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppEntityViewMode {
    /// Default grid/table view.
    Grid,
    /// Default JSON payload view.
    Json,
}

impl AppEntityViewMode {
    /// Returns stable storage value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::Json => "json",
        }
    }

    /// Parses storage value into a view mode.
    pub fn parse(value: &str) -> AppResult<Self> {
        match value {
            "grid" => Ok(Self::Grid),
            "json" => Ok(Self::Json),
            _ => Err(AppError::Validation(format!(
                "unknown app entity view mode '{value}'"
            ))),
        }
    }
}

impl FromStr for AppEntityViewMode {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl AppEntityBinding {
    /// Creates a validated app entity binding.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_logical_name: impl Into<String>,
        entity_logical_name: impl Into<String>,
        navigation_label: Option<String>,
        navigation_order: i32,
        forms: Vec<AppEntityForm>,
        list_views: Vec<AppEntityView>,
        default_form_logical_name: impl Into<String>,
        default_list_view_logical_name: impl Into<String>,
        default_view_mode: AppEntityViewMode,
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

        if forms.is_empty() {
            return Err(AppError::Validation(
                "forms must include at least one app-scoped form".to_owned(),
            ));
        }

        if list_views.is_empty() {
            return Err(AppError::Validation(
                "list_views must include at least one app-scoped list view".to_owned(),
            ));
        }

        validate_unique_surface_logical_names(
            forms.iter().map(AppEntityForm::logical_name),
            "forms",
        )?;
        validate_unique_surface_logical_names(
            list_views.iter().map(AppEntityView::logical_name),
            "list_views",
        )?;

        let default_form_logical_name = NonEmptyString::new(default_form_logical_name)?;
        let default_list_view_logical_name = NonEmptyString::new(default_list_view_logical_name)?;

        if !forms
            .iter()
            .any(|form| form.logical_name().as_str() == default_form_logical_name.as_str())
        {
            return Err(AppError::Validation(format!(
                "default form '{}' is not present in forms",
                default_form_logical_name.as_str()
            )));
        }

        if !list_views
            .iter()
            .any(|view| view.logical_name().as_str() == default_list_view_logical_name.as_str())
        {
            return Err(AppError::Validation(format!(
                "default list view '{}' is not present in list_views",
                default_list_view_logical_name.as_str()
            )));
        }

        Ok(Self {
            app_logical_name: NonEmptyString::new(app_logical_name)?,
            entity_logical_name: NonEmptyString::new(entity_logical_name)?,
            navigation_label,
            navigation_order,
            forms,
            list_views,
            default_form_logical_name,
            default_list_view_logical_name,
            default_view_mode,
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

    /// Returns app-scoped model-driven forms.
    #[must_use]
    pub fn forms(&self) -> &[AppEntityForm] {
        &self.forms
    }

    /// Returns app-scoped model-driven list views.
    #[must_use]
    pub fn list_views(&self) -> &[AppEntityView] {
        &self.list_views
    }

    /// Returns the default form logical name used by worker create/edit surfaces.
    #[must_use]
    pub fn default_form_logical_name(&self) -> &NonEmptyString {
        &self.default_form_logical_name
    }

    /// Returns the default list view logical name used by worker list surfaces.
    #[must_use]
    pub fn default_list_view_logical_name(&self) -> &NonEmptyString {
        &self.default_list_view_logical_name
    }

    /// Returns default worker view mode for this entity binding.
    #[must_use]
    pub fn default_view_mode(&self) -> AppEntityViewMode {
        self.default_view_mode
    }
}

impl AppEntityForm {
    /// Creates a validated app-scoped form definition.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        field_logical_names: Vec<String>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            field_logical_names: normalize_field_logical_names(
                field_logical_names,
                "field_logical_names",
            )?,
        })
    }

    /// Returns stable form logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns ordered field logical names.
    #[must_use]
    pub fn field_logical_names(&self) -> &[String] {
        &self.field_logical_names
    }
}

impl AppEntityView {
    /// Creates a validated app-scoped list view definition.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        field_logical_names: Vec<String>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            field_logical_names: normalize_field_logical_names(
                field_logical_names,
                "field_logical_names",
            )?,
        })
    }

    /// Returns stable list view logical name.
    #[must_use]
    pub fn logical_name(&self) -> &NonEmptyString {
        &self.logical_name
    }

    /// Returns display name.
    #[must_use]
    pub fn display_name(&self) -> &NonEmptyString {
        &self.display_name
    }

    /// Returns ordered column logical names.
    #[must_use]
    pub fn field_logical_names(&self) -> &[String] {
        &self.field_logical_names
    }
}

fn normalize_field_logical_names(values: Vec<String>, field_name: &str) -> AppResult<Vec<String>> {
    let mut normalized = Vec::with_capacity(values.len());
    let mut seen = HashSet::with_capacity(values.len());

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation(format!(
                "{field_name} contains an empty logical name"
            )));
        }

        if !seen.insert(trimmed.to_owned()) {
            return Err(AppError::Validation(format!(
                "{field_name} contains duplicate logical name '{trimmed}'"
            )));
        }

        normalized.push(trimmed.to_owned());
    }

    Ok(normalized)
}

fn validate_unique_surface_logical_names<'a>(
    values: impl Iterator<Item = &'a NonEmptyString>,
    field_name: &str,
) -> AppResult<()> {
    let mut seen = HashSet::new();
    for value in values {
        if !seen.insert(value.as_str().to_owned()) {
            return Err(AppError::Validation(format!(
                "{field_name} contains duplicate logical name '{}'",
                value.as_str()
            )));
        }
    }

    Ok(())
}

/// Hierarchical app navigation sitemap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSitemap {
    app_logical_name: NonEmptyString,
    areas: Vec<SitemapArea>,
}

impl AppSitemap {
    /// Creates a validated sitemap.
    pub fn new(app_logical_name: impl Into<String>, areas: Vec<SitemapArea>) -> AppResult<Self> {
        Ok(Self {
            app_logical_name: NonEmptyString::new(app_logical_name)?,
            areas,
        })
    }

    /// Returns app logical name.
    #[must_use]
    pub fn app_logical_name(&self) -> &NonEmptyString {
        &self.app_logical_name
    }

    /// Returns top-level areas.
    #[must_use]
    pub fn areas(&self) -> &[SitemapArea] {
        &self.areas
    }
}

/// Top-level area in an app sitemap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapArea {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    position: i32,
    icon: Option<String>,
    groups: Vec<SitemapGroup>,
}

impl SitemapArea {
    /// Creates a validated sitemap area.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        position: i32,
        icon: Option<String>,
        groups: Vec<SitemapGroup>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            icon,
            groups,
        })
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

    /// Returns display position.
    #[must_use]
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Returns optional icon identifier.
    #[must_use]
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Returns area groups.
    #[must_use]
    pub fn groups(&self) -> &[SitemapGroup] {
        &self.groups
    }
}

/// Mid-level group in an app sitemap area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapGroup {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    position: i32,
    sub_areas: Vec<SitemapSubArea>,
}

impl SitemapGroup {
    /// Creates a validated sitemap group.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        position: i32,
        sub_areas: Vec<SitemapSubArea>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            sub_areas,
        })
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

    /// Returns display position.
    #[must_use]
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Returns sub areas.
    #[must_use]
    pub fn sub_areas(&self) -> &[SitemapSubArea] {
        &self.sub_areas
    }
}

/// Leaf node in an app sitemap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapSubArea {
    logical_name: NonEmptyString,
    display_name: NonEmptyString,
    position: i32,
    target: SitemapTarget,
    icon: Option<String>,
}

impl SitemapSubArea {
    /// Creates a validated sub area.
    pub fn new(
        logical_name: impl Into<String>,
        display_name: impl Into<String>,
        position: i32,
        target: SitemapTarget,
        icon: Option<String>,
    ) -> AppResult<Self> {
        Ok(Self {
            logical_name: NonEmptyString::new(logical_name)?,
            display_name: NonEmptyString::new(display_name)?,
            position,
            target,
            icon,
        })
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

    /// Returns display position.
    #[must_use]
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Returns target payload.
    #[must_use]
    pub fn target(&self) -> &SitemapTarget {
        &self.target
    }

    /// Returns optional icon identifier.
    #[must_use]
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

/// Target destination for a sitemap sub area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SitemapTarget {
    /// Entity destination in runtime workspace.
    Entity {
        /// Entity logical name.
        entity_logical_name: String,
        /// Default form logical name.
        default_form: Option<String>,
        /// Default view logical name.
        default_view: Option<String>,
    },
    /// Dashboard destination (future surface).
    Dashboard {
        /// Dashboard logical name.
        dashboard_logical_name: String,
    },
    /// Custom page destination.
    CustomPage {
        /// Custom page URL.
        url: String,
    },
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
    use super::{AppDefinition, AppEntityBinding, AppEntityForm, AppEntityView, AppEntityViewMode};

    #[test]
    fn app_definition_requires_non_empty_values() {
        let app = AppDefinition::new("", "Sales", None);
        assert!(app.is_err());
    }

    #[test]
    fn app_entity_binding_rejects_negative_navigation_order() {
        let binding = AppEntityBinding::new(
            "sales",
            "account",
            None,
            -1,
            vec![
                AppEntityForm::new("main", "Main Form", Vec::new())
                    .unwrap_or_else(|_| unreachable!()),
            ],
            vec![
                AppEntityView::new("main", "Main View", Vec::new())
                    .unwrap_or_else(|_| unreachable!()),
            ],
            "main",
            "main",
            AppEntityViewMode::Grid,
        );
        assert!(binding.is_err());
    }

    #[test]
    fn app_entity_binding_rejects_duplicate_form_fields() {
        let form = AppEntityForm::new(
            "main",
            "Main Form",
            vec!["name".to_owned(), "name".to_owned()],
        );
        assert!(form.is_err());
    }

    #[test]
    fn app_entity_binding_rejects_unknown_default_form() {
        let binding = AppEntityBinding::new(
            "sales",
            "account",
            None,
            0,
            vec![
                AppEntityForm::new("main", "Main Form", Vec::new())
                    .unwrap_or_else(|_| unreachable!()),
            ],
            vec![
                AppEntityView::new("main", "Main View", Vec::new())
                    .unwrap_or_else(|_| unreachable!()),
            ],
            "missing",
            "main",
            AppEntityViewMode::Grid,
        );
        assert!(binding.is_err());
    }

    #[test]
    fn app_entity_binding_rejects_duplicate_form_logical_names() {
        let binding = AppEntityBinding::new(
            "sales",
            "account",
            None,
            0,
            vec![
                AppEntityForm::new("main", "Main A", Vec::new()).unwrap_or_else(|_| unreachable!()),
                AppEntityForm::new("main", "Main B", Vec::new()).unwrap_or_else(|_| unreachable!()),
            ],
            vec![
                AppEntityView::new("main", "Main View", Vec::new())
                    .unwrap_or_else(|_| unreachable!()),
            ],
            "main",
            "main",
            AppEntityViewMode::Grid,
        );
        assert!(binding.is_err());
    }

    #[test]
    fn app_entity_binding_rejects_unknown_default_list_view() {
        let binding = AppEntityBinding::new(
            "sales",
            "account",
            None,
            0,
            vec![
                AppEntityForm::new("main", "Main Form", Vec::new())
                    .unwrap_or_else(|_| unreachable!()),
            ],
            vec![
                AppEntityView::new("main", "Main View", Vec::new())
                    .unwrap_or_else(|_| unreachable!()),
            ],
            "main",
            "missing",
            AppEntityViewMode::Grid,
        );
        assert!(binding.is_err());
    }
}
