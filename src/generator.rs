use quick_xml::escape::escape;
use std::sync::Arc;

const DEFAULT_TEMPLATE: &str = include_str!("../templates/twilight.svg");

/// Generate an SVG with the given parameters by replacing ogis_ prefixed placeholders
pub fn generate_svg(
    title: &str,
    description: &str,
    logo: &str,
    subtitle: &str,
    logo_image_base64: Option<&str>,
) -> String {
    // Create logo element - either image or text
    let logo_element = if let Some(base64) = logo_image_base64 {
        // Replace text element with image (positioned near the same spot)
        format!(
            r#"<image id="logo-image" x="80" y="60" width="120" height="40" href="data:image/png;base64,{}"/>"#,
            base64
        )
    } else {
        // Keep text element, just replace content
        format!(
            r##"<text id="logo-text" x="80" y="100" font-family="sans-serif" font-size="40" font-weight="bold" fill="#667eea">{}</text>"##,
            escape(logo)
        )
    };

    DEFAULT_TEMPLATE
        .replace("ogis_title", &escape(title).to_string())
        .replace("ogis_description", &escape(description).to_string())
        .replace(
            r##"<text id="logo-text" x="80" y="100" font-family="sans-serif" font-size="40" font-weight="bold" fill="#667eea">ogis_logo</text>"##,
            &logo_element
        )
        .replace("ogis_subtitle", &escape(subtitle).to_string())
}

/// Render SVG to PNG using resvg, automatically using the SVG's defined dimensions
pub fn render_to_png(
    svg_data: &str,
    fontdb: &Arc<usvg::fontdb::Database>,
) -> Result<Vec<u8>, String> {
    // Parse SVG with shared font database
    let options = usvg::Options {
        fontdb: Arc::clone(fontdb),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(svg_data, &options)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Get dimensions from the SVG itself
    let size = tree.size();
    let width = size.width().round() as u32;
    let height = size.height().round() as u32;

    // Create pixmap with SVG's dimensions
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    // Render
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Encode to PNG
    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {}", e))
}
