use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::AppState;
use crate::config::ImageFallbackBehavior;

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
    pub fn validate(&self, max_length: usize) -> Result<(), String> {
        let fields = [
            ("Title", &self.title),
            ("Description", &self.description),
            ("Subtitle", &self.subtitle),
            ("Logo URL", &self.logo),
        ];

        for (name, field) in fields {
            if let Some(value) = field {
                if value.len() > max_length {
                    return Err(format!("{} exceeds maximum length of {}", name, max_length));
                }
            }
        }

        Ok(())
    }

    /// Fetch logo image if URL provided, respecting fallback behavior
    pub async fn fetch_logo(&self, state: &AppState) -> Result<Option<String>, Response> {
        if let Some(ref url) = self.logo {
            match state.image_fetcher.fetch_image(url).await {
                Ok(base64) => {
                    tracing::info!("Successfully fetched logo image from: {}", url);
                    Ok(Some(base64))
                }
                Err(e) => match state.image_fallback {
                    ImageFallbackBehavior::Skip => {
                        tracing::warn!(
                            "Failed to fetch logo from {}: {} - skipping logo element",
                            url,
                            e
                        );
                        Ok(None)
                    }
                    ImageFallbackBehavior::Error => {
                        tracing::error!("Failed to fetch logo from {}: {}", url, e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to fetch logo: {}", e),
                        )
                            .into_response())
                    }
                },
            }
        } else {
            Ok(None)
        }
    }

    /// Apply defaults for missing parameters
    pub fn with_defaults(&self, state: &AppState) -> (String, String, String) {
        let no_params = self.title.is_none()
            && self.description.is_none()
            && self.subtitle.is_none()
            && self.logo.is_none();

        let get = |param: &Option<String>, default: &str| {
            if no_params {
                default.to_string()
            } else {
                param.clone().unwrap_or_default()
            }
        };

        (
            get(&self.title, &state.default_title),
            get(&self.description, &state.default_description),
            get(&self.subtitle, &state.default_subtitle),
        )
    }
}
