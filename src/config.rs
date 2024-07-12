use std::{net::SocketAddr, time::Duration};

#[derive(clap::Parser)]
pub struct Config {
    #[cfg(feature = "fixture")]
    #[clap(flatten)]
    pub edamam: EdamamConfig,

    #[clap(env)]
    #[cfg_attr(
        feature = "dev",
        clap(default_value = "postgresql://postgres:postgres@127.0.0.1:5432/burger")
    )]
    pub database_url: String,

    #[clap(env)]
    #[cfg_attr(feature = "dev", clap(default_value = "127.0.0.1:8080"))]
    pub server_addr: SocketAddr,

    #[clap(flatten)]
    pub aws: AwsConfig,
}

#[derive(clap::Parser)]
pub struct AwsConfig {
    #[clap(env = "AWS_ACCESS_KEY_ID")]
    #[cfg_attr(feature = "dev", clap(default_value = "minio"))]
    pub access_key_id: String,

    #[clap(env = "AWS_SECRET_ACCESS_KEY")]
    #[cfg_attr(feature = "dev", clap(default_value = "burger-minio"))]
    pub secret_access_key: String,

    #[clap(env = "AWS_ENDPOINT_URL")]
    #[cfg_attr(feature = "dev", clap(default_value = "http://127.0.0.1:9000/"))]
    pub endpoint_url: String,

    #[clap(env = "AWS_REGION_NAME")]
    #[cfg_attr(feature = "dev", clap(default_value = "minio"))]
    pub region_name: String,

    #[clap(flatten)]
    pub s3: S3Config,
}

#[derive(Clone, clap::Parser)]
pub struct S3Config {
    #[clap(env = "AWS_S3_BUCKET_NAME")]
    #[cfg_attr(feature = "dev", clap(default_value = "burger"))]
    pub bucket_name: String,

    #[clap(env = "AWS_S3_EXPIRES_IN_MINUTES")]
    #[clap(default_value = "180", value_parser = |s: &str| s.parse().map(Duration::from_mins))]
    pub expires_in: Duration,
}

#[derive(Clone, clap::Parser)]
pub struct EdamamConfig {
    #[clap(env = "EDAMAM_ID_APP")]
    pub id_app: String,

    #[clap(env = "EDAMAM_KEY_APP")]
    pub key_app: String,
}
