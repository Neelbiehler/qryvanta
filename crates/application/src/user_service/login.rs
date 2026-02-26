use crate::AuthEvent;

use super::*;

impl UserService {
    /// Authenticates a user with email and password.
    ///
    /// Returns `AuthOutcome::Failed` with a generic message for any failure
    /// (unknown email, wrong password, locked account) to prevent enumeration.
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthOutcome> {
        let user = self.user_repository.find_by_email(email).await?;

        let Some(user) = user else {
            // OWASP: always hash to prevent timing attacks even when user not found.
            let _ = self.password_hasher.hash_password(password);
            return Ok(AuthOutcome::Failed);
        };

        // Check account lockout.
        if let Some(locked_until) = user.locked_until
            && chrono::Utc::now() < locked_until
        {
            // Still locked -- don't reveal this; just say failed.
            let _ = self.password_hasher.hash_password(password);

            self.auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.id.to_string()),
                    event_type: "login_attempt".to_owned(),
                    outcome: "account_locked".to_owned(),
                    ip_address,
                    user_agent,
                })
                .await?;

            return Ok(AuthOutcome::Failed);
        }

        let Some(ref stored_hash) = user.password_hash else {
            // Passkey-only user trying password login -- fail generically.
            let _ = self.password_hasher.hash_password(password);
            return Ok(AuthOutcome::Failed);
        };

        let password_valid = self
            .password_hasher
            .verify_password(password, stored_hash)?;

        if !password_valid {
            self.user_repository.record_failed_login(user.id).await?;

            self.auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.id.to_string()),
                    event_type: "login_attempt".to_owned(),
                    outcome: "invalid_password".to_owned(),
                    ip_address,
                    user_agent,
                })
                .await?;

            return Ok(AuthOutcome::Failed);
        }

        // Password correct -- reset failed login counter.
        self.user_repository.reset_failed_logins(user.id).await?;

        // Check if MFA is required.
        if user.totp_enabled {
            self.auth_event_service
                .record_event(AuthEvent {
                    subject: Some(user.id.to_string()),
                    event_type: "login_attempt".to_owned(),
                    outcome: "mfa_required".to_owned(),
                    ip_address,
                    user_agent,
                })
                .await?;

            return Ok(AuthOutcome::MfaRequired { user_id: user.id });
        }

        self.auth_event_service
            .record_event(AuthEvent {
                subject: Some(user.id.to_string()),
                event_type: "login_attempt".to_owned(),
                outcome: "success".to_owned(),
                ip_address,
                user_agent,
            })
            .await?;

        Ok(AuthOutcome::Authenticated(user))
    }
}
