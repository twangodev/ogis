pub mod docs;
pub mod health;
pub mod index;

use crate::AppState;
use crate::auth::hmac_auth_middleware;
use axum::{Router, middleware, routing::get};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    let swagger_ui = SwaggerUi::new("/docs").url("/docs/openapi.json", docs::ApiDoc::openapi());
    let docs_router = Router::new().merge(swagger_ui);

    // Apply CORS to Swagger endpoints if origins are configured
    let docs_router = if let Some(origins) = state.docs.allowed_origins() {
        let cors_layer = origins
            .iter()
            .filter_map(|origin| origin.parse::<axum::http::HeaderValue>().ok())
            .fold(CorsLayer::new(), |cors, header| cors.allow_origin(header))
            .allow_methods([axum::http::Method::GET]);

        docs_router.layer(cors_layer)
    } else {
        docs_router
    };

    Router::new()
        .route("/", get(index::generate))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            hmac_auth_middleware, // Apply HMAC middleware to routes above
        ))
        .route("/health", get(health::health_check))
        .merge(docs_router)
        .with_state(state)
}
