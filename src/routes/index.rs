use super::timing::{CacheableDuration, ServerTiming};
use crate::{
    AppState,
    generator::{self, Images, TextContent},
    params::OgParams,
};
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use std::time::{Duration, Instant};

/// Result from the blocking render task
struct RenderResult {
    png_data: Vec<u8>,
    template_time: Duration,
    render_time: Duration,
}

/// Metadata collector for single-log-per-request pattern
struct RequestLog {
    start: Instant,
    status: StatusCode,
    template: Option<String>,
    logo_domain: Option<String>,
    logo_cached: bool,
    image_domain: Option<String>,
    image_cached: bool,
    title_len: usize,
    desc_len: usize,
    subtitle_len: usize,
    error: Option<String>,
}

impl RequestLog {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            status: StatusCode::OK,
            template: None,
            logo_domain: None,
            logo_cached: false,
            image_domain: None,
            image_cached: false,
            title_len: 0,
            desc_len: 0,
            subtitle_len: 0,
            error: None,
        }
    }

    fn set_params(&mut self, params: &OgParams) {
        self.title_len = params.title.as_ref().map(|s| s.len()).unwrap_or(0);
        self.desc_len = params.description.as_ref().map(|s| s.len()).unwrap_or(0);
        self.subtitle_len = params.subtitle.as_ref().map(|s| s.len()).unwrap_or(0);
        self.template = params.template.clone();

        // Extract domains from URLs
        if let Some(url) = &params.logo {
            self.logo_domain = extract_domain(url);
        }
        if let Some(url) = &params.image {
            self.image_domain = extract_domain(url);
        }
    }

    fn log(self) {
        let duration_ms = self.start.elapsed().as_millis();

        tracing::info!(
            status = %self.status.as_u16(),
            duration_ms = duration_ms,
            template = self.template.as_deref().unwrap_or("default"),
            logo = self.logo_domain.as_deref().unwrap_or("-"),
            logo_cached = self.logo_cached,
            image = self.image_domain.as_deref().unwrap_or("-"),
            image_cached = self.image_cached,
            title_len = self.title_len,
            desc_len = self.desc_len,
            subtitle_len = self.subtitle_len,
            error = self.error.as_deref().unwrap_or("-"),
            "request completed"
        );
    }
}

/// Extract domain from URL (no path or query params)
fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
}

#[utoipa::path(
    get,
    path = "/",
    params(OgParams),
    responses(
        (status = 200, description = "Successfully generated PNG image (1200x630)", content_type = "image/png"),
        (status = 400, description = "Invalid input - field exceeds maximum length or invalid signature format"),
        (status = 401, description = "Authentication required or invalid signature"),
        (status = 500, description = "Failed to generate image")
    ),
    tag = "image"
)]
pub async fn generate(
    State(state): State<AppState>,
    Query(params): Query<OgParams>,
) -> impl IntoResponse {
    let mut log = RequestLog::new();
    let mut timing = ServerTiming::new();
    log.set_params(&params);

    // Validate input lengths
    if let Err(err) = params.validate(state.max_input_length) {
        log.status = StatusCode::BAD_REQUEST;
        log.error = Some(format!("Validation: {}", err));
        log.log();
        return (StatusCode::BAD_REQUEST, format!("Invalid input: {}", err)).into_response();
    }

    // Fetch logo image if URL provided
    let has_logo_url = params.logo.is_some();
    let logo_start = Instant::now();
    let logo = match params.fetch_logo(&state).await {
        Ok(img) => {
            if let Some(ref validated) = img {
                log.logo_cached = validated.cached;
                timing.logo = Some(CacheableDuration::new(
                    logo_start.elapsed(),
                    validated.cached,
                ));
            } else if has_logo_url {
                timing.logo = Some(CacheableDuration::miss_since(logo_start));
            }
            img
        }
        Err(response) => {
            log.status = StatusCode::INTERNAL_SERVER_ERROR;
            log.error = Some("Logo fetch failed".to_string());
            log.log();
            return response;
        }
    };

    // Fetch custom image if URL provided
    let has_image_url = params.image.is_some();
    let image_start = Instant::now();
    let image = match params.fetch_image(&state).await {
        Ok(img) => {
            if let Some(ref validated) = img {
                log.image_cached = validated.cached;
                timing.image = Some(CacheableDuration::new(
                    image_start.elapsed(),
                    validated.cached,
                ));
            } else if has_image_url {
                // Image URL was provided but fetch failed (Skip mode)
                timing.image = Some(CacheableDuration::miss_since(image_start));
            }
            img
        }
        Err(response) => {
            log.status = StatusCode::INTERNAL_SERVER_ERROR;
            log.error = Some("Image fetch failed".to_string());
            log.log();
            return response;
        }
    };

    // Apply defaults for missing params
    let (title, description, subtitle) = params.with_defaults(&state);

    // Determine template name for color validation
    let template_name = params
        .template
        .as_deref()
        .unwrap_or(&state.templates.default);

    let color_overrides = match params.extract_colors(template_name, &state) {
        Ok(colors) => colors,
        Err(err) => {
            log.status = StatusCode::BAD_REQUEST;
            log.error = Some(format!("Color validation: {}", err));
            log.log();
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid color parameter: {}", err),
            )
                .into_response();
        }
    };

    // Wait for a render slot with timeout (defers response, only 503 if truly overloaded)
    const RENDER_TIMEOUT: Duration = Duration::from_secs(5);
    let queue_start = Instant::now();
    let _render_permit =
        match tokio::time::timeout(RENDER_TIMEOUT, state.render_semaphore.acquire()).await {
            Ok(Ok(permit)) => {
                timing.queue = Some(queue_start.elapsed());
                permit
            }
            Ok(Err(_)) => {
                // Semaphore closed (shouldn't happen)
                log.status = StatusCode::INTERNAL_SERVER_ERROR;
                log.error = Some("Render semaphore closed".to_string());
                log.log();
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service unavailable".to_string(),
                )
                    .into_response();
            }
            Err(_) => {
                log.status = StatusCode::SERVICE_UNAVAILABLE;
                log.error = Some("Render timeout".to_string());
                log.log();
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Server overloaded, try again later".to_string(),
                )
                    .into_response();
            }
        };

    let images = Images { logo, image };
    let template_name_owned = template_name.to_string();
    let templates = state.templates.clone();
    let fontdb = state.fontdb.clone();

    let render_result = tokio::task::spawn_blocking(move || {
        let text = TextContent {
            title: &title,
            description: &description,
            subtitle: &subtitle,
        };

        let template_start = Instant::now();
        let svg_data = generator::generate_svg(
            text,
            images,
            &template_name_owned,
            &templates,
            &color_overrides,
            &fontdb,
        )?;
        let template_time = template_start.elapsed();

        let render_start = Instant::now();
        let png_data = generator::render_to_png(&svg_data, &fontdb)?;
        let render_time = render_start.elapsed();

        Ok::<_, String>(RenderResult {
            png_data,
            template_time,
            render_time,
        })
    })
    .await;

    match render_result {
        Ok(Ok(result)) => {
            timing.template = Some(result.template_time);
            timing.render = Some(result.render_time);
            log.log();
            (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        header::HeaderValue::from_static("image/png"),
                    ),
                    (
                        header::HeaderName::from_static("server-timing"),
                        timing.to_header_value(),
                    ),
                ],
                result.png_data,
            )
                .into_response()
        }
        Ok(Err(err)) => {
            log.status = StatusCode::INTERNAL_SERVER_ERROR;
            log.error = Some(format!("Render: {}", err));
            log.log();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render image: {}", err),
            )
                .into_response()
        }
        Err(join_err) => {
            log.status = StatusCode::INTERNAL_SERVER_ERROR;
            log.error = Some(format!("Task join: {}", join_err));
            log.log();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Rendering task failed".to_string(),
            )
                .into_response()
        }
    }
}
