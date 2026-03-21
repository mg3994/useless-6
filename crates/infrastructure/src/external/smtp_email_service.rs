
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use lettre::message::{header, SinglePart};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use salvo::async_trait;
use app_error::AppError;
use services::email_service::EmailService;
use crate::config::SmtpConfig;

#[derive(Debug)]
pub struct SmtpEmailService {
    config: SmtpConfig,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpEmailService {
    pub fn new(config: SmtpConfig) -> Self {
        let creds = Credentials::new(
            config.auth.user.clone(),
            config.auth.pass.clone(),
        );

        let tls_parameters = TlsParameters::new(config.host.clone())
            .expect("Failed to create TLS parameters");

        let transport = if config.secure {
            // Port 465 usually uses Implicit TLS (Wrapper)
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .expect("Failed to create SMTP relay")
                .credentials(creds)
                .port(config.port)
                .tls(Tls::Wrapper(tls_parameters))
                .build()
        } else {
            // Port 587 usually uses STARTTLS
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .expect("Failed to create SMTP relay")
                .credentials(creds)
                .port(config.port)
                .tls(Tls::Required(tls_parameters))
                .build()
        };

        Self { config, transport }
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        is_html: Option<bool>,
    ) -> Result<(), AppError> {
        let is_html = is_html.unwrap_or(false);
        let from = self.config.from.parse().map_err(|e| {
            AppError::Internal(format!("Invalid 'from' address: {}", e))
        })?;

        let to_addr = to.parse().map_err(|e| {
            AppError::Internal(format!("Invalid 'to' address: {}", e))
        })?;

        let content_type = if is_html {
            header::ContentType::TEXT_HTML
        } else {
            header::ContentType::TEXT_PLAIN
        };

        let email = Message::builder()
            .from(from)
            .to(to_addr)
            .subject(subject)
            .singlepart(
                SinglePart::builder().header(content_type).body(body.to_string())
            )

            .map_err(|e| AppError::Internal(format!("Failed to build email: {}", e)))?;

        self.transport.send(email).await
            .map_err(|e| AppError::Internal(format!("Failed to send email: {}", e)))?;

        Ok(())
    }
}