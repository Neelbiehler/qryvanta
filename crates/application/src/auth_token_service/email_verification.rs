use qryvanta_domain::{AuthTokenType, EmailAddress, UserId};

use super::token_crypto::generate_token;
use super::*;

impl AuthTokenService {
    /// Issues an email verification token and sends the verification email.
    pub async fn send_email_verification(&self, user_id: UserId, email: &str) -> AppResult<()> {
        let canonical_email = EmailAddress::new(email)?;

        // Invalidate previous verification tokens.
        self.token_repository
            .invalidate_tokens_for_user(user_id, AuthTokenType::EmailVerification)
            .await?;

        let (raw_token, token_hash) = generate_token()?;

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        self.token_repository
            .create_token(
                Some(user_id),
                canonical_email.as_str(),
                &token_hash,
                AuthTokenType::EmailVerification,
                expires_at,
                None,
            )
            .await?;

        let verify_url = format!("{}/verify-email?token={}", self.frontend_url, raw_token);

        let subject = "Verify your Qryvanta email address";
        let text_body = format!(
            "Welcome to Qryvanta!\n\n\
             Please verify your email address by clicking the link below:\n{verify_url}\n\n\
             This link expires in 24 hours."
        );

        self.email_service
            .send_email(canonical_email.as_str(), subject, &text_body, None)
            .await?;

        Ok(())
    }
}
