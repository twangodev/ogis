use super::timing::{CacheableDuration, ServerTiming};
use crate::{
    AppState,
    error::ApiError,
    generator::{self, GeneratorError, Images, RenderOutput, TextContent},
    params::OgParams,
    telemetry,
};
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use opentelemetry::KeyValue;
use std::time::{Duration, Instant};

/// Result from the blocking render task
struct RenderResult {
    output: RenderOutput,
    template_time: Duration,
    render_time: Duration,
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
        (status = 400, description = "Invalid input - field exceeds maximum length or invalid hex color"),
        (status = 401, description = "Authentication required - missing signature"),
        (status = 403, description = "Forbidden - invalid signature or SSRF blocked"),
        (status = 404, description = "Template not found"),
        (status = 422, description = "Unprocessable - invalid image URL, unsupported format, or image too large"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "Bad gateway - upstream image fetch failed"),
        (status = 503, description = "Service unavailable - server overloaded"),
        (status = 504, description = "Gateway timeout - image fetch timed out")
    ),
    tag = "image"
)]
#[tracing::instrument(
    name = "generate_og_image",
    skip_all,
    fields(
        template,
        ogis.has_logo,
        ogis.has_image,
        ogis.logo_domain,
        ogis.image_domain,
        ogis.format,
        ogis.scale,
        http.response.status_code,
    )
)]
pub async fn generate(
    State(state): State<AppState>,
    Query(params): Query<OgParams>,
) -> impl IntoResponse {
    let span = tracing::Span::current();
    let mut timing = ServerTiming::new();

    // Record initial span attributes
    let template_name = params
        .template
        .as_deref()
        .unwrap_or(&state.templates.default);
    span.record("template", template_name);
    span.record("ogis.has_logo", params.logo.is_some());
    span.record("ogis.has_image", params.image.is_some());

    let is_params_empty = params.title.is_none()
        && params.description.is_none()
        && params.subtitle.is_none()
        && params.logo.is_none()
        && params.image.is_none();
    let effective_logo_url = if is_params_empty {
        Some(&state.defaults.logo)
    } else {
        params.logo.as_ref()
    };
    let logo_domain = effective_logo_url.and_then(|u| extract_domain(u));
    let image_domain = params.image.as_ref().and_then(|u| extract_domain(u));
    span.record("ogis.logo_domain", logo_domain.as_deref().unwrap_or("-"));
    span.record("ogis.image_domain", image_domain.as_deref().unwrap_or("-"));

    // Validate input lengths and format/scale/quality
    if let Err(err) = params.validate(state.max_input_length) {
        span.record(
            "http.response.status_code",
            err.status_code().as_u16() as i64,
        );
        tracing::warn!(error = %err, "Validation failed");
        return err.into_response();
    }

    // Get render options and record in span
    let render_options = params.render_options();
    let format_str = render_options.format.as_str();
    let scale = render_options.scale;
    span.record("ogis.format", format_str);
    span.record("ogis.scale", scale as f64);

    // Fetch logo image if URL provided
    let has_logo_url = params.logo.is_some();
    let logo_start = Instant::now();
    let logo = match params.fetch_logo(&state).await {
        Ok(img) => {
            if let Some(ref validated) = img {
                timing.logo = Some(CacheableDuration::new(
                    logo_start.elapsed(),
                    validated.cached,
                ));
                record_image_metrics(
                    "logo",
                    &logo_domain,
                    validated.cached,
                    validated.bytes.len(),
                );
            } else if has_logo_url {
                timing.logo = Some(CacheableDuration::miss_since(logo_start));
            }
            img
        }
        Err(err) => {
            span.record(
                "http.response.status_code",
                err.status_code().as_u16() as i64,
            );
            tracing::error!(error = %err, "Logo fetch failed");
            return err.into_response();
        }
    };

    // Fetch custom image if URL provided
    let has_image_url = params.image.is_some();
    let image_start = Instant::now();
    let image = match params.fetch_image(&state).await {
        Ok(img) => {
            if let Some(ref validated) = img {
                timing.image = Some(CacheableDuration::new(
                    image_start.elapsed(),
                    validated.cached,
                ));
                record_image_metrics(
                    "image",
                    &image_domain,
                    validated.cached,
                    validated.bytes.len(),
                );
            } else if has_image_url {
                timing.image = Some(CacheableDuration::miss_since(image_start));
            }
            img
        }
        Err(err) => {
            span.record(
                "http.response.status_code",
                err.status_code().as_u16() as i64,
            );
            tracing::error!(error = %err, "Image fetch failed");
            return err.into_response();
        }
    };

    // Apply defaults for missing params
    let (title, description, subtitle) = params.with_defaults(&state);

    let color_overrides = match params.extract_colors(template_name, &state) {
        Ok(colors) => colors,
        Err(err) => {
            span.record(
                "http.response.status_code",
                err.status_code().as_u16() as i64,
            );
            tracing::warn!(error = %err, "Color validation failed");
            return err.into_response();
        }
    };

    // Wait for a render slot with timeout
    const RENDER_TIMEOUT: Duration = Duration::from_secs(5);
    let queue_start = Instant::now();
    let _render_permit =
        match tokio::time::timeout(RENDER_TIMEOUT, state.render_semaphore.acquire()).await {
            Ok(Ok(permit)) => {
                let queue_wait = queue_start.elapsed();
                timing.queue = Some(queue_wait);
                // Record queue wait metric
                if let Some(m) = telemetry::get_metrics() {
                    m.render_queue_wait.record(queue_wait.as_secs_f64(), &[]);
                }
                permit
            }
            Ok(Err(_)) => {
                let err = ApiError::internal("Render semaphore closed");
                span.record("http.response.status_code", 500_i64);
                tracing::error!("Render semaphore closed");
                return err.into_response();
            }
            Err(_) => {
                let err = ApiError::service_overloaded();
                span.record("http.response.status_code", 503_i64);
                tracing::warn!("Render queue timeout");
                return err.into_response();
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
        let output = generator::render(&svg_data, &fontdb, &render_options)?;
        let render_time = render_start.elapsed();

        Ok::<_, GeneratorError>(RenderResult {
            output,
            template_time,
            render_time,
        })
    })
    .await;

    match render_result {
        Ok(Ok(result)) => {
            timing.template = Some(result.template_time);
            timing.render = Some(result.render_time);
            span.record("http.response.status_code", 200_i64);

            // Record render duration and output format metrics
            if let Some(m) = telemetry::get_metrics() {
                m.render_duration.record(
                    (result.template_time + result.render_time).as_secs_f64(),
                    &[
                        KeyValue::new("template", template_name.to_string()),
                        KeyValue::new("format", format_str),
                    ],
                );
                m.output_format.add(
                    1,
                    &[
                        KeyValue::new("format", format_str),
                        KeyValue::new("scale_bucket", telemetry::scale_bucket(scale)),
                    ],
                );
            }

            (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        header::HeaderValue::from_static(result.output.content_type),
                    ),
                    (
                        header::HeaderName::from_static("server-timing"),
                        timing.to_header_value(),
                    ),
                ],
                result.output.bytes,
            )
                .into_response()
        }
        Ok(Err(gen_err)) => {
            let err: ApiError = gen_err.into();
            span.record(
                "http.response.status_code",
                err.status_code().as_u16() as i64,
            );
            tracing::error!(error = %err, "Render failed");
            err.into_response()
        }
        Err(join_err) => {
            let err = ApiError::internal("Rendering task failed");
            span.record("http.response.status_code", 500_i64);
            tracing::error!(error = %join_err, "Render task join failed");
            err.into_response()
        }
    }
}

/// Record image fetch metrics
fn record_image_metrics(image_type: &str, domain: &Option<String>, cached: bool, size: usize) {
    if let Some(m) = telemetry::get_metrics() {
        let domain_str = domain.as_deref().unwrap_or("unknown");
        let attrs = [
            KeyValue::new("type", image_type.to_string()),
            KeyValue::new("domain", domain_str.to_string()),
            KeyValue::new("cached", cached),
        ];
        m.image_fetch.add(1, &attrs);
        m.image_size.record(
            size as u64,
            &[
                KeyValue::new("type", image_type.to_string()),
                KeyValue::new("domain", domain_str.to_string()),
            ],
        );
    }
}
