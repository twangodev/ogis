pub mod docs;
pub mod health;
pub mod index;

use crate::AppState;
use crate::auth::hmac_auth_middleware;
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    let swagger_ui = SwaggerUi::new("/docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi());

    Router::new()
        .route("/", get(index::generate))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            hmac_auth_middleware, // Apply HMAC middleware to routes above
        ))
        .route("/health", get(health::health_check))
        .merge(swagger_ui)
        .with_state(state)
}
