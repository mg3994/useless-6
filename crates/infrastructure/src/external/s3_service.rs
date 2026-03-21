use aws_sdk_s3::{Client, config::Region};
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use crate::config::S3Config;
use app_error::AppError;
use salvo::async_trait;
use services::s3_service::StorageService;
use typedef::UploadResult;

#[derive(Debug)]
pub struct S3Service {
    client: Client,
    bucket: String,
    endpoint: String,
}

impl S3Service {
    pub async fn new(config: &S3Config) -> Self {
        let creds = Credentials::new(
            config.access_key.clone(),
            config.secret_key.clone(),
            None,
            None,
            "static",
        );

        let region = Region::new(config.region.clone());

        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(creds)
            .load()
            .await;

        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&shared_config);

        // Custom endpoint (MinIO / R2 etc.)
        if !config.endpoint.is_empty() {
            s3_config_builder = s3_config_builder.endpoint_url(config.endpoint.clone());
        }

        let client = Client::from_conf(s3_config_builder.build());

        Self {
            client,
            bucket: config.bucket.clone(),
            endpoint: config.endpoint.clone(),
        }
    }
}
#[async_trait]
impl StorageService for S3Service {
    async fn upload(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<UploadResult, AppError> {
        let resp = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 upload failed: {}", e)))?;

        let etag = resp.e_tag().map(|s| s.trim_matches('"').to_string());

        let url = format!(
            "{}/{}/{}",
            self.endpoint.trim_end_matches('/'),
            self.bucket,
            key
        );

        Ok(UploadResult { url, etag })
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 delete failed: {}", e)))?;

        Ok(())
    }

    fn get_url(&self, key: &str) -> String {
        format!(
            "{}/{}/{}",
            self.endpoint.trim_end_matches('/'),
            self.bucket,
            key
        )
    }
}