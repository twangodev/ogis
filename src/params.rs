use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

use crate::AppState;
use crate::config::ImageFallbackBehavior;
use crate::image::ValidatedImage;

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
    /// Optional custom image URL
    #[serde(default)]
    pub image: Option<String>,
    /// Template name (e.g., "twilight", "daybreak")
    #[serde(default)]
    pub template: Option<String>,
    /// HMAC signature (required when authentication is enabled)
    #[serde(default)]
    #[schema(example = "a1b2c3d4e5f6...")]
    #[allow(dead_code)] // Used by middleware, not by route handler
    pub signature: Option<String>,
    /// Additional parameters (used for color customization)
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl OgParams {
    /// Check if all parameters are None (no params provided)
    fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.subtitle.is_none()
            && self.logo.is_none()
            && self.image.is_none()
    }

    /// Validate input parameters against maximum length
    pub fn validate(&self, max_length: usize) -> Result<(), String> {
        let fields = [
            ("Title", &self.title),
            ("Description", &self.description),
            ("Subtitle", &self.subtitle),
            ("Logo URL", &self.logo),
            ("Image URL", &self.image),
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

    /// Extract and validate color overrides from extra parameters
    pub fn extract_colors(
        &self,
        template_name: &str,
        state: &AppState,
    ) -> Result<HashMap<String, String>, String> {
        let mut color_overrides = HashMap::new();

        let template_colors = state.templates.colors.get(template_name);
        let hex_regex = Regex::new(r"^[0-9A-Fa-f]{6}$").unwrap();

        for (key, value) in &self.extra {
            if let Some(template_colors_map) = template_colors {
                if template_colors_map.contains_key(key) {
                    if !hex_regex.is_match(value) {
                        return Err(format!(
                            "Invalid hex color '{}' for parameter '{}'. Expected 6 hex characters (e.g., 'FF0000')",
                            value, key
                        ));
                    }

                    color_overrides.insert(key.clone(), format!("#{}", value));
                }
            }
        }

        Ok(color_overrides)
    }

    /// Fetch logo image if URL provided, respecting fallback behavior
    pub async fn fetch_logo(&self, state: &AppState) -> Result<Option<ValidatedImage>, Response> {
        let logo_url = self.get_effective_logo(state);
        self.fetch_image_from_url(&logo_url, "logo", state).await
    }

    /// Get the effective logo URL (using default if no params provided)
    fn get_effective_logo(&self, state: &AppState) -> Option<String> {
        if self.is_empty() {
            Some(state.defaults.logo.clone())
        } else {
            self.logo.clone()
        }
    }

    /// Fetch custom image if URL provided, respecting fallback behavior
    pub async fn fetch_image(&self, state: &AppState) -> Result<Option<ValidatedImage>, Response> {
        self.fetch_image_from_url(&self.image, "image", state).await
    }

    /// Helper to fetch an image from a URL with error handling
    async fn fetch_image_from_url(
        &self,
        url_option: &Option<String>,
        name: &str,
        state: &AppState,
    ) -> Result<Option<ValidatedImage>, Response> {
        if let Some(url) = url_option {
            match state.image.fetcher.fetch_image(url).await {
                Ok(validated) => Ok(Some(validated)),
                Err(e) => match state.image.fallback {
                    ImageFallbackBehavior::Skip => Ok(None),
                    ImageFallbackBehavior::Error => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to fetch {}: {}", name, e),
                    )
                        .into_response()),
                },
            }
        } else {
            Ok(None)
        }
    }

    /// Apply defaults for missing parameters
    pub fn with_defaults(&self, state: &AppState) -> (String, String, String) {
        let no_params = self.is_empty();

        let get = |param: &Option<String>, default: &str| {
            if no_params {
                default.to_string()
            } else {
                param.clone().unwrap_or_default()
            }
        };

        (
            get(&self.title, &state.defaults.title),
            get(&self.description, &state.defaults.description),
            get(&self.subtitle, &state.defaults.subtitle),
        )
    }
}
