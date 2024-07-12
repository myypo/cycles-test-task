use axum::{extract::State, http::StatusCode};
use axum_extra::extract::Query;
use futures::stream::FuturesUnordered;
use sqlx::types::Uuid;

use crate::{
    domain::ImageList,
    http::{
        controllers::{response, List, Response, WithImageList},
        router::ApiContext,
    },
};

use super::Ingredient;

#[derive(Default, serde::Deserialize, schemars::JsonSchema)]
pub struct Filters {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn list_ingredients(
    cx: State<ApiContext>,
    Query(qry): Query<Filters>,
) -> Response<List<WithImageList<Ingredient>>> {
    let list_ingredient = sqlx::query!(
        r#"
        select
            i.ingredient_id as id,
            i.name,
            coalesce(nullif(array_agg(distinct(ii.external_image_id)), '{null}'), '{}') as "image_id_list!: Vec<Uuid>" 
        from "ingredient" i
        left join
            "image_ingredient" ii on i.ingredient_id = ii.ingredient_id
        group by id
        limit $1
        offset $2
    "#,
        qry.limit,
        qry.offset
    )
    .fetch_all(&cx.db)
    .await?;

    let fc = &cx.file;
    response(
        StatusCode::OK,
        List::from_unordered(
            list_ingredient
                .into_iter()
                .map(|i| async move {
                    WithImageList {
                        inner: Ingredient {
                            id: i.id,
                            name: i.name,
                        },
                        image_list: ImageList::fetch_all(fc.as_ref(), i.image_id_list).await,
                    }
                })
                .collect::<FuturesUnordered<_>>(),
        )
        .await,
    )
}
