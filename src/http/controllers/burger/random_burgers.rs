use axum::{extract::State, http::StatusCode};
use axum_extra::extract::Query;
use futures::stream::FuturesUnordered;
use sqlx::types::Uuid;

use crate::{
    domain::ImageList,
    http::{
        controllers::{response, List, Response},
        router::ApiContext,
    },
};

use super::Burger;

#[derive(Default, serde::Deserialize, schemars::JsonSchema)]
pub struct Filters {
    limit: Option<i64>,
}

pub async fn random_burgers(
    cx: State<ApiContext>,
    Query(qry): Query<Filters>,
) -> Response<List<Burger>> {
    let list_burger = sqlx::query!(
        r#"
        select
            b.burger_id as id,
            b.name,
            coalesce(nullif(array_agg(distinct(t.tag_id, t.name)), '{null}'), '{}') as "tag_list!: Vec<(Uuid, String)>" ,
            coalesce(nullif(array_agg(distinct(i.ingredient_id, i.name)), '{null}'), '{}') as "ingredient_list!: Vec<(Uuid, String)>",
            coalesce(nullif(array_agg(distinct(ib.external_image_id)), '{null}'), '{}') as "image_id_list!: Vec<Uuid>" 
        from "burger" b
        left join 
            "burger_tag" bt on bt.burger_id = b.burger_id
        left join
            "tag" t on t.tag_id = bt.tag_id
        left join
            burger_ingredient bi on b.burger_id = bi.burger_id
        left join 
            ingredient i on bi.ingredient_id = i.ingredient_id
        left join
            "image_burger" ib on b.burger_id = ib.burger_id
        group by id
        order by random()
        limit $1
    "#,
        qry.limit,
    )
    .fetch_all(&cx.db)
    .await?;

    let fc = &cx.file;
    response(
        StatusCode::OK,
        List::from_unordered(
            list_burger
                .into_iter()
                .map(|b| async move {
                    Burger {
                        id: b.id,
                        name: b.name,
                        tag_list: b.tag_list.into_iter().map(|t| t.into()).collect(),
                        ingredient_list: b.ingredient_list.into_iter().map(|i| i.into()).collect(),
                        image_list: ImageList::fetch_all(fc.as_ref(), b.image_id_list).await,
                    }
                })
                .collect::<FuturesUnordered<_>>(),
        )
        .await,
    )
}
