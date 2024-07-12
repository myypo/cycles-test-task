use aide_axum_typed_multipart::{FieldData, TypedMultipart};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use super::Ingredient;
use crate::{
    domain::ImageList,
    http::{
        controllers::{response, Response, WithImageList},
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

pub async fn add_ingredient_image(
    cx: State<ApiContext>,
    Path(path): Path<Selector>,
    TypedMultipart(mup): TypedMultipart<AddImage>,
) -> Response<WithImageList<Ingredient>> {
    let mut ingredient = sqlx::query!(
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
    .ok_or_else(|| {
        Error::ResourceNotFound("the ingredient chosen for the image upload does not exist")
    })?;

    let mut tx = cx.db.begin().await?;
    let image_id = Uuid::new_v4();
    sqlx::query!(
        r#"insert into "image_ingredient" (external_image_id, ingredient_id) values ($1, $2)"#,
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
        WithImageList {
            inner: Ingredient {
                id: path.id,
                name: ingredient.name,
            },
            image_list: ImageList::fetch_all(cx.file.as_ref(), {
                ingredient.image_id_list.push(image_id);
                ingredient.image_id_list
            })
            .await,
        },
    )
}
