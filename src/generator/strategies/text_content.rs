use quick_xml::Writer;
use quick_xml::events::{BytesStart, BytesText, Event};
use std::io::Cursor;

/// Strategy for replacing text content while preserving the element and all its attributes
///
/// Example: <text id="title" x="50" y="100" class="fancy">Old Text</text>
/// Becomes: <text id="title" x="50" y="100" class="fancy">New Text</text>
pub fn replace(
    original: &BytesStart,
    text: &str,
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<(), String> {
    // Write the original element start tag with all its attributes
    writer
        .write_event(Event::Start(original.clone()))
        .map_err(|e| format!("Write error: {}", e))?;

    // Write the new text content
    writer
        .write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| format!("Write error: {}", e))?;

    // Close the element
    writer
        .write_event(Event::End(original.to_end()))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
