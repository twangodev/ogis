pub mod docs;
pub mod health;
pub mod index;

use crate::AppState;
use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::health_check))
        .with_state(state)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
}
