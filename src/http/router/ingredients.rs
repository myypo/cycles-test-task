use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};

use crate::http::{
    controllers::ingredient::{
        add_ingredient_image, create_ingredient, get_ingredient_by_id, list_ingredients, Ingredient,
    },
    error::InputErrBody,
};
use crate::http::{
    controllers::{List, OkBody, WithImageList},
    error::LogicErrBody,
};

use super::ApiContext;

pub(crate) fn router(cx: ApiContext) -> ApiRouter {
    const TAG_INGREDIENT: &str = "Ingredients";

    ApiRouter::new()
        .api_route(
            "/",
            post_with(create_ingredient, |op| {
                op.tag(TAG_INGREDIENT)
                    .description("Create a new ingredient.")
                    .response::<201, OkBody<Ingredient>>()
                    .response::<400, InputErrBody>()
                    .response_with::<409, LogicErrBody, _>(|res| {
                        res.description("An ingredient with the provided name already exists.")
                    })
            }),
        )
        .api_route(
            "/:id/images",
            post_with(add_ingredient_image, |op| {
                op.tag(TAG_INGREDIENT)
                    .description("Add a new ingredient image to the existsing ones.")
                    .response::<201, OkBody<WithImageList<Ingredient>>>()
                    .response::<400, InputErrBody>()
                    .response::<404, LogicErrBody>()
            }),
        )
        .api_route(
            "/:id",
            get_with(get_ingredient_by_id, |op| {
                op.tag(TAG_INGREDIENT)
                    .description("Get an ingredient by its internal ID.")
                    .response::<200, OkBody<WithImageList<Ingredient>>>()
                    .response::<400, InputErrBody>()
                    .response::<404, LogicErrBody>()
            }),
        )
        .api_route(
            "/",
            get_with(list_ingredients, |op| {
                op.tag(TAG_INGREDIENT)
                    .description("Get a list of ingredients.")
                    .response::<200, OkBody<List<WithImageList<Ingredient>>>>()
                    .response::<400, InputErrBody>()
            }),
        )
        .with_state(cx)
}
