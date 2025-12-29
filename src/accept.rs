//! Accept header content negotiation for image formats.

use headers_accept::Accept;
use std::str::FromStr;
use std::sync::LazyLock;

use crate::generator::OutputFormat;

/// All supported output formats for negotiation.
const SUPPORTED_FORMATS: [OutputFormat; 3] =
    [OutputFormat::WebP, OutputFormat::Png, OutputFormat::Jpeg];

/// Parsed MIME types for negotiation, initialized once.
static AVAILABLE_MIMES: LazyLock<[mediatype::MediaType; 3]> = LazyLock::new(|| {
    SUPPORTED_FORMATS.map(|f| mediatype::MediaType::parse(f.content_type()).unwrap())
});

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

    let best = accept.negotiate(&*AVAILABLE_MIMES)?;
    let best_mime = best.essence().to_string();

    SUPPORTED_FORMATS
        .iter()
        .find(|f| f.content_type() == best_mime)
        .copied()
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
