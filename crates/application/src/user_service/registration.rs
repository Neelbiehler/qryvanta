use qryvanta_core::AppError;
use qryvanta_domain::{EmailAddress, RegistrationMode, validate_password};

use crate::AuthEvent;

use super::*;

impl UserService {
    /// Registers a new user with email and password.
    ///
    /// Only allowed when the tenant registration mode is `Open` or when
    /// called from an invite acceptance flow (caller is responsible for
    /// that check).
    pub async fn register(&self, params: RegisterParams) -> AppResult<UserId> {
        if params.registration_mode == RegistrationMode::InviteOnly {
            return Err(AppError::Forbidden(
                "registration is currently invite-only".to_owned(),
            ));
        }

        let email_address = EmailAddress::new(&params.email)?;
        validate_password(&params.password, false)?;

        // Check for existing user -- always hash to prevent timing attacks.
        let existing = self
            .user_repository
            .find_by_email(email_address.as_str())
            .await?;

        if existing.is_some() {
            // OWASP: do not reveal that the account exists.
            // Still hash the password to prevent timing side-channels.
            let _ = self.password_hasher.hash_password(&params.password);
            return Err(AppError::Conflict(
                "a link to activate your account has been emailed to the address provided"
                    .to_owned(),
            ));
        }

        let password_hash = self.password_hasher.hash_password(&params.password)?;
        let user_id = self
            .user_repository
            .create(email_address.as_str(), Some(&password_hash), false)
            .await?;

        // Create tenant membership for the new user.
        let tenant_id = self
            .tenant_repository
            .ensure_membership_for_subject(
                &user_id.to_string(),
                &params.display_name,
                Some(email_address.as_str()),
                params.preferred_tenant_id,
            )
            .await?;

        // Link membership to user_id -- the tenant repository uses subject strings,
        // so we pass user_id as the subject for new users.
        let _ = tenant_id;

        self.auth_event_service
            .record_event(AuthEvent {
                subject: Some(user_id.to_string()),
                event_type: "registration".to_owned(),
                outcome: "success".to_owned(),
                ip_address: params.ip_address,
                user_agent: params.user_agent,
            })
            .await?;

        Ok(user_id)
    }
}
