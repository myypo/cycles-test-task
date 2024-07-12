use aide::{
    axum::{
        routing::{get_with, post_with},
        ApiRouter,
    },
    transform::TransformOperation,
};

use crate::http::{
    controllers::{
        burger::{
            add_burger_image, create_burger, get_burger_by_id, list_burgers, random_burgers, Burger,
        },
        List, OkBody,
    },
    error::{InputErrBody, LogicErrBody},
};

use super::ApiContext;

pub(crate) fn router(cx: ApiContext) -> ApiRouter {
    const TAG_BURGER: &str = "Burgers";

    ApiRouter::new()
        .api_route(
            "/",
            post_with(create_burger, |op: TransformOperation| {
                op.tag(TAG_BURGER)
                    .description(
                        "Create a new burger passing its data alongside ingredients and tags.",
                    )
                    .response::<201, OkBody<Burger>>()
                    .response::<400, InputErrBody>()
                    .response_with::<409, LogicErrBody, _>(|res| {
                        res.description("A burger with the provided name already exists.")
                    })
            }),
        )
        .api_route(
            "/:id/images",
            post_with(add_burger_image, |op| {
                op.tag(TAG_BURGER)
                    .description("Add a new burger image to the existsing ones.")
                    .response::<201, OkBody<Burger>>()
                    .response::<400, InputErrBody>()
                    .response::<404, LogicErrBody>()
            }),
        )
        .api_route(
            "/:id",
            get_with(get_burger_by_id, |op| {
                op.tag(TAG_BURGER)
                    .description("Get a single burger by its internal ID.")
                    .response::<200, OkBody<Burger>>()
                    .response::<400, InputErrBody>()
                    .response::<404, LogicErrBody>()
            }),
        )
        .api_route(
            "/",
            get_with(list_burgers, |op| {
                op.tag(TAG_BURGER)
                    .description("Get a list of burgers according to filters.")
                    .response::<200, OkBody<List<Burger>>>()
                    .response::<400, InputErrBody>()
            }),
        )
        .api_route(
            "/random",
            get_with(random_burgers, |op| {
                op.tag(TAG_BURGER)
                    .description("Get random burgers.")
                    .response::<200, OkBody<List<Burger>>>()
                    .response::<400, InputErrBody>()
            }),
        )
        .with_state(cx)
}
