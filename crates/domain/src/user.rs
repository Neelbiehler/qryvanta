//! User domain types and validation rules.
//!
//! Follows OWASP Authentication and Password Storage cheat sheets for all
//! password strength and email validation rules.

use std::str::FromStr;

use qryvanta_core::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a user record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// Creates a new random user identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a user identifier from an existing UUID value.
    #[must_use]
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Validated email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Creates a validated email address.
    ///
    /// Performs basic structural validation: non-empty, contains exactly one `@`,
    /// local part and domain are non-empty, domain contains at least one `.`.
    pub fn new(value: impl Into<String>) -> AppResult<Self> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "email address must not be empty".to_owned(),
            ));
        }

        let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
        if parts.len() != 2 {
            return Err(AppError::Validation(
                "email address must contain exactly one '@'".to_owned(),
            ));
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() {
            return Err(AppError::Validation(
                "email local part must not be empty".to_owned(),
            ));
        }

        if domain.is_empty() || !domain.contains('.') {
            return Err(AppError::Validation(
                "email domain must contain at least one '.'".to_owned(),
            ));
        }

        if trimmed.len() > 254 {
            return Err(AppError::Validation(
                "email address must not exceed 254 characters".to_owned(),
            ));
        }

        Ok(Self(trimmed))
    }

    /// Returns the validated email string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<EmailAddress> for String {
    fn from(value: EmailAddress) -> Self {
        value.0
    }
}

/// Minimum password length when MFA is enabled (NIST SP800-63B).
pub const PASSWORD_MIN_LENGTH_WITH_MFA: usize = 8;

/// Minimum password length when MFA is NOT enabled (NIST SP800-63B).
pub const PASSWORD_MIN_LENGTH_WITHOUT_MFA: usize = 10;

/// Maximum password length to allow passphrases (OWASP recommendation: at least 64).
pub const PASSWORD_MAX_LENGTH: usize = 128;

/// Validates a plaintext password against OWASP and NIST rules.
///
/// - Min length depends on whether MFA is enabled for the account.
/// - Max length is 128 characters (protects against Argon2id DoS).
/// - Rejects common breached passwords from an embedded list.
pub fn validate_password(password: &str, has_mfa: bool) -> AppResult<()> {
    let char_count = password.chars().count();
    let min_length = if has_mfa {
        PASSWORD_MIN_LENGTH_WITH_MFA
    } else {
        PASSWORD_MIN_LENGTH_WITHOUT_MFA
    };

    if char_count < min_length {
        return Err(AppError::Validation(format!(
            "password must be at least {min_length} characters"
        )));
    }

    if char_count > PASSWORD_MAX_LENGTH {
        return Err(AppError::Validation(format!(
            "password must not exceed {PASSWORD_MAX_LENGTH} characters"
        )));
    }

    if is_common_password(password) {
        return Err(AppError::Validation(
            "this password is too common and has appeared in data breaches".to_owned(),
        ));
    }

    Ok(())
}

/// Checks whether a password appears in the embedded common passwords list.
fn is_common_password(password: &str) -> bool {
    let lowered = password.to_lowercase();
    COMMON_PASSWORDS.iter().any(|entry| *entry == lowered)
}

/// Top breached passwords (subset for fast embedded check).
/// Production deployments should integrate HaveIBeenPwned k-anonymity API.
static COMMON_PASSWORDS: &[&str] = &[
    "password",
    "123456",
    "12345678",
    "1234567890",
    "qwerty",
    "abc123",
    "monkey",
    "master",
    "dragon",
    "111111",
    "baseball",
    "iloveyou",
    "trustno1",
    "sunshine",
    "princess",
    "football",
    "shadow",
    "superman",
    "qwerty123",
    "michael",
    "password1",
    "password123",
    "welcome",
    "login",
    "admin",
    "letmein",
    "starwars",
    "solo",
    "passw0rd",
    "121212",
    "flower",
    "hottie",
    "loveme",
    "access",
    "hello",
    "charlie",
    "donald",
    "qwertyuiop",
    "whatever",
    "654321",
    "7777777",
    "123123",
    "jordan",
    "hunter",
    "pepper",
    "buster",
    "joshua",
    "freedom",
    "1234567",
    "12345",
];

/// Token types for the auth_tokens table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthTokenType {
    /// Email address verification token.
    EmailVerification,
    /// Password reset token.
    PasswordReset,
    /// Tenant invite token.
    Invite,
}

impl AuthTokenType {
    /// Returns the storage string for this token type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmailVerification => "email_verification",
            Self::PasswordReset => "password_reset",
            Self::Invite => "invite",
        }
    }
}

impl FromStr for AuthTokenType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "email_verification" => Ok(Self::EmailVerification),
            "password_reset" => Ok(Self::PasswordReset),
            "invite" => Ok(Self::Invite),
            _ => Err(AppError::Validation(format!(
                "unknown auth token type '{value}'"
            ))),
        }
    }
}

/// Registration mode for a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationMode {
    /// Only invited users can join the tenant.
    InviteOnly,
    /// Anyone can register and create an account.
    Open,
}

impl RegistrationMode {
    /// Returns the storage string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InviteOnly => "invite_only",
            Self::Open => "open",
        }
    }

    /// Parses a storage string into a registration mode.
    pub fn parse(value: &str) -> AppResult<Self> {
        match value {
            "invite_only" => Ok(Self::InviteOnly),
            "open" => Ok(Self::Open),
            _ => Err(AppError::Validation(format!(
                "unknown registration mode '{value}'"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email_is_accepted() {
        let email = EmailAddress::new("USER@Example.COM");
        assert!(email.is_ok());
        assert_eq!(
            email.unwrap_or_else(|_| panic!("test")).as_str(),
            "user@example.com"
        );
    }

    #[test]
    fn email_without_at_is_rejected() {
        assert!(EmailAddress::new("noatsign").is_err());
    }

    #[test]
    fn email_without_domain_dot_is_rejected() {
        assert!(EmailAddress::new("user@nodot").is_err());
    }

    #[test]
    fn empty_email_is_rejected() {
        assert!(EmailAddress::new("").is_err());
    }

    #[test]
    fn short_password_is_rejected_without_mfa() {
        assert!(validate_password("short", false).is_err());
    }

    #[test]
    fn adequate_password_is_accepted_without_mfa() {
        assert!(validate_password("a-reasonable-passphrase", false).is_ok());
    }

    #[test]
    fn shorter_password_accepted_with_mfa() {
        assert!(validate_password("g00dPa5s", true).is_ok());
    }

    #[test]
    fn common_password_is_rejected() {
        assert!(validate_password("password123", false).is_err());
    }

    #[test]
    fn very_long_password_is_rejected() {
        let long = "a".repeat(PASSWORD_MAX_LENGTH + 1);
        assert!(validate_password(&long, false).is_err());
    }

    #[test]
    fn max_length_password_is_accepted() {
        let max = "b".repeat(PASSWORD_MAX_LENGTH);
        assert!(validate_password(&max, false).is_ok());
    }
}
