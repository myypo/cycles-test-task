use futures::{stream::FuturesUnordered, StreamExt};

pub mod docs;
pub mod health;

pub mod burger;
pub mod ingredient;
pub mod tag;

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct Body<T> {
    data: T,
}

pub type OkBody<T> = axum::Json<Body<T>>;

pub type Response<T> = Result<(axum::http::StatusCode, OkBody<T>), crate::http::error::Error>;

fn response<T>(code: axum::http::StatusCode, data: T) -> Response<T> {
    Ok((code, axum::Json(Body { data })))
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct List<T> {
    count: usize,
    list: Vec<T>,
}

impl<T> List<T> {
    pub fn new(list: Vec<T>) -> Self {
        Self {
            count: list.len(),
            list,
        }
    }

    pub async fn from_unordered(
        stream: FuturesUnordered<impl futures::Future<Output = T>>,
    ) -> Self {
        Self {
            count: stream.len(),
            list: stream.collect::<Vec<T>>().await,
        }
    }
}

pub type WithImageList<T> = crate::domain::WithImageList<T, sqlx::types::Uuid>;
