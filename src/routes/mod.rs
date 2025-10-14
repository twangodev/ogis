pub mod health;
pub mod og;

use axum::{routing::get, Router};

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health::handler))
        .route("/og", get(og::handler))
}
