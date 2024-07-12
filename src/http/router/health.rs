use aide::{
    axum::{routing::get_with, ApiRouter},
    transform::TransformOperation,
};

use crate::http::controllers::health::check_health;

pub(crate) fn router() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        get_with(check_health, |op: TransformOperation| {
            const TAG_HEALTH: &str = "Health";

            op.tag(TAG_HEALTH)
                .description("Check health of the service.")
                .response::<200, ()>()
        }),
    )
}
