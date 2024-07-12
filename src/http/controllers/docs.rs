use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    openapi::OpenApi,
};
use axum::{
    response::IntoResponse,
    routing::{get, get_service},
    Extension, Json,
};
use std::sync::Arc;
use tower_http::services::ServeFile;

pub(crate) fn router() -> ApiRouter {
    const TAG_DOCS: &str = "Documentation";

    aide::gen::infer_responses(true);

    async fn serve_json(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
        Json(api).into_response()
    }

    let router: ApiRouter = ApiRouter::new()
        .route("/api.json", get(serve_json))
        .route(
            "/swagger",
            get_service(ServeFile::new("assets/swagger.html")),
        )
        .api_route(
            "/scalar",
            get_with(
                aide::scalar::Scalar::new("/api/v1/docs/api.json")
                    .with_title("Scalar UI API documentation.")
                    .axum_handler(),
                |op| {
                    op.tag(TAG_DOCS)
                        .description("View Scalar generated html API documentation.")
                },
            ),
        );

    aide::gen::infer_responses(false);

    router
}
