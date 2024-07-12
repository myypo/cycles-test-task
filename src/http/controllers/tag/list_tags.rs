use axum::{extract::State, http::StatusCode};
use axum_extra::extract::Query;

use crate::http::{
    controllers::{response, List, Response},
    router::ApiContext,
};

use super::Tag;

#[derive(Default, serde::Deserialize, schemars::JsonSchema)]
pub struct Filters {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn list_tags(cx: State<ApiContext>, Query(qry): Query<Filters>) -> Response<List<Tag>> {
    let list_tag = sqlx::query_as!(
        Tag,
        r#"
        select
            tag_id as id,
            name
        from "tag"
        limit $1
        offset $2
    "#,
        qry.limit,
        qry.offset
    )
    .fetch_all(&cx.db)
    .await?;

    response(StatusCode::OK, List::new(list_tag))
}
