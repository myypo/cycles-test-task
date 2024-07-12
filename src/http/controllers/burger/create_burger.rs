use axum::{extract::State, http::StatusCode, Json};
use sqlx::{types::Uuid, Transaction};

use super::Burger;
use crate::{
    domain::ImageList,
    http::{
        controllers::{ingredient::Ingredient, response, tag::Tag, Response},
        error::{Error, OnConstraint, Result},
        router::ApiContext,
    },
};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateBurger {
    name: String,
    tag_id_list: Vec<Uuid>,
    ingredient_id_list: Vec<Uuid>,
}

pub async fn create_burger<'a>(
    cx: State<ApiContext>,
    Json(req): Json<CreateBurger>,
) -> Response<Burger> {
    let mut tx = cx.db.begin().await?;

    let id = sqlx::query_scalar!(
        r#"insert into "burger" (name) values ($1) returning burger_id"#,
        req.name,
    )
    .fetch_one(tx.as_mut())
    .await
    .on_constraint("burger_name_key", |_| {
        Error::conflict([("name", "burger with this name already exists")])
    })?;

    batch_insert_tags(&mut tx, id, &req.tag_id_list).await?;
    batch_insert_ingredients(&mut tx, id, &req.ingredient_id_list).await?;

    tx.commit().await?;

    let tag_list = sqlx::query_as!(
        Tag,
        r#"
        select 
            tag_id as id,
            name 
        from "tag" 
        where 
            tag_id = any($1)
    "#,
        &req.tag_id_list
    )
    .fetch_all(&cx.db)
    .await?;
    let ingredient_list = sqlx::query_as!(
        Ingredient,
        r#"
        select 
            ingredient_id as id,
            name 
        from "ingredient" 
        where 
            ingredient_id = any($1)
    "#,
        &req.ingredient_id_list
    )
    .fetch_all(&cx.db)
    .await?;

    response(
        StatusCode::CREATED,
        Burger {
            id,
            name: req.name,
            tag_list,
            ingredient_list,
            image_list: ImageList::default(),
        },
    )
}

async fn batch_insert_tags(
    tx: &mut Transaction<'static, sqlx::Postgres>,
    burger_id: Uuid,
    tag_id_list: &[Uuid],
) -> Result<()> {
    if tag_id_list.is_empty() {
        return Ok(());
    }

    sqlx::query!(
        r#"
        insert into "burger_tag" (burger_id, tag_id)
        SELECT * FROM unnest($1::uuid[], $2::uuid[])
    "#,
        &vec![burger_id; tag_id_list.len()],
        tag_id_list,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

async fn batch_insert_ingredients(
    tx: &mut Transaction<'static, sqlx::Postgres>,
    burger_id: Uuid,
    ingr_id_list: &[Uuid],
) -> Result<()> {
    if ingr_id_list.is_empty() {
        return Ok(());
    }

    sqlx::query!(
        r#"
        insert into "burger_ingredient" (burger_id, ingredient_id)
        SELECT * FROM unnest($1::uuid[], $2::uuid[])
    "#,
        &vec![burger_id; ingr_id_list.len()],
        ingr_id_list,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
