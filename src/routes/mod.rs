pub mod health;
pub mod index;

use axum::{routing::get, Router};

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::handler))
}
