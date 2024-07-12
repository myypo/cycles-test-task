use crate::{
    config::Config,
    services::file_storage::{FileClient, S3Client},
};
use aide::{axum::ApiRouter, openapi::OpenApi};
use anyhow::Context;
use axum::{
    body::Body,
    extract::MatchedPath,
    http::{header::AUTHORIZATION, Request},
    Extension,
};
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use super::controllers::docs;

pub(crate) struct GenericApiContext<F: FileClient> {
    pub file: Arc<F>,
    pub db: PgPool,
}

impl<F: FileClient> Clone for GenericApiContext<F> {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            db: self.db.clone(),
        }
    }
}

pub type ApiContext = GenericApiContext<S3Client>;

mod burgers;
mod health;
mod ingredients;
mod tags;
pub async fn serve(conf: Config, db: PgPool, file: S3Client) -> anyhow::Result<()> {
    let listener = TcpListener::bind(conf.server_addr).await?;

    let mut api = setup_openapi();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = {
        let cx = ApiContext {
            file: Arc::new(file),
            db,
        };

        ApiRouter::new()
            .nest_api_service(
                "/api/v1",
                ApiRouter::new()
                    .nest_api_service("/health", health::router())
                    .nest_api_service("/docs", docs::router())
                    .nest_api_service("/burgers", burgers::router(cx.clone()))
                    .nest_api_service("/ingredients", ingredients::router(cx.clone()))
                    .nest_api_service("/tags", tags::router(cx.clone())),
            )
            .finish_api_with(&mut api, |op| op.title("Burger Open API documentation."))
            .layer((
                Extension(Arc::new(api)),
                SetSensitiveHeadersLayer::new([AUTHORIZATION]),
                CompressionLayer::new(),
                TraceLayer::new_for_http().make_span_with(|r: &Request<Body>| {
                    let matched_path = r.extensions().get::<MatchedPath>().map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?r.method(),
                        matched_path,
                    )
                }),
                TimeoutLayer::new(Duration::from_secs(20)),
                CatchPanicLayer::new(),
            ))
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("error running HTTP server")
}

fn setup_openapi() -> OpenApi {
    aide::gen::on_error(|error| {
        panic!("{error}");
    });
    aide::gen::extract_schemas(true);
    OpenApi::default()
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
