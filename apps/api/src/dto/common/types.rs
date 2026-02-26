use serde::Serialize;
use ts_rs::TS;

/// Health response payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/health-response.ts"
)]
pub struct HealthResponse {
    pub status: &'static str,
    pub ready: bool,
    pub postgres: HealthDependencyStatus,
    pub redis: HealthDependencyStatus,
}

/// One runtime dependency health status.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/health-dependency-status.ts"
)]
pub struct HealthDependencyStatus {
    pub status: &'static str,
    pub detail: Option<String>,
}

/// Generic message response for auth flows.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/generic-message-response.ts"
)]
pub struct GenericMessageResponse {
    pub message: String,
}

/// API representation of the authenticated user.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/user-identity-response.ts"
)]
pub struct UserIdentityResponse {
    pub subject: String,
    pub display_name: String,
    pub email: Option<String>,
    pub tenant_id: String,
    /// Surfaces the authenticated user may access (e.g. `['admin', 'maker', 'worker']`).
    pub accessible_surfaces: Vec<String>,
}
