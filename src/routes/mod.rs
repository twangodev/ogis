pub mod health;
pub mod index;

use axum::{routing::get, Router};
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health::handler))
        .with_state(state)
}
