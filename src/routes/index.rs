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
    #[serde(default)]
    pub title: Option<String>,
    /// Description text for the image
    #[serde(default)]
    pub description: Option<String>,
    /// Subtitle text (above title)
    #[serde(default)]
    pub subtitle: Option<String>,
    /// Optional logo image URL
    #[serde(default)]
    pub logo: Option<String>,
}

impl OgParams {
    /// Validate input parameters against maximum length
    fn validate(&self, max_length: usize) -> Result<(), String> {
        if let Some(ref title) = self.title {
            if title.len() > max_length {
                return Err(format!("Title exceeds maximum length of {}", max_length));
            }
        }
        if let Some(ref description) = self.description {
            if description.len() > max_length {
                return Err(format!(
                    "Description exceeds maximum length of {}",
                    max_length
                ));
            }
        }
        if let Some(ref subtitle) = self.subtitle {
            if subtitle.len() > max_length {
                return Err(format!("Subtitle exceeds maximum length of {}", max_length));
            }
        }
        if let Some(ref logo_url) = self.logo {
            if logo_url.len() > max_length {
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
    let logo_image_base64 = if let Some(ref url) = params.logo {
        match state.image_fetcher.fetch_image(url).await {
            Ok(base64) => {
                tracing::info!("Successfully fetched logo image from: {}", url);
                Some(base64)
            }
            Err(e) => {
                match state.image_fallback {
                    crate::config::ImageFallbackBehavior::Skip => {
                        tracing::warn!(
                            "Failed to fetch logo from {}: {} - skipping logo element",
                            url,
                            e
                        );
                        None // Skip logo element
                    }
                    crate::config::ImageFallbackBehavior::Error => {
                        tracing::error!("Failed to fetch logo from {}: {}", url, e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to fetch logo: {}", e),
                        )
                            .into_response();
                    }
                }
            }
        }
    } else {
        None
    };

    // Check if NO params were provided at all
    let no_params_provided = params.title.is_none()
        && params.description.is_none()
        && params.subtitle.is_none()
        && params.logo.is_none();

    // Apply defaults only when NO params provided, otherwise use blank
    let title = if no_params_provided {
        &state.default_title
    } else {
        params.title.as_deref().unwrap_or("")
    };

    let description = if no_params_provided {
        &state.default_description
    } else {
        params.description.as_deref().unwrap_or("")
    };

    let subtitle = if no_params_provided {
        &state.default_subtitle
    } else {
        params.subtitle.as_deref().unwrap_or("")
    };

    // Generate SVG
    let svg_data = match generator::generate_svg(
        title,
        description,
        subtitle,
        logo_image_base64.as_deref(),
    ) {
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
