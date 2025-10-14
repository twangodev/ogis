use quick_xml::escape::escape;
use std::sync::Arc;

const DEFAULT_TEMPLATE: &str = include_str!("../templates/default.svg");

/// Generate an SVG with the given parameters by replacing ogis_ prefixed placeholders
pub fn generate_svg(title: &str, description: &str, width: u32, height: u32) -> String {
    DEFAULT_TEMPLATE
        .replace("ogis_title", &escape(title))
        .replace("ogis_description", &escape(description))
        .replace("ogis_width", &width.to_string())
        .replace("ogis_height", &height.to_string())
}

/// Render SVG to PNG using resvg
pub fn render_to_png(
    svg_data: &str,
    width: u32,
    height: u32,
    fontdb: &Arc<usvg::fontdb::Database>,
) -> Result<Vec<u8>, String> {
    // Parse SVG with shared font database
    let options = usvg::Options {
        fontdb: Arc::clone(fontdb),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(svg_data, &options)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Create pixmap
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    // Render
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Encode to PNG
    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {}", e))
}
