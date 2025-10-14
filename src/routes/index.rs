use crate::{AppState, generator};
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct OgParams {
    /// Title text for the image
    #[serde(default = "default_title")]
    pub title: String,
    /// Description text for the image
    #[serde(default = "default_description")]
    pub description: String,
    /// Image width in pixels
    #[serde(default = "default_width")]
    pub width: u32,
    /// Image height in pixels
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_title() -> String {
    "Open Graph Image".to_string()
}

fn default_description() -> String {
    "Generated with OGIS".to_string()
}

fn default_width() -> u32 {
    1200
}

fn default_height() -> u32 {
    630
}

#[utoipa::path(
    get,
    path = "/",
    params(OgParams),
    responses(
        (status = 200, description = "Successfully generated PNG image", content_type = "image/png"),
        (status = 500, description = "Failed to generate image")
    )
)]
pub async fn handler(
    State(state): State<AppState>,
    Query(params): Query<OgParams>,
) -> impl IntoResponse {
    tracing::info!(
        "Generating OG image: {}x{}, title: {}",
        params.width,
        params.height,
        params.title
    );

    // Generate SVG
    let svg_data = generator::generate_svg(
        &params.title,
        &params.description,
        params.width,
        params.height,
    );

    // Render SVG to PNG using resvg
    match generator::render_to_png(&svg_data, params.width, params.height, &state.fontdb) {
        Ok(png_data) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/png")],
            png_data,
        )
            .into_response(),
        Err(err) => {
            tracing::error!("Failed to generate image: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate image: {}", err),
            )
                .into_response()
        }
    }
}
