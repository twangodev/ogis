//! Accept header content negotiation for image formats.

use headers_accept::Accept;
use std::str::FromStr;

use crate::generator::OutputFormat;

/// Supported image media types for content negotiation.
const IMAGE_WEBP: &str = "image/webp";
const IMAGE_PNG: &str = "image/png";
const IMAGE_JPEG: &str = "image/jpeg";

/// Negotiate the best output format based on the Accept header.
///
/// Returns `None` if:
/// - No Accept header is provided
/// - The header cannot be parsed
/// - No supported format matches
///
/// Uses RFC 9110 quality value (q) precedence via headers-accept crate:
/// higher q values are preferred over lower ones.
pub fn negotiate_format(accept_header: Option<&str>) -> Option<OutputFormat> {
    let header = accept_header?;
    let accept = Accept::from_str(header).ok()?;

    // Available formats we support
    let available: Vec<mediatype::MediaType> = [IMAGE_WEBP, IMAGE_PNG, IMAGE_JPEG]
        .iter()
        .filter_map(|s| mediatype::MediaType::parse(s).ok())
        .collect();

    // Negotiate returns the best match based on client preferences (RFC 9110)
    let best = accept.negotiate(&available)?;

    match best.essence().to_string().as_str() {
        IMAGE_WEBP => Some(OutputFormat::WebP),
        IMAGE_PNG => Some(OutputFormat::Png),
        IMAGE_JPEG => Some(OutputFormat::Jpeg),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_accept_header() {
        assert_eq!(negotiate_format(None), None);
    }

    #[test]
    fn test_empty_accept_header() {
        assert_eq!(negotiate_format(Some("")), None);
    }

    #[test]
    fn test_webp_only() {
        assert_eq!(
            negotiate_format(Some("image/webp")),
            Some(OutputFormat::WebP)
        );
    }

    #[test]
    fn test_png_only() {
        assert_eq!(negotiate_format(Some("image/png")), Some(OutputFormat::Png));
    }

    #[test]
    fn test_jpeg_only() {
        assert_eq!(
            negotiate_format(Some("image/jpeg")),
            Some(OutputFormat::Jpeg)
        );
    }

    #[test]
    fn test_quality_preference_png_wins() {
        // PNG has higher q value than WebP
        let result = negotiate_format(Some("image/webp;q=0.8, image/png;q=0.9"));
        assert_eq!(result, Some(OutputFormat::Png));
    }

    #[test]
    fn test_quality_preference_webp_wins() {
        // WebP has implicit q=1.0, PNG has q=0.9
        let result = negotiate_format(Some("image/png;q=0.9, image/webp"));
        assert_eq!(result, Some(OutputFormat::WebP));
    }

    #[test]
    fn test_unsupported_format_returns_none() {
        assert_eq!(negotiate_format(Some("image/gif")), None);
        assert_eq!(negotiate_format(Some("text/html")), None);
    }

    #[test]
    fn test_browser_like_header() {
        // Chrome-like Accept header for images
        let header = "image/avif,image/webp,image/apng,image/svg+xml,image/*;q=0.8,*/*;q=0.5";
        let result = negotiate_format(Some(header));
        // Should return WebP since we don't support avif/apng/svg
        assert_eq!(result, Some(OutputFormat::WebP));
    }
}
