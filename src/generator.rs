use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::io::Cursor;
use std::sync::Arc;

const DEFAULT_TEMPLATE: &str = include_str!("../templates/twilight.svg");

pub fn generate_svg(
    title: &str,
    description: &str,
    logo: &str,
    subtitle: &str,
    _logo_image_base64: Option<&str>,
) -> Result<String, String> {
    let mut reader = Reader::from_str(DEFAULT_TEMPLATE);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer
                    .write_event(e)
                    .map_err(|e| format!("Write error: {}", e))?;
            }
            Err(e) => return Err(format!("Parse error: {:?}", e)),
        }
        buf.clear();
    }

    String::from_utf8(writer.into_inner().into_inner())
        .map_err(|e| format!("UTF-8 error: {}", e))
}

pub fn render_to_png(
    svg_data: &str,
    fontdb: &Arc<usvg::fontdb::Database>,
) -> Result<Vec<u8>, String> {
    let options = usvg::Options {
        fontdb: Arc::clone(fontdb),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(svg_data, &options)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    let size = tree.size();
    let width = size.width().round() as u32;
    let height = size.height().round() as u32;

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {}", e))
}