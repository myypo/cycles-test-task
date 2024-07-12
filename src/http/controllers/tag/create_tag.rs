use axum::{extract::State, http::StatusCode, Json};

use crate::http::{
    controllers::{response, Response},
    error::{Error, OnConstraint},
    router::ApiContext,
};

use super::Tag;

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateTag {
    name: String,
}

pub async fn create_tag(cx: State<ApiContext>, Json(req): Json<CreateTag>) -> Response<Tag> {
    let id = sqlx::query_scalar!(
        r#"insert into "tag" (name) values ($1) returning tag_id"#,
        req.name,
    )
    .fetch_one(&cx.db)
    .await
    .on_constraint("tag_name_key", |_| {
        Error::conflict([("name", "tag with this name already exists")])
    })?;

    response(StatusCode::CREATED, Tag { id, name: req.name })
}
