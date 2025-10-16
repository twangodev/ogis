mod config;
mod fonts;
mod generator;
mod image;
mod routes;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub fontdb: Arc<usvg::fontdb::Database>,
    pub max_input_length: usize,
    pub image_fetcher: Arc<image::ImageFetcher>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let config = config::Config::parse();

    // Load fonts
    let fontdb = fonts::load_fonts();

    // Initialize image fetcher with SSRF protection
    let image_fetcher = Arc::new(image::ImageFetcher::new(
        config.logo_connect_timeout_secs,
        config.logo_total_timeout_secs,
        config.logo_max_size_bytes,
        config.logo_cache_size,
        config.logo_cache_ttl_secs,
        config.logo_max_redirects,
    )?);

    let state = AppState {
        fontdb: Arc::new(fontdb),
        max_input_length: config.max_input_length,
        image_fetcher,
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.addr).await?;
    tracing::info!("OGIS server listening on http://{}", config.addr);
    tracing::info!("Swagger UI available at http://{}/docs", config.addr);
    axum::serve(listener, app).await?;
    Ok(())
}
