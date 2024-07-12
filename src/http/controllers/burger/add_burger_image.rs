use aide_axum_typed_multipart::{FieldData, TypedMultipart};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use super::Burger;
use crate::{
    domain::ImageList,
    http::{
        controllers::{response, Response},
        error::Error,
        router::ApiContext,
    },
    services::file_storage::FileClient,
};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct Selector {
    id: Uuid,
}

#[derive(axum_typed_multipart::TryFromMultipart, schemars::JsonSchema)]
pub struct AddImage {
    #[form_data(limit = "5MiB")]
    image: FieldData<Bytes>,
}

pub async fn add_burger_image(
    cx: State<ApiContext>,
    Path(path): Path<Selector>,
    TypedMultipart(mup): TypedMultipart<AddImage>,
) -> Response<Burger> {
    let mut burger = sqlx::query!(
        r#"
        select 
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
        where
            b.burger_id = $1
        group by b.burger_id
        limit 1
        "#,
        path.id,
    )
    .fetch_optional(&cx.db)
    .await?
    .ok_or_else(|| Error::ResourceNotFound("the burger chosen for the image upload does not exist"))?;

    let mut tx = cx.db.begin().await?;
    let image_id = Uuid::new_v4();
    sqlx::query!(
        r#"insert into "image_burger" (external_image_id, burger_id) values ($1, $2)"#,
        image_id,
        path.id,
    )
    .execute(tx.as_mut())
    .await?;
    cx.file
        .as_ref()
        .upload_file(path.id, mup.image.contents.clone())
        .await?;

    tx.commit().await?;

    response(
        StatusCode::CREATED,
        Burger {
            id: path.id,
            name: burger.name,
            tag_list: burger.tag_list.into_iter().map(|t| t.into()).collect(),
            ingredient_list: burger
                .ingredient_list
                .into_iter()
                .map(|i| i.into())
                .collect(),
            image_list: ImageList::fetch_all(cx.file.as_ref(), {
                burger.image_id_list.push(image_id);
                burger.image_id_list
            })
            .await,
        },
    )
}
