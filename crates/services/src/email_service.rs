use salvo::async_trait;
use app_error::AppError;

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), AppError>;
}