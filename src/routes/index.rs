use crate::{AppState, generator};
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct OgParams {
    /// Title text for the image
    #[serde(default = "default_title")]
    pub title: String,
    /// Description text for the image
    #[serde(default = "default_description")]
    pub description: String,
    /// Logo text (top-left, used if logo_url is not provided)
    #[serde(default = "default_logo")]
    pub logo: String,
    /// Subtitle text (above title)
    #[serde(default = "default_subtitle")]
    pub subtitle: String,
    /// Optional logo image URL (overrides text logo if provided)
    #[serde(default)]
    pub logo_url: Option<String>,
}

fn default_title() -> String {
    "Open Graph Image".to_string()
}

fn default_description() -> String {
    "Generated with OGIS".to_string()
}

fn default_logo() -> String {
    "OGIS".to_string()
}

fn default_subtitle() -> String {
    String::new()
}

impl OgParams {
    /// Validate input parameters against maximum length
    fn validate(&self, max_length: usize) -> Result<(), String> {
        if self.title.len() > max_length {
            return Err(format!("Title exceeds maximum length of {}", max_length));
        }
        if self.description.len() > max_length {
            return Err(format!(
                "Description exceeds maximum length of {}",
                max_length
            ));
        }
        if self.logo.len() > max_length {
            return Err(format!("Logo exceeds maximum length of {}", max_length));
        }
        if self.subtitle.len() > max_length {
            return Err(format!("Subtitle exceeds maximum length of {}", max_length));
        }
        if let Some(ref url) = self.logo_url {
            if url.len() > max_length {
                return Err(format!("Logo URL exceeds maximum length of {}", max_length));
            }
        }
        Ok(())
    }
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
    // Validate input lengths
    if let Err(err) = params.validate(state.max_input_length) {
        tracing::warn!("Input validation failed: {}", err);
        return (StatusCode::BAD_REQUEST, format!("Invalid input: {}", err)).into_response();
    }

    tracing::info!("Generating OG image with params: {:?}", params);

    // Fetch logo image if URL provided
    let logo_image_base64 = if let Some(ref url) = params.logo_url {
        match state.image_fetcher.fetch_image(url).await {
            Ok(base64) => {
                tracing::info!("Successfully fetched logo image from: {}", url);
                Some(base64)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch logo from {}: {} - falling back to text logo",
                    url,
                    e
                );
                None // Fallback to text logo
            }
        }
    } else {
        None
    };

    // Generate SVG
    let svg_data = generator::generate_svg(
        &params.title,
        &params.description,
        &params.logo,
        &params.subtitle,
        logo_image_base64.as_deref(),
    );

    // Render SVG to PNG using resvg (dimensions from SVG)
    match generator::render_to_png(&svg_data, &state.fontdb) {
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
