use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Incoming payload for email/password registration.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/auth-register-request.ts"
)]
pub struct AuthRegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

/// Incoming payload for email/password login.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/auth-login-request.ts"
)]
pub struct AuthLoginRequest {
    pub email: String,
    pub password: String,
}

/// Auth status response for login and challenge flows.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/auth-login-response.ts"
)]
pub struct AuthLoginResponse {
    pub status: String,
    pub requires_totp: bool,
}

/// Incoming payload for TOTP or recovery code verification.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/auth-mfa-verify-request.ts"
)]
pub struct AuthMfaVerifyRequest {
    pub code: String,
    pub method: Option<String>,
}

/// Incoming payload for invite creation.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/invite-request.ts"
)]
pub struct InviteRequest {
    pub email: String,
    pub tenant_name: Option<String>,
}

/// Incoming payload for invite acceptance.
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../../packages/api-types/src/generated/accept-invite-request.ts"
)]
pub struct AcceptInviteRequest {
    pub token: String,
    pub password: Option<String>,
    pub display_name: Option<String>,
}
