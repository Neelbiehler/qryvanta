//! Domain entities and invariants.

#![forbid(unsafe_code)]

mod app;
mod business_rule;
mod dashboard;
mod form;
mod metadata;
mod security;
mod user;
mod view;
mod workflow;

pub use app::{
    AppDefinition, AppEntityAction, AppEntityBinding, AppEntityForm, AppEntityRolePermission,
    AppEntityView, AppEntityViewMode, AppSitemap, SitemapArea, SitemapGroup, SitemapSubArea,
    SitemapTarget,
};
pub use business_rule::{
    BusinessRuleAction, BusinessRuleActionType, BusinessRuleCondition, BusinessRuleDefinition,
    BusinessRuleDefinitionInput, BusinessRuleOperator, BusinessRuleScope,
};
pub use dashboard::{
    ChartAggregation, ChartDefinition, ChartType, DashboardDefinition, DashboardWidget,
};
pub use form::{FormDefinition, FormFieldPlacement, FormSection, FormSubgrid, FormTab, FormType};
pub use metadata::{
    EntityDefinition, EntityFieldDefinition, EntityFieldMutableUpdateInput, FieldType,
    OptionSetDefinition, OptionSetItem, PublishedEntitySchema, RuntimeRecord,
};
pub use security::{AuditAction, Permission, Surface};
pub use user::{
    AuthTokenType, EmailAddress, PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH_WITH_MFA,
    PASSWORD_MIN_LENGTH_WITHOUT_MFA, RegistrationMode, UserId, validate_password,
};
pub use view::{
    FilterOperator, LogicalMode, SortDirection, ViewColumn, ViewDefinition, ViewFilterCondition,
    ViewFilterGroup, ViewSort, ViewType,
};
pub use workflow::{
    WorkflowAction, WorkflowConditionOperator, WorkflowDefinition, WorkflowDefinitionInput,
    WorkflowStep, WorkflowTrigger,
};
