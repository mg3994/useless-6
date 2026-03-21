use std::sync::OnceLock;
use infrastructure::config::s3_config::S3Config;
use infrastructure::external::s3_service::S3Service;

pub static S3: OnceLock<S3Service> = OnceLock::new();

pub async fn init(config: &S3Config) {
    let service = S3Service::new(config).await;

    S3.set(service)
        .expect("S3 service should be set");
}

pub fn get() -> &'static S3Service {
    S3.get().expect("S3 service not initialized")
}