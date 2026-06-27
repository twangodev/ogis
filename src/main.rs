mod auth;
mod config;
mod error;
mod fonts;
mod generator;
mod image;
mod params;
mod routes;
mod telemetry;
mod templates;
mod wire;
mod yaml_loader;

use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_shutdown::Shutdown;

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
    pub render_semaphore: Arc<Semaphore>,
    pub gradient_cache: Arc<generator::GradientCache>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Initialize telemetry BEFORE tokio runtime starts
    // (blocking reqwest client can't be created inside async context)
    let _telemetry_guard = telemetry::init(&config.otel)?;

    // Build and run the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_server(config))
}

/// Assemble an `AppState` from a `Config`.  Does NOT start telemetry, bind
/// the listener, or launch warm-up tasks - those remain in `run_server`.
pub(crate) fn build_state(config: &config::Config) -> Result<AppState, Box<dyn std::error::Error>> {
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

    // Create render semaphore sized to CPU cores
    let render_concurrency = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    tracing::info!("Render concurrency: {}", render_concurrency);

    let gradient_cache_bytes = config.gradient_cache.effective_max_bytes();
    tracing::info!(
        "Gradient cache budget: {} MB",
        gradient_cache_bytes / (1024 * 1024)
    );
    let gradient_cache = Arc::new(generator::GradientCache::new(gradient_cache_bytes));

    Ok(AppState {
        fontdb: Arc::new(fontdb),
        templates: Arc::new(templates),
        max_input_length: config.max_input_length,
        defaults: config.defaults.clone(),
        image: ImageState {
            fetcher: image_fetcher,
            fallback: config.image.fallback,
        },
        hmac_validator,
        docs: config.docs.clone(),
        render_semaphore: Arc::new(Semaphore::new(render_concurrency)),
        gradient_cache,
    })
}

async fn run_server(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let state = build_state(&config)?;

    // Spawn gradient-cache warm-up in the background; the listener binds
    // immediately and the lazy fallback handles anything not warmed.
    spawn_gradient_warmup(&state, &config.gradient_cache.warmup_templates);

    let app = routes::create_router(state);

    let shutdown = Shutdown::new()?;

    let listener = tokio::net::TcpListener::bind(&config.addr).await?;
    tracing::info!("ogis server listening on http://{}", config.addr);
    tracing::info!("Swagger UI available at http://{}/docs", config.addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown.handle().await;
            tracing::info!("Shutdown signal received, draining connections...");
        })
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// Spawn a background task that prerenders the listed gradient templates into
/// the cache. Skips empty / unknown / non-gradient names with a warning.
fn spawn_gradient_warmup(state: &AppState, names: &[String]) {
    // Trim then filter so comma-separated env input like "a, b" doesn't keep
    // leading whitespace and silently miss valid template names.
    let names: Vec<String> = names
        .iter()
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .collect();
    if names.is_empty() {
        return;
    }

    let gradient_cache = state.gradient_cache.clone();
    let templates = state.templates.clone();
    let fontdb = state.fontdb.clone();

    tokio::task::spawn_blocking(move || {
        let total = names.len();
        tracing::info!("Gradient cache warm-up starting: {total} template(s)");
        let mut warmed = 0_usize;
        let mut skipped = 0_usize;
        for name in &names {
            let Some(split) = templates.gradient_splits.get(name) else {
                tracing::warn!("Warm-up: template '{name}' has no gradient split; skipping");
                skipped += 1;
                continue;
            };
            let key = generator::gradient_cache::build_key(
                name,
                &split.gradient_color_keys,
                &std::collections::HashMap::new(),
            );
            let result = gradient_cache.get_or_render(key, || {
                generator::render_to_pixmap(&split.gradient_svg, &fontdb, 1.0)
            });
            match result {
                Ok(_) => {
                    warmed += 1;
                }
                Err(e) => {
                    tracing::error!("Warm-up render failed for '{name}': {e}");
                    skipped += 1;
                }
            }
        }
        tracing::info!(
            "Gradient cache warm-up complete: {warmed}/{total} warmed, {skipped} skipped"
        );
        if let Some(metrics) = telemetry::get_metrics() {
            metrics.gradient_cache_warmup_completed.add(1, &[]);
        }
    });
}
