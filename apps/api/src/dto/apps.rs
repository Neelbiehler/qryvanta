use qryvanta_domain::{AppDefinition, AppEntityBinding, AppEntityRolePermission};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Incoming payload for app creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/create-app-request.ts"
)]
pub struct CreateAppRequest {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
}

/// API representation of an app definition.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/app-response.ts"
)]
pub struct AppResponse {
    pub logical_name: String,
    pub display_name: String,
    pub description: Option<String>,
}

/// Incoming payload for binding an entity into app navigation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/bind-app-entity-request.ts"
)]
pub struct BindAppEntityRequest {
    pub entity_logical_name: String,
    pub navigation_label: Option<String>,
    pub navigation_order: i32,
}

/// API representation of an app entity navigation binding.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/app-entity-binding-response.ts"
)]
pub struct AppEntityBindingResponse {
    pub app_logical_name: String,
    pub entity_logical_name: String,
    pub navigation_label: Option<String>,
    pub navigation_order: i32,
}

/// Incoming payload for app role entity permission updates.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/save-app-role-entity-permission-request.ts"
)]
pub struct SaveAppRoleEntityPermissionRequest {
    pub role_name: String,
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// API representation of app-scoped role entity permissions.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/app-role-entity-permission-response.ts"
)]
pub struct AppRoleEntityPermissionResponse {
    pub app_logical_name: String,
    pub role_name: String,
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// API representation of effective app entity capabilities for the current subject.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/app-entity-capabilities-response.ts"
)]
pub struct AppEntityCapabilitiesResponse {
    pub entity_logical_name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

impl From<AppDefinition> for AppResponse {
    fn from(value: AppDefinition) -> Self {
        Self {
            logical_name: value.logical_name().as_str().to_owned(),
            display_name: value.display_name().as_str().to_owned(),
            description: value.description().map(ToOwned::to_owned),
        }
    }
}

impl From<AppEntityBinding> for AppEntityBindingResponse {
    fn from(value: AppEntityBinding) -> Self {
        Self {
            app_logical_name: value.app_logical_name().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            navigation_label: value.navigation_label().map(ToOwned::to_owned),
            navigation_order: value.navigation_order(),
        }
    }
}

impl From<AppEntityRolePermission> for AppRoleEntityPermissionResponse {
    fn from(value: AppEntityRolePermission) -> Self {
        Self {
            app_logical_name: value.app_logical_name().as_str().to_owned(),
            role_name: value.role_name().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            can_read: value.can_read(),
            can_create: value.can_create(),
            can_update: value.can_update(),
            can_delete: value.can_delete(),
        }
    }
}

impl From<qryvanta_application::SubjectEntityPermission> for AppEntityCapabilitiesResponse {
    fn from(value: qryvanta_application::SubjectEntityPermission) -> Self {
        Self {
            entity_logical_name: value.entity_logical_name,
            can_read: value.can_read,
            can_create: value.can_create,
            can_update: value.can_update,
            can_delete: value.can_delete,
        }
    }
}
