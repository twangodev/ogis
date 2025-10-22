use crate::{AppState, generator, params::OgParams};
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use std::time::Instant;

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
    url::Url::parse(url).ok().and_then(|u| u.host_str().map(String::from))
}

#[utoipa::path(
    get,
    path = "/",
    params(OgParams),
    responses(
        (status = 200, description = "Successfully generated PNG image (1200x630)", content_type = "image/png"),
        (status = 400, description = "Invalid input - field exceeds maximum length"),
        (status = 500, description = "Failed to generate image")
    ),
    tag = "image"
)]
pub async fn generate(
    State(state): State<AppState>,
    Query(params): Query<OgParams>,
) -> impl IntoResponse {
    let mut log = RequestLog::new();
    log.set_params(&params);

    // Validate input lengths
    if let Err(err) = params.validate(state.max_input_length) {
        log.status = StatusCode::BAD_REQUEST;
        log.error = Some(format!("Validation: {}", err));
        log.log();
        return (StatusCode::BAD_REQUEST, format!("Invalid input: {}", err)).into_response();
    }

    // Fetch logo image if URL provided
    let logo = match params.fetch_logo(&state).await {
        Ok(img) => {
            if let Some(ref validated) = img {
                log.logo_cached = validated.cached;
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
    let image = match params.fetch_image(&state).await {
        Ok(img) => {
            if let Some(ref validated) = img {
                log.image_cached = validated.cached;
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

    // Generate SVG
    let svg_data = match generator::generate_svg(
        &title,
        &description,
        &subtitle,
        logo,
        image,
        params.template.as_deref(),
        &state.templates,
    ) {
        Ok(data) => data,
        Err(err) => {
            log.status = StatusCode::INTERNAL_SERVER_ERROR;
            log.error = Some(format!("SVG generation: {}", err));
            log.log();
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate SVG: {}", err),
            )
                .into_response();
        }
    };

    // Render SVG to PNG
    match generator::render_to_png(&svg_data, &state.fontdb) {
        Ok(png_data) => {
            log.log();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/png")],
                png_data,
            )
                .into_response()
        }
        Err(err) => {
            log.status = StatusCode::INTERNAL_SERVER_ERROR;
            log.error = Some(format!("PNG render: {}", err));
            log.log();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render PNG: {}", err),
            )
                .into_response()
        }
    }
}
