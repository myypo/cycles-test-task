use axum::{extract::State, http::StatusCode, Json};

use super::Ingredient;
use crate::http::{
    controllers::{response, Response},
    error::{Error, OnConstraint},
    router::ApiContext,
};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateIngredient {
    name: String,
}

pub async fn create_ingredient(
    cx: State<ApiContext>,
    Json(req): Json<CreateIngredient>,
) -> Response<Ingredient> {
    let id = sqlx::query_scalar!(
        r#"insert into "ingredient" (name) values ($1) returning ingredient_id"#,
        req.name,
    )
    .fetch_one(&cx.db)
    .await
    .on_constraint("ingredient_name_key", |_| {
        Error::conflict([("name", "ingredient with this name already exists")])
    })?;

    response(StatusCode::CREATED, Ingredient { id, name: req.name })
}
