use std::{str::FromStr, time::Duration};

use aws_config::SdkConfig;
use aws_sdk_s3::{client::Waiters, presigning::PresigningConfig};
use axum::body::Bytes;
use url::Url;

use crate::config::S3Config;

use super::FileClient;

pub struct S3Client {
    cli: aws_sdk_s3::Client,
    conf: S3Config,
}

impl FileClient for S3Client {
    async fn upload_file(
        &self,
        key: impl Into<String>,
        file: impl Into<Bytes>,
    ) -> anyhow::Result<()> {
        self.cli
            .put_object()
            .bucket(&self.conf.bucket_name)
            .key(key)
            .body(Into::<Bytes>::into(file).into())
            .send()
            .await?;

        Ok(())
    }

    async fn get_file_url(&self, key: impl Into<String>) -> anyhow::Result<Url> {
        Ok(Url::from_str(
            self.cli
                .get_object()
                .key(key)
                .bucket(&self.conf.bucket_name)
                .presigned(PresigningConfig::expires_in(self.conf.expires_in)?)
                .await?
                .uri(),
        )?)
    }
}

impl S3Client {
    pub async fn new(conf: S3Config, aws: &SdkConfig) -> anyhow::Result<Self> {
        let cli = aws_sdk_s3::Client::from_conf(
            aws_sdk_s3::config::Builder::from(aws)
                .force_path_style(true)
                .build(),
        );
        let _ = cli
            .create_bucket()
            .bucket(conf.bucket_name.as_str())
            .send()
            .await;
        cli.wait_until_bucket_exists()
            .bucket(conf.bucket_name.as_str())
            .wait(Duration::from_secs(60))
            .await?;

        Ok(Self { cli, conf })
    }
}
