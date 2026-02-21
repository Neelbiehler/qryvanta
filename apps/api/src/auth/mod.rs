use qryvanta_application::RateLimitRule;

mod bootstrap;
mod invite;
mod mfa;
mod passkey;
mod password;
mod session;
mod session_helpers;

pub use bootstrap::bootstrap_handler;
pub use invite::{accept_invite_handler, send_invite_handler};
pub use mfa::{
    mfa_confirm_handler, mfa_disable_handler, mfa_enroll_handler,
    mfa_regenerate_recovery_codes_handler,
};
pub use passkey::{
    webauthn_login_finish_handler, webauthn_login_start_handler,
    webauthn_registration_finish_handler, webauthn_registration_start_handler,
};
pub use password::{
    change_password_handler, forgot_password_handler, login_handler, mfa_verify_handler,
    register_handler, resend_verification_handler, reset_password_handler, verify_email_handler,
};
pub use session::{logout_handler, me_handler};

pub const SESSION_USER_KEY: &str = "user_identity";
/// Absolute session creation timestamp for OWASP absolute timeout enforcement.
pub const SESSION_CREATED_AT_KEY: &str = "session_created_at";
pub(super) const SESSION_MFA_PENDING_KEY: &str = "mfa_pending_user_id";
pub(super) const SESSION_WEBAUTHN_REG_STATE_KEY: &str = "webauthn_reg_state";
pub(super) const SESSION_WEBAUTHN_AUTH_STATE_KEY: &str = "webauthn_auth_state";

pub(super) const RESEND_VERIFICATION_RATE_RULE: (i32, i64) = (5, 60 * 60);
pub(super) const INVITE_SENDER_RATE_RULE: (i32, i64) = (20, 60 * 60);
pub(super) const INVITE_RECIPIENT_RATE_RULE: (i32, i64) = (3, 60 * 60);
pub(super) const VERIFY_EMAIL_RATE_RULE: (i32, i64) = (30, 60 * 60);

pub(super) fn verify_email_rate_rule() -> RateLimitRule {
    RateLimitRule::new(
        "verify_email",
        VERIFY_EMAIL_RATE_RULE.0,
        VERIFY_EMAIL_RATE_RULE.1,
    )
}

pub(super) fn resend_verification_rate_rule() -> RateLimitRule {
    RateLimitRule::new(
        "resend_verification",
        RESEND_VERIFICATION_RATE_RULE.0,
        RESEND_VERIFICATION_RATE_RULE.1,
    )
}

pub(super) fn invite_sender_rate_rule() -> RateLimitRule {
    RateLimitRule::new(
        "invite_sender",
        INVITE_SENDER_RATE_RULE.0,
        INVITE_SENDER_RATE_RULE.1,
    )
}

pub(super) fn invite_recipient_rate_rule() -> RateLimitRule {
    RateLimitRule::new(
        "invite_recipient",
        INVITE_RECIPIENT_RATE_RULE.0,
        INVITE_RECIPIENT_RATE_RULE.1,
    )
}
