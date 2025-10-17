use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::collections::HashMap;
use std::io::Cursor;

use super::events::{State, ImageReplacement, handle_default, handle_empty, handle_end, handle_start};
use crate::image::ValidatedImage;

const DEFAULT_TEMPLATE: &str = include_str!("../../templates/twilight.svg");

pub fn generate_svg(
    title: &str,
    description: &str,
    subtitle: &str,
    logo: Option<ValidatedImage>,
    image: Option<ValidatedImage>,
) -> Result<String, String> {
    let mut reader = Reader::from_str(DEFAULT_TEMPLATE);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Create text replacement map: element ID -> replacement text
    let text_replacements = HashMap::from([
        ("ogis_title".to_string(), title.to_string()),
        ("ogis_description".to_string(), description.to_string()),
        ("ogis_subtitle".to_string(), subtitle.to_string()),
    ]);

    // Convert ValidatedImage to ImageReplacement
    let logo_replacement = logo.map(|v| ImageReplacement {
        bytes: v.bytes,
        mime_type: v.mime_type,
    });
    let image_replacement = image.map(|v| ImageReplacement {
        bytes: v.bytes,
        mime_type: v.mime_type,
    });

    // Create image replacement map: element ID -> Option<ImageReplacement>
    // None means remove the element, Some means replace with image
    let image_replacements = HashMap::from([
        ("ogis_logo".to_string(), logo_replacement),
        ("ogis_image".to_string(), image_replacement),
    ]);

    let mut state = State::new(text_replacements, image_replacements);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => handle_start(e, &mut writer, &mut state)?,
            Ok(Event::Empty(e)) => handle_empty(e, &mut writer, &mut state)?,
            Ok(Event::End(e)) => handle_end(e, &mut writer, &mut state)?,
            Ok(e) => handle_default(e, &mut writer, &state)?,
            Err(e) => return Err(format!("Parse error: {:?}", e)),
        }
        buf.clear();
    }

    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| format!("UTF-8 error: {}", e))
}
