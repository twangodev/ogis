use quick_xml::Writer;
use quick_xml::events::{BytesStart, BytesText, Event};
use std::io::Cursor;

use crate::generator::utils::write_event;

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
    write_event(writer, Event::Start(original.clone()))?;

    // Write the new text content
    write_event(writer, Event::Text(BytesText::new(text)))?;

    // Close the element
    write_event(writer, Event::End(original.to_end()))
}
