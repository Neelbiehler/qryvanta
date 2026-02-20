//! SMTP email service using the `lettre` crate.

use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use qryvanta_application::EmailService;
use qryvanta_core::{AppError, AppResult};

/// SMTP email service configuration.
#[derive(Clone)]
pub struct SmtpEmailConfig {
    /// SMTP server hostname.
    pub host: String,
    /// SMTP server port.
    pub port: u16,
    /// SMTP username.
    pub username: String,
    /// SMTP password.
    pub password: String,
    /// Sender email address.
    pub from_address: String,
}

/// Production email service using SMTP.
#[derive(Clone)]
pub struct SmtpEmailService {
    config: SmtpEmailConfig,
}

impl SmtpEmailService {
    /// Creates a new SMTP email service.
    #[must_use]
    pub fn new(config: SmtpEmailConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        _html_body: Option<&str>,
    ) -> AppResult<()> {
        let from = self
            .config
            .from_address
            .parse()
            .map_err(|error| AppError::Internal(format!("invalid from address: {error}")))?;

        let to_mailbox = to
            .parse()
            .map_err(|error| AppError::Internal(format!("invalid recipient address: {error}")))?;

        let message = Message::builder()
            .from(from)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(text_body.to_owned())
            .map_err(|error| AppError::Internal(format!("failed to build email: {error}")))?;

        let credentials =
            Credentials::new(self.config.username.clone(), self.config.password.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)
            .map_err(|error| {
                AppError::Internal(format!("failed to create SMTP transport: {error}"))
            })?
            .port(self.config.port)
            .credentials(credentials)
            .build();

        mailer
            .send(message)
            .await
            .map_err(|error| AppError::Internal(format!("failed to send email: {error}")))?;

        Ok(())
    }
}
