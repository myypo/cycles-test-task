use anyhow::Context;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use burger::{config::Config, http, services::file_storage::S3Client};
use clap::Parser;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf = Config::try_parse()?;

    let aws = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(conf.aws.endpoint_url.clone())
        .credentials_provider(Credentials::new(
            conf.aws.access_key_id.clone(),
            conf.aws.secret_access_key.clone(),
            None,
            None,
            "minio",
        ))
        .region(Region::new(conf.aws.region_name.clone()))
        .load()
        .await;

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&conf.database_url)
        .await
        .with_context(|| "failed to connect to the postgres db")?;
    sqlx::migrate!().run(&db).await?;

    let s3 = S3Client::new(conf.aws.s3.clone(), &aws).await?;

    #[cfg(feature = "fixture")]
    {
        use burger::services::fixture::{ingest_fixture, EdamamClient, FixtureClient};

        if sqlx::query_scalar!(r#"select count(*) from "burger""#)
            .fetch_one(&db)
            .await?
            .unwrap_or_default()
            < 5
        {
            let http = reqwest::Client::new();
            let fix = EdamamClient::new(http, conf.edamam.clone())
                .fixture()
                .await?;
            ingest_fixture(fix, &db, &s3).await?;
        }
    }

    http::serve(conf, db, s3).await
}
