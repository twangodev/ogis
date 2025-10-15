mod config;
mod fonts;
mod generator;
mod routes;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub fontdb: Arc<usvg::fontdb::Database>,
}

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let config = config::Config::parse();

    // Load fonts
    let fontdb = fonts::load_fonts();

    let state = AppState {
        fontdb: Arc::new(fontdb),
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.addr).await.unwrap();
    tracing::info!("OGIS server listening on http://{}", config.addr);
    tracing::info!("Swagger UI available at http://{}/docs", config.addr);
    axum::serve(listener, app).await.unwrap();
}
