use std::sync::Arc;

use qryvanta_application::EmailService;
use qryvanta_core::AppError;
use qryvanta_infrastructure::{ConsoleEmailService, SmtpEmailConfig, SmtpEmailService};

use crate::api_config::{ApiConfig, EmailProviderConfig};

pub(super) fn build_email_service(config: &ApiConfig) -> Result<Arc<dyn EmailService>, AppError> {
    let service: Arc<dyn EmailService> = match &config.email_provider {
        EmailProviderConfig::Console => Arc::new(ConsoleEmailService::new()),
        EmailProviderConfig::Smtp(smtp) => {
            let smtp_config = SmtpEmailConfig {
                host: smtp.host.clone(),
                port: smtp.port,
                username: smtp.username.clone(),
                password: smtp.password.clone(),
                from_address: smtp.from_address.clone(),
            };
            Arc::new(SmtpEmailService::new(smtp_config)?)
        }
    };

    Ok(service)
}
