mod config;
mod fonts;
mod generator;
mod image;
mod params;
mod routes;

use std::sync::Arc;

/// Runtime image state
#[derive(Clone)]
pub struct ImageState {
    pub fetcher: Arc<image::ImageFetcher>,
    pub fallback: config::ImageFallbackBehavior,
}

#[derive(Clone)]
pub struct AppState {
    pub fontdb: Arc<usvg::fontdb::Database>,
    pub max_input_length: usize,
    pub defaults: config::Defaults,
    pub image: ImageState,
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
        config.image.connect_timeout_secs,
        config.image.total_timeout_secs,
        config.image.max_size_bytes,
        config.image.cache_size,
        config.image.cache_ttl_secs,
        config.image.max_redirects,
        config.image.allow_http,
    )?);

    let state = AppState {
        fontdb: Arc::new(fontdb),
        max_input_length: config.max_input_length,
        defaults: config.defaults,
        image: ImageState {
            fetcher: image_fetcher,
            fallback: config.image.fallback,
        },
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.addr).await?;
    tracing::info!("OGIS server listening on http://{}", config.addr);
    tracing::info!("Swagger UI available at http://{}/docs", config.addr);
    axum::serve(listener, app).await?;
    Ok(())
}
