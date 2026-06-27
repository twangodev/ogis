pub mod compressed;
pub mod docs;
pub mod health;
pub mod index;
pub mod render;
pub mod templates;
pub mod timing;

#[cfg(test)]
mod c_e2e;

use crate::AppState;
use crate::auth::hmac_auth_middleware;
use crate::telemetry::metrics_middleware;
use axum::{Router, middleware, routing::get};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    let swagger_ui = SwaggerUi::new("/docs").url("/docs/openapi.json", docs::ApiDoc::openapi());
    let docs_router = Router::new().merge(swagger_ui);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(index::generate))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            hmac_auth_middleware, // Apply HMAC middleware to routes above
        ))
        .route("/c/{blob}", get(compressed::generate_compressed))
        .route(
            "/c/{blob}/{sig}",
            get(compressed::generate_compressed_signed),
        )
        .route_layer(middleware::from_fn(metrics_middleware))
        .route("/health", get(health::health_check))
        .route("/templates", get(templates::list_templates))
        .merge(docs_router)
        .layer(cors)
        .with_state(state)
}
