use std::sync::Arc;

/// Generate an SVG with the given parameters
pub fn generate_svg(title: &str, description: &str, width: u32, height: u32) -> String {
    format!(
        r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <rect width="100%" height="100%" fill="#1a1a2e"/>
  <style>
    .title {{ font: bold 72px sans-serif; fill: #ffffff; }}
    .description {{ font: 36px sans-serif; fill: #cccccc; }}
  </style>
  <text x="50%" y="40%" text-anchor="middle" class="title">{}</text>
  <text x="50%" y="55%" text-anchor="middle" class="description">{}</text>
</svg>"##,
        width,
        height,
        escape_xml(title),
        escape_xml(description)
    )
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
