mod auth;
mod config;
mod fonts;
mod generator;
mod image;
mod params;
mod routes;
mod templates;
mod yaml_loader;

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
    pub templates: Arc<templates::TemplateMap>,
    pub max_input_length: usize,
    pub defaults: config::Defaults,
    pub image: ImageState,
    pub hmac_validator: Option<Arc<auth::HmacValidator>>,
    pub docs: config::DocsSettings,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let config = config::Config::parse();

    // Export OpenAPI spec and exit if requested
    if config.export_openapi {
        use utoipa::OpenApi;
        let spec = routes::docs::ApiDoc::openapi().to_pretty_json()?;
        println!("{spec}");
        return Ok(());
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load fonts
    let fontdb = fonts::load_fonts();

    // Load templates
    let templates = templates::load_templates();

    // Initialize image fetcher with SSRF protection
    let image_fetcher = Arc::new(image::ImageFetcher::new(
        config.image.connect_timeout_secs,
        config.image.total_timeout_secs,
        config.image.max_size_bytes,
        config.image.effective_cache_size(),
        config.image.cache_ttl_secs,
        config.image.max_redirects,
        config.image.allow_http,
    )?);

    // Initialize HMAC validator if configured
    let hmac_validator = if config.hmac.is_enabled() {
        let secret = config
            .hmac
            .secret_bytes()
            .expect("Secret should exist when enabled");
        tracing::info!("HMAC authentication enabled");
        Some(Arc::new(auth::HmacValidator::new(secret)))
    } else {
        tracing::info!("HMAC authentication disabled");
        None
    };

    let state = AppState {
        fontdb: Arc::new(fontdb),
        templates: Arc::new(templates),
        max_input_length: config.max_input_length,
        defaults: config.defaults,
        image: ImageState {
            fetcher: image_fetcher,
            fallback: config.image.fallback,
        },
        hmac_validator,
        docs: config.docs,
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.addr).await?;
    tracing::info!("ogis server listening on http://{}", config.addr);
    tracing::info!("Swagger UI available at http://{}/docs", config.addr);
    axum::serve(listener, app).await?;
    Ok(())
}
