use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::types::Uuid;

use super::Ingredient;
use crate::{
    domain::ImageList,
    http::{
        controllers::{response, Response, WithImageList},
        error::Error,
        router::ApiContext,
    },
};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct Selector {
    id: Uuid,
}

pub async fn get_ingredient_by_id(
    cx: State<ApiContext>,
    Path(path): Path<Selector>,
) -> Response<WithImageList<Ingredient>> {
    let ingredient = sqlx::query!(
        r#"
        select 
            name,
            coalesce(nullif(array_agg(distinct(image_ingredient.external_image_id)), '{null}'), '{}') as "image_id_list!: Vec<Uuid>" 
        from "ingredient"
        left join
            "image_ingredient" on image_ingredient.ingredient_id = $1
        where
            ingredient.ingredient_id = $1
        group by ingredient.ingredient_id
        limit 1
        "#,
        path.id,
    )
    .fetch_optional(&cx.db)
    .await?
    .ok_or_else(|| Error::ResourceNotFound("no ingredient with the provided id"))?;

    response(
        StatusCode::OK,
        WithImageList {
            inner: Ingredient {
                id: path.id,
                name: ingredient.name,
            },
            image_list: ImageList::fetch_all(cx.file.as_ref(), ingredient.image_id_list).await,
        },
    )
}
