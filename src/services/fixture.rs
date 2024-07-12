use axum::body::Bytes;
use futures::{stream::FuturesUnordered, Future, StreamExt};
use sqlx::PgPool;
use uuid::Uuid;

mod edamam;
pub use edamam::EdamamClient;

use super::file_storage::FileClient;

pub trait FixtureClient {
    fn fixture(&self) -> impl Future<Output = anyhow::Result<Fixture>>;
}

#[derive(PartialEq, Eq, Hash)]
pub struct FixtureTag {
    id: Uuid,
    name: String,
}

#[derive(PartialEq, Eq, Hash)]
pub struct FixtureIngredient {
    id: Uuid,
    name: String,
}

#[derive(PartialEq, Eq, Hash)]
pub struct FixtureBurger {
    name: String,
    tag_id_list: Vec<Uuid>,
    ingredient_id_list: Vec<Uuid>,
    image_list: Vec<(Uuid, Bytes)>,
}

pub struct Fixture {
    burger_list: Vec<FixtureBurger>,
    tag_list: Vec<FixtureTag>,
    ingredient_list: Vec<FixtureIngredient>,
}

pub async fn ingest_fixture(fix: Fixture, db: &PgPool, fc: &impl FileClient) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;

    let tag_list = fix.tag_list;
    sqlx::query!(
        r#"
        insert into "tag" (tag_id, name) 
            select * from unnest($1::uuid[], $2::varchar[])
        "#,
        &tag_list.iter().map(|t| t.id).collect::<Vec<Uuid>>(),
        &tag_list
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<String>>(),
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        r#"
        insert into "ingredient" (ingredient_id, name)
            select * from unnest($1::uuid[], $2::varchar[])
        "#,
        &fix.ingredient_list
            .iter()
            .map(|t| t.id)
            .collect::<Vec<Uuid>>(),
        &fix.ingredient_list
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<String>>(),
    )
    .execute(tx.as_mut())
    .await?;

    let mut image_stream = fix
        .burger_list
        .iter()
        .map(|b| b.image_list.clone())
        .map(|il| async move {
            il.into_iter()
                .map(|i| fc.upload_file(i.0, i.1))
                .collect::<FuturesUnordered<_>>()
        })
        .collect::<FuturesUnordered<_>>();
    while let Some(mut f) = image_stream.next().await {
        while let Some(i) = f.next().await {
            tracing::debug!("image fixt upload: {:?}", i);
        }
    }

    for FixtureBurger {
        name,
        tag_id_list,
        ingredient_id_list,
        image_list,
    } in fix.burger_list
    {
        let burger_id = sqlx::query_scalar!(
            r#"insert into "burger" (name) values ($1) returning burger_id"#,
            name,
        )
        .fetch_one(tx.as_mut())
        .await?;

        sqlx::query!(
            r#"
        insert into "burger_tag" (burger_id, tag_id)
        select * from unnest($1::uuid[], $2::uuid[])
    "#,
            &vec![burger_id; tag_id_list.len()],
            &tag_id_list,
        )
        .execute(tx.as_mut())
        .await?;

        sqlx::query!(
            r#"
        insert into "burger_ingredient" (burger_id, ingredient_id)
        select * from unnest($1::uuid[], $2::uuid[])
    "#,
            &vec![burger_id; ingredient_id_list.len()],
            &ingredient_id_list,
        )
        .execute(tx.as_mut())
        .await?;

        let len_image = image_list.len();
        sqlx::query!(
            r#"
            insert into "image_burger" (external_image_id, burger_id) 
                select * from unnest($1::uuid[], $2::uuid[])
            "#,
            &image_list.iter().map(|i| i.0).collect::<Vec<Uuid>>(),
            &vec![burger_id; len_image],
        )
        .execute(tx.as_mut())
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
