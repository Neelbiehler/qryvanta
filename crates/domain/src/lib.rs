//! Domain entities and invariants.

#![forbid(unsafe_code)]

mod app;
mod business_rule;
mod dashboard;
mod extension;
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
pub use extension::{
    ExtensionCapability, ExtensionDefinition, ExtensionIsolationPolicy, ExtensionLifecycleState,
    ExtensionManifest, ExtensionManifestInput, ExtensionRuntimeKind,
};
pub use form::{FormDefinition, FormFieldPlacement, FormSection, FormSubgrid, FormTab, FormType};
pub use metadata::{
    EntityDefinition, EntityFieldDefinition, EntityFieldMutableUpdateInput, FieldType,
    OptionSetDefinition, OptionSetItem, PublishedEntitySchema, RuntimeRecord,
};
pub use security::{AuditAction, AuthEventOutcome, AuthEventType, Permission, Surface};
pub use user::{
    AuthTokenType, EmailAddress, PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH_WITH_MFA,
    PASSWORD_MIN_LENGTH_WITHOUT_MFA, RegistrationMode, UserId, validate_password,
};
pub use view::{
    FilterOperator, LogicalMode, SortDirection, ViewColumn, ViewDefinition, ViewFilterCondition,
    ViewFilterGroup, ViewSort, ViewType,
};
pub use workflow::{
    WorkflowConditionOperator, WorkflowDefinition, WorkflowDefinitionInput, WorkflowLifecycleState,
    WorkflowStep, WorkflowTrigger, is_sensitive_workflow_header_name,
    redact_sensitive_workflow_headers, redact_workflow_header_secret_refs,
};
