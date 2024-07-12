use axum::body::Bytes;
use url::Url;

mod s3;
pub use s3::S3Client;

pub trait FileClient {
    fn upload_file(
        &self,
        key: impl Into<String>,
        file: impl Into<Bytes>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>>;
    fn get_file_url(
        &self,
        key: impl Into<String>,
    ) -> impl std::future::Future<Output = anyhow::Result<Url>>;
}
