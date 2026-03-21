use app_error::AppError;
use salvo::async_trait;
use typedef::UploadResult;

#[async_trait]
pub trait StorageService: Send + Sync {
    async fn upload(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<UploadResult, AppError>;

    async fn delete(&self, key: &str) -> Result<(), AppError>;

    fn get_url(&self, key: &str) -> String; // client-facing URL
}