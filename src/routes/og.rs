use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OgParams {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_description")]
    pub description: String,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_title() -> String {
    "Open Graph Image".to_string()
}

fn default_description() -> String {
    "Generated with OGIS".to_string()
}

fn default_width() -> u32 {
    1200
}

fn default_height() -> u32 {
    630
}

pub async fn handler(Query(params): Query<OgParams>) -> impl IntoResponse {
    tracing::info!(
        "Generating OG image: {}x{}, title: {}",
        params.width,
        params.height,
        params.title
    );

    // Generate SVG
    let svg_data = generate_svg(&params.title, &params.description, params.width, params.height);

    // Render SVG to PNG using resvg
    match render_svg_to_png(&svg_data, params.width, params.height) {
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

fn generate_svg(title: &str, description: &str, width: u32, height: u32) -> String {
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

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn render_svg_to_png(svg_data: &str, width: u32, height: u32) -> Result<Vec<u8>, String> {
    // Parse SVG
    let options = usvg::Options::default();
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
