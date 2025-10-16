use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::io::Cursor;

use super::events::{ReplacementContext, State, handle_empty, handle_end, handle_start};

const DEFAULT_TEMPLATE: &str = include_str!("../../templates/twilight.svg");

pub fn generate_svg(
    title: &str,
    description: &str,
    logo: &str,
    subtitle: &str,
    logo_image_base64: Option<&str>,
) -> Result<String, String> {
    let mut reader = Reader::from_str(DEFAULT_TEMPLATE);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Create replacement context with all the data needed for replacements
    let context = ReplacementContext::new(title, description, logo, subtitle, logo_image_base64);
    let mut state = State::new(context);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => handle_start(e, &mut writer, &mut state)?,
            Ok(Event::Empty(e)) => handle_empty(e, &mut writer, &mut state)?,
            Ok(Event::End(e)) => handle_end(e, &mut writer, &mut state)?,
            Ok(e) => {
                // Handle all other events (Text, Comment, CData, etc.)
                // Only write if we're not inside a skipped element
                if !state.is_skipping() {
                    writer
                        .write_event(e)
                        .map_err(|e| format!("Write error: {}", e))?;
                }
            }
            Err(e) => return Err(format!("Parse error: {:?}", e)),
        }
        buf.clear();
    }

    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| format!("UTF-8 error: {}", e))
}
