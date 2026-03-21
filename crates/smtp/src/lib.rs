use std::sync::OnceLock;
use infrastructure::config::SmtpConfig;
use infrastructure::external::smtp_email_service::SmtpEmailService;

pub static SMTP: OnceLock<SmtpEmailService> = OnceLock::new();

pub fn init(config: SmtpConfig) {
    let service = SmtpEmailService::new(config);

    SMTP.set(service)
        .expect("SMTP service should be set");
}

pub fn get() -> &'static SmtpEmailService {
    SMTP.get().expect("SMTP not initialized")
}