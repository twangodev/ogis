use crate::{AppState, generator, params::OgParams};
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};

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
    // Validate input lengths
    if let Err(err) = params.validate(state.max_input_length) {
        tracing::warn!("Input validation failed: {}", err);
        return (StatusCode::BAD_REQUEST, format!("Invalid input: {}", err)).into_response();
    }

    tracing::info!("Generating OG image with params: {:?}", params);

    // Fetch logo image if URL provided
    let logo_bytes = match params.fetch_logo(&state).await {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };

    // Apply defaults for missing params
    let (title, description, subtitle) = params.with_defaults(&state);

    // Generate SVG
    let svg_data = match generator::generate_svg(&title, &description, &subtitle, logo_bytes) {
        Ok(data) => data,
        Err(err) => {
            tracing::error!("Failed to generate SVG: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate SVG: {}", err),
            )
                .into_response();
        }
    };

    // Render SVG to PNG
    match generator::render_to_png(&svg_data, &state.fontdb) {
        Ok(png_data) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/png")],
            png_data,
        )
            .into_response(),
        Err(err) => {
            tracing::error!("Failed to render PNG: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render PNG: {}", err),
            )
                .into_response()
        }
    }
}
