use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};

use crate::http::{
    controllers::{
        tag::{create_tag, list_tags, Tag},
        List, OkBody,
    },
    error::{InputErrBody, LogicErrBody},
};

use super::ApiContext;

pub(crate) fn router(cx: ApiContext) -> ApiRouter {
    const TAG_TAG: &str = "Tags";

    ApiRouter::new()
        .api_route(
            "/",
            post_with(create_tag, |op| {
                op.tag(TAG_TAG)
                    .description("Create a new tag.")
                    .response::<201, OkBody<Tag>>()
                    .response::<400, InputErrBody>()
                    .response_with::<409, LogicErrBody, _>(|res| {
                        res.description("A tag with the provided name already exists")
                    })
            }),
        )
        .api_route(
            "/",
            get_with(list_tags, |op| {
                op.tag(TAG_TAG)
                    .description("Get a list of tags")
                    .response::<200, OkBody<List<Tag>>>()
                    .response::<400, InputErrBody>()
            }),
        )
        .with_state(cx)
}
