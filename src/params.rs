use serde::Deserialize;
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

use crate::AppState;
use crate::config::ImageFallbackBehavior;
use crate::error::ApiError;
use crate::generator::{OutputFormat, RenderOptions};
use crate::image::ValidatedImage;

/// Validate that a string is exactly 6 hexadecimal characters
#[inline]
fn is_valid_hex_color(s: &str) -> bool {
    s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit())
}

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
    /// Output format: png (default), webp, jpeg
    #[serde(default)]
    #[schema(example = "webp")]
    pub format: Option<String>,
    /// Scale factor for output resolution (0.1-1.0, default: 1.0)
    #[serde(default)]
    #[schema(example = 0.5)]
    pub scale: Option<f32>,
    /// Quality for lossy formats (1-100, default: 90, ignored for PNG)
    #[serde(default)]
    #[schema(example = 90)]
    pub quality: Option<u8>,
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

    /// Validate input parameters against maximum length and format constraints
    pub fn validate(&self, max_length: usize) -> Result<(), ApiError> {
        let fields = [
            ("Title", &self.title),
            ("Description", &self.description),
            ("Subtitle", &self.subtitle),
            ("Logo URL", &self.logo),
            ("Image URL", &self.image),
        ];

        for (name, field) in fields {
            if let Some(value) = field
                && value.len() > max_length
            {
                return Err(ApiError::validation_field_too_long(name, max_length));
            }
        }

        // Validate format if provided
        if let Some(format) = &self.format
            && OutputFormat::from_str(format).is_none()
        {
            return Err(ApiError::validation_invalid_format(format));
        }

        // Validate scale if provided (0.1 to 1.0)
        if let Some(scale) = self.scale
            && !(0.1..=1.0).contains(&scale)
        {
            return Err(ApiError::validation_invalid_scale(scale));
        }

        // Validate quality if provided (1 to 100)
        if let Some(quality) = self.quality
            && !(1..=100).contains(&quality)
        {
            return Err(ApiError::validation_invalid_quality(quality));
        }

        Ok(())
    }

    /// Get render options from parameters (with defaults)
    pub fn render_options(&self) -> RenderOptions {
        RenderOptions {
            format: self
                .format
                .as_deref()
                .and_then(OutputFormat::from_str)
                .unwrap_or_default(),
            scale: self.scale.unwrap_or(1.0),
            quality: self.quality.unwrap_or(90),
        }
    }

    /// Extract and validate color overrides from extra parameters
    pub fn extract_colors(
        &self,
        template_name: &str,
        state: &AppState,
    ) -> Result<HashMap<String, String>, ApiError> {
        let mut color_overrides = HashMap::new();

        let template_colors = state.templates.colors.get(template_name);

        for (key, value) in &self.extra {
            if let Some(template_colors_map) = template_colors
                && template_colors_map.contains_key(key)
            {
                if !is_valid_hex_color(value) {
                    return Err(ApiError::validation_invalid_hex_color(key, value));
                }

                color_overrides.insert(key.to_string(), format!("#{}", value.to_lowercase()));
            }
        }

        Ok(color_overrides)
    }

    /// Fetch logo image if URL provided, respecting fallback behavior
    pub async fn fetch_logo(&self, state: &AppState) -> Result<Option<ValidatedImage>, ApiError> {
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
    pub async fn fetch_image(&self, state: &AppState) -> Result<Option<ValidatedImage>, ApiError> {
        self.fetch_image_from_url(&self.image, "image", state).await
    }

    /// Helper to fetch an image from a URL with error handling
    async fn fetch_image_from_url(
        &self,
        url_option: &Option<String>,
        name: &str,
        state: &AppState,
    ) -> Result<Option<ValidatedImage>, ApiError> {
        if let Some(url) = url_option {
            match state.image.fetcher.fetch_image(url).await {
                Ok(validated) => Ok(Some(validated)),
                Err(e) => match state.image.fallback {
                    ImageFallbackBehavior::Skip => Ok(None),
                    ImageFallbackBehavior::Error => {
                        let api_error: ApiError = e.into();
                        Err(api_error.with_field(name))
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn default_params() -> OgParams {
        OgParams {
            title: None,
            description: None,
            subtitle: None,
            logo: None,
            image: None,
            template: None,
            signature: None,
            format: None,
            scale: None,
            quality: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_validate_invalid_format() {
        let mut params = default_params();
        params.format = Some("gif".to_string());
        assert!(params.validate(1000).is_err());
    }

    #[test]
    fn test_validate_scale_out_of_range() {
        let mut params = default_params();
        params.scale = Some(0.05);
        assert!(params.validate(1000).is_err());

        params.scale = Some(1.5);
        assert!(params.validate(1000).is_err());
    }

    #[test]
    fn test_validate_quality_out_of_range() {
        let mut params = default_params();
        params.quality = Some(0);
        assert!(params.validate(1000).is_err());

        params.quality = Some(101);
        assert!(params.validate(1000).is_err());
    }

    #[test]
    fn test_render_options_defaults() {
        let params = default_params();
        let opts = params.render_options();
        assert_eq!(opts.format, OutputFormat::Png);
        assert_eq!(opts.scale, 1.0);
        assert_eq!(opts.quality, 90);
    }

    #[test]
    fn test_render_options_custom() {
        let mut params = default_params();
        params.format = Some("webp".to_string());
        params.scale = Some(0.5);
        params.quality = Some(80);

        let opts = params.render_options();
        assert_eq!(opts.format, OutputFormat::WebP);
        assert_eq!(opts.scale, 0.5);
        assert_eq!(opts.quality, 80);
    }
}
