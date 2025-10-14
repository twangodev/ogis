pub mod docs;
pub mod health;
pub mod index;

use crate::AppState;
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::handler))
        .with_state(state)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
}
