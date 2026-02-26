use qryvanta_domain::{AuthTokenType, EmailAddress};

use super::token_crypto::generate_token;
use super::*;

impl AuthTokenService {
    /// Issues an invite token and sends the invitation email.
    pub async fn send_invite(
        &self,
        email: &str,
        inviter_name: &str,
        tenant_name: &str,
        metadata: &serde_json::Value,
    ) -> AppResult<()> {
        let canonical_email = EmailAddress::new(email)?;

        let (raw_token, token_hash) = generate_token()?;

        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
        self.token_repository
            .create_token(
                None,
                canonical_email.as_str(),
                &token_hash,
                AuthTokenType::Invite,
                expires_at,
                Some(metadata),
            )
            .await?;

        let invite_url = format!("{}/accept-invite?token={}", self.frontend_url, raw_token);

        let subject = format!("{inviter_name} invited you to {tenant_name} on Qryvanta");
        let text_body = format!(
            "{inviter_name} has invited you to join {tenant_name} on Qryvanta.\n\n\
             Click the link below to accept the invitation:\n{invite_url}\n\n\
             This link expires in 7 days."
        );

        self.email_service
            .send_email(canonical_email.as_str(), &subject, &text_body, None)
            .await?;

        Ok(())
    }
}
