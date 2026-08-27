use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

use super::ApiDoc;

/// Serves the raw specification at `/openapi.json` and Swagger UI at `/docs`.
pub fn router() -> Router<AppState> {
    Router::new().merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
}
