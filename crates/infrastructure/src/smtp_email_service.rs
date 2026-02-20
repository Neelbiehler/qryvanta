//! SMTP email service using the `lettre` crate.

use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart, SinglePart, header::ContentType};
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
    from_address: Mailbox,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpEmailService {
    /// Creates a new SMTP email service.
    pub fn new(config: SmtpEmailConfig) -> AppResult<Self> {
        let from_address = config
            .from_address
            .parse()
            .map_err(|error| AppError::Validation(format!("invalid SMTP_FROM_ADDRESS: {error}")))?;

        let credentials = Credentials::new(config.username, config.password);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .map_err(|error| {
                AppError::Validation(format!("failed to create SMTP transport: {error}"))
            })?
            .port(config.port)
            .credentials(credentials)
            .timeout(Some(std::time::Duration::from_secs(10)))
            .build();

        Ok(Self {
            from_address,
            mailer,
        })
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        html_body: Option<&str>,
    ) -> AppResult<()> {
        if subject.contains('\r') || subject.contains('\n') {
            return Err(AppError::Validation(
                "email subject must not contain newline characters".to_owned(),
            ));
        }

        let to_mailbox = to
            .parse()
            .map_err(|error| AppError::Validation(format!("invalid recipient address: {error}")))?;

        let message_builder = Message::builder()
            .from(self.from_address.clone())
            .to(to_mailbox)
            .subject(subject);

        let plain_text = SinglePart::builder()
            .header(ContentType::TEXT_PLAIN)
            .body(text_body.to_owned());

        let message = if let Some(html_body) = html_body {
            let html_part = SinglePart::builder()
                .header(ContentType::TEXT_HTML)
                .body(html_body.to_owned());

            message_builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(plain_text)
                        .singlepart(html_part),
                )
                .map_err(|error| AppError::Internal(format!("failed to build email: {error}")))?
        } else {
            message_builder
                .singlepart(plain_text)
                .map_err(|error| AppError::Internal(format!("failed to build email: {error}")))?
        };

        self.mailer
            .send(message)
            .await
            .map_err(|error| AppError::Internal(format!("failed to send email: {error}")))?;

        Ok(())
    }
}
