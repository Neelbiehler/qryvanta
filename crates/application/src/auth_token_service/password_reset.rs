use qryvanta_domain::{AuthTokenType, EmailAddress, UserId};

use super::token_crypto::generate_token;
use super::*;

impl AuthTokenService {
    /// Issues a password reset token and sends the reset email.
    ///
    /// Always returns `Ok(())` regardless of whether the email exists,
    /// per OWASP Forgot Password: "If that email is in our system, we will
    /// send you an email to reset your password."
    pub async fn request_password_reset(
        &self,
        email: &str,
        user_id: Option<UserId>,
    ) -> AppResult<()> {
        let Ok(canonical_email) = EmailAddress::new(email) else {
            // Keep generic success response semantics for invalid inputs.
            return Ok(());
        };

        // Rate limit: max 3 reset requests per email per hour.
        let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);
        let recent_count = self
            .token_repository
            .count_recent_tokens(
                canonical_email.as_str(),
                AuthTokenType::PasswordReset,
                one_hour_ago,
            )
            .await?;

        if recent_count >= 3 {
            // Silently succeed to prevent enumeration.
            return Ok(());
        }

        let Some(uid) = user_id else {
            // User not found -- silently succeed.
            return Ok(());
        };

        // Invalidate any existing reset tokens for this user.
        self.token_repository
            .invalidate_tokens_for_user(uid, AuthTokenType::PasswordReset)
            .await?;

        let (raw_token, token_hash) = generate_token()?;

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        self.token_repository
            .create_token(
                Some(uid),
                canonical_email.as_str(),
                &token_hash,
                AuthTokenType::PasswordReset,
                expires_at,
                None,
            )
            .await?;

        let reset_url = format!("{}/reset-password?token={}", self.frontend_url, raw_token);

        let subject = "Reset your Qryvanta password";
        let text_body = format!(
            "You requested a password reset.\n\n\
             Click the link below to set a new password:\n{reset_url}\n\n\
             This link expires in 1 hour.\n\n\
             If you did not request this, you can safely ignore this email."
        );

        self.email_service
            .send_email(canonical_email.as_str(), subject, &text_body, None)
            .await?;

        Ok(())
    }
}
