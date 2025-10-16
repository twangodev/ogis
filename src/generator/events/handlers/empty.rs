use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use crate::generator::events::{State, replacements};

/// Handles Event::Empty (self-closing tags like `<rect ... />`)
pub fn handle_empty(
    e: BytesStart,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &mut State,
) -> Result<(), String> {
    // If we're already skipping, skip this element
    if state.is_skipping() {
        return Ok(());
    }

    // If we're awaiting a rect for image replacement, handle elements inside the group
    if state.awaiting_rect_for.is_some() {
        return replacements::image::handle_element_inside_image_group(&e, writer, state);
    }

    // Write element as-is
    writer
        .write_event(Event::Empty(e))
        .map_err(|e| format!("Write error: {}", e))
}
