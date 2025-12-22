use std::sync::Arc;

use super::error::GeneratorError;

pub fn render_to_png(
    svg_data: &str,
    fontdb: &Arc<usvg::fontdb::Database>,
) -> Result<Vec<u8>, GeneratorError> {
    let options = usvg::Options {
        fontdb: Arc::clone(fontdb),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(svg_data, &options)
        .map_err(|e| GeneratorError::SvgParse(e.to_string()))?;

    let size = tree.size();
    let width = size.width().round() as u32;
    let height = size.height().round() as u32;

    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(GeneratorError::PixmapCreation)?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| GeneratorError::PngEncode(e.to_string()))
}
