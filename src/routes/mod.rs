pub mod docs;
pub mod health;
pub mod index;

use crate::AppState;
use crate::auth::hmac_auth_middleware;
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    let protected_routes =
        Router::new()
            .route("/", get(index::generate))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                hmac_auth_middleware,
            ));

    let public_routes = Router::new().route("/health", get(health::health_check));
    let swagger_ui = SwaggerUi::new("/docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi());

    // Combine routes
    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .with_state(state)
        .merge(swagger_ui)
}
