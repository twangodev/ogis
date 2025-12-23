use std::io::Cursor;
use std::sync::Arc;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};

use super::error::GeneratorError;

/// Output image format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Png,
    Jpeg,
    WebP,
}

impl OutputFormat {
    /// Parse format from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpeg" | "jpg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            _ => None,
        }
    }

    /// Get the content-type for this format
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
        }
    }

    /// Get format name for metrics/logging
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::WebP => "webp",
        }
    }
}

/// Options for rendering SVG to image
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Output format (default: PNG)
    pub format: OutputFormat,
    /// Scale factor 0.1-1.0 (default: 1.0)
    pub scale: f32,
    /// Quality 1-100 for lossy formats (default: 90, ignored for PNG)
    pub quality: u8,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Png,
            scale: 1.0,
            quality: 90,
        }
    }
}

/// Result of rendering an image
pub struct RenderOutput {
    /// Encoded image bytes
    pub bytes: Vec<u8>,
    /// Content-Type header value
    pub content_type: &'static str,
}

/// Render SVG to image with given options
pub fn render(
    svg_data: &str,
    fontdb: &Arc<usvg::fontdb::Database>,
    options: &RenderOptions,
) -> Result<RenderOutput, GeneratorError> {
    let usvg_options = usvg::Options {
        fontdb: Arc::clone(fontdb),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(svg_data, &usvg_options)
        .map_err(|e| GeneratorError::SvgParse(e.to_string()))?;

    let size = tree.size();

    // Calculate scaled dimensions
    let scaled_width = (size.width() * options.scale).round() as u32;
    let scaled_height = (size.height() * options.scale).round() as u32;

    // Create pixmap at scaled size
    let mut pixmap = tiny_skia::Pixmap::new(scaled_width, scaled_height)
        .ok_or(GeneratorError::PixmapCreation)?;

    // For JPEG (no alpha), fill with white background
    if options.format == OutputFormat::Jpeg {
        pixmap.fill(tiny_skia::Color::WHITE);
    }

    // Apply scale transform when rendering
    let transform = tiny_skia::Transform::from_scale(options.scale, options.scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Encode to requested format
    let bytes = encode_pixmap(&pixmap, options)?;

    Ok(RenderOutput {
        bytes,
        content_type: options.format.content_type(),
    })
}

/// Encode pixmap to the specified format
fn encode_pixmap(
    pixmap: &tiny_skia::Pixmap,
    options: &RenderOptions,
) -> Result<Vec<u8>, GeneratorError> {
    let width = pixmap.width();
    let height = pixmap.height();

    // Convert RGBA (premultiplied) to standard RGBA for image crate
    let rgba_data = demultiply_alpha(pixmap.data());

    match options.format {
        OutputFormat::Png => encode_png(&rgba_data, width, height),
        OutputFormat::Jpeg => encode_jpeg(&rgba_data, width, height, options.quality),
        OutputFormat::WebP => encode_webp(&rgba_data, width, height, options.quality),
    }
}

/// Convert premultiplied alpha to straight alpha
fn demultiply_alpha(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());

    for chunk in data.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];

        if a == 0 {
            result.extend_from_slice(&[0, 0, 0, 0]);
        } else if a == 255 {
            result.extend_from_slice(&[r, g, b, a]);
        } else {
            // Demultiply: color = premultiplied_color * 255 / alpha
            let alpha_f = a as f32;
            result.push((r as f32 * 255.0 / alpha_f).round() as u8);
            result.push((g as f32 * 255.0 / alpha_f).round() as u8);
            result.push((b as f32 * 255.0 / alpha_f).round() as u8);
            result.push(a);
        }
    }

    result
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, GeneratorError> {
    let mut buffer = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut buffer);

    encoder
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .map_err(|e| GeneratorError::ImageEncode(format!("PNG: {}", e)))?;

    Ok(buffer.into_inner())
}

fn encode_jpeg(
    rgba: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, GeneratorError> {
    // Convert RGBA to RGB (drop alpha channel since JPEG doesn't support it)
    let rgb_data: Vec<u8> = rgba
        .chunks_exact(4)
        .flat_map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();

    let mut buffer = Cursor::new(Vec::new());
    let encoder = JpegEncoder::new_with_quality(&mut buffer, quality);

    encoder
        .write_image(&rgb_data, width, height, ExtendedColorType::Rgb8)
        .map_err(|e| GeneratorError::ImageEncode(format!("JPEG: {}", e)))?;

    Ok(buffer.into_inner())
}

fn encode_webp(
    rgba: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, GeneratorError> {
    let encoder = webp::Encoder::from_rgba(rgba, width, height);
    let lossless = quality >= 100;
    let memory = encoder
        .encode_simple(lossless, quality as f32)
        .map_err(|e| GeneratorError::ImageEncode(format!("WebP: {:?}", e)))?;

    Ok(memory.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("png"), Some(OutputFormat::Png));
        assert_eq!(OutputFormat::from_str("PNG"), Some(OutputFormat::Png));
        assert_eq!(OutputFormat::from_str("jpeg"), Some(OutputFormat::Jpeg));
        assert_eq!(OutputFormat::from_str("jpg"), Some(OutputFormat::Jpeg));
        assert_eq!(OutputFormat::from_str("webp"), Some(OutputFormat::WebP));
        assert_eq!(OutputFormat::from_str("WEBP"), Some(OutputFormat::WebP));
        assert_eq!(OutputFormat::from_str("gif"), None);
        assert_eq!(OutputFormat::from_str(""), None);
    }

    #[test]
    fn test_output_format_content_type() {
        assert_eq!(OutputFormat::Png.content_type(), "image/png");
        assert_eq!(OutputFormat::Jpeg.content_type(), "image/jpeg");
        assert_eq!(OutputFormat::WebP.content_type(), "image/webp");
    }

    #[test]
    fn test_output_format_as_str() {
        assert_eq!(OutputFormat::Png.as_str(), "png");
        assert_eq!(OutputFormat::Jpeg.as_str(), "jpeg");
        assert_eq!(OutputFormat::WebP.as_str(), "webp");
    }

    #[test]
    fn test_render_options_default() {
        let opts = RenderOptions::default();
        assert_eq!(opts.format, OutputFormat::Png);
        assert_eq!(opts.scale, 1.0);
        assert_eq!(opts.quality, 90);
    }

    #[test]
    fn test_demultiply_alpha_opaque() {
        let input = [100, 150, 200, 255];
        let result = demultiply_alpha(&input);
        assert_eq!(result, vec![100, 150, 200, 255]);
    }

    #[test]
    fn test_demultiply_alpha_transparent() {
        let input = [0, 0, 0, 0];
        let result = demultiply_alpha(&input);
        assert_eq!(result, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_render_simple_svg() {
        let svg = r#"<svg width="10" height="10" xmlns="http://www.w3.org/2000/svg"><rect width="10" height="10" fill="red"/></svg>"#;
        let fontdb = Arc::new(usvg::fontdb::Database::new());
        let result = render(svg, &fontdb, &RenderOptions::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content_type, "image/png");
    }

    #[test]
    fn test_render_invalid_svg() {
        let fontdb = Arc::new(usvg::fontdb::Database::new());
        let result = render("not valid svg", &fontdb, &RenderOptions::default());
        assert!(result.is_err());
    }
}
