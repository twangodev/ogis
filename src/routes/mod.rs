pub mod health;
pub mod index;

use crate::AppState;
use axum::{Router, routing::get};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::handler))
        .with_state(state)
}
