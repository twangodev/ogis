mod config;
mod fonts;
mod generator;
mod image;
mod params;
mod routes;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub fontdb: Arc<usvg::fontdb::Database>,
    pub max_input_length: usize,
    pub image_fetcher: Arc<image::ImageFetcher>,
    pub image_fallback: config::ImageFallbackBehavior,
    pub default_title: String,
    pub default_description: String,
    pub default_subtitle: String,
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
        config.allow_http,
    )?);

    let state = AppState {
        fontdb: Arc::new(fontdb),
        max_input_length: config.max_input_length,
        image_fetcher,
        image_fallback: config.image_fallback,
        default_title: config.default_title,
        default_description: config.default_description,
        default_subtitle: config.default_subtitle,
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.addr).await?;
    tracing::info!("OGIS server listening on http://{}", config.addr);
    tracing::info!("Swagger UI available at http://{}/docs", config.addr);
    axum::serve(listener, app).await?;
    Ok(())
}
