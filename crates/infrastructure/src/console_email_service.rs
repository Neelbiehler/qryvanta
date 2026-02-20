//! Console email service for development. Logs emails to tracing output.

use async_trait::async_trait;
use qryvanta_application::EmailService;
use qryvanta_core::AppResult;
use tracing::info;

/// Development email service that logs emails to the console.
#[derive(Clone)]
pub struct ConsoleEmailService;

impl ConsoleEmailService {
    /// Creates a new console email service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConsoleEmailService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmailService for ConsoleEmailService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        _html_body: Option<&str>,
    ) -> AppResult<()> {
        info!(
            to = to,
            subject = subject,
            "--- EMAIL (console) ---\nTo: {}\nSubject: {}\n\n{}\n--- END EMAIL ---",
            to,
            subject,
            text_body
        );

        Ok(())
    }
}
