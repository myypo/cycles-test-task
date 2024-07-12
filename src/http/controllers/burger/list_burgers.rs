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
#[serde(default)]
pub struct Filters {
    limit: Option<i64>,
    offset: Option<i64>,

    first_letter_name: Option<char>,
    tag_name_list: Vec<String>,
    ingredient_name_list: Vec<String>,
}

pub async fn list_burgers(
    cx: State<ApiContext>,
    Query(qry): Query<Filters>,
) -> Response<List<Burger>> {
    let list_burger = sqlx::query!(
    r#"
    with filtered_burgers as (
        select
            b.burger_id
        from
            "burger" b
        left join 
            "burger_tag" bt on b.burger_id = bt.burger_id
        left join
            "tag" t on t.tag_id = bt.tag_id
        left join
            "burger_ingredient" bi on b.burger_id = bi.burger_id
        left join 
            "ingredient" i on i.ingredient_id = bi.ingredient_id
        where
            ($1::varchar is null or b.name ilike $1 collate "en-US-x-icu")
            and (cardinality($2::varchar[]) = 0 or t.name = any($2))
            and (cardinality($3::varchar[]) = 0 or i.name = any($3))
        group by b.burger_id
        limit $4
        offset $5
    )
    select
        b.burger_id as id,
        b.name,
        coalesce(nullif(array_agg(distinct(t.tag_id, t.name)), '{null}'), '{}') as "tag_list!: Vec<(Uuid, String)>" ,
        coalesce(nullif(array_agg(distinct(i.ingredient_id, i.name)), '{null}'), '{}') as "ingredient_list!: Vec<(Uuid, String)>",
        coalesce(nullif(array_agg(distinct(ib.external_image_id)), '{null}'), '{}') as "image_id_list!: Vec<Uuid>" 
    from
        "burger" b
    left join 
        "burger_tag" bt on b.burger_id = bt.burger_id
    left join
        "tag" t on t.tag_id = bt.tag_id
    left join
        "burger_ingredient" bi on b.burger_id = bi.burger_id
    left join 
        "ingredient" i on i.ingredient_id = bi.ingredient_id
    left join
        "image_burger" ib on b.burger_id = ib.burger_id
    where
        b.burger_id in (select burger_id from filtered_burgers)
    group by b.burger_id
    "#,
    qry
        .first_letter_name
        .map(|l| { format!("{}{}", l, '%') }),
    &qry.tag_name_list,
    &qry.ingredient_name_list,
    qry.limit,
    qry.offset
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
