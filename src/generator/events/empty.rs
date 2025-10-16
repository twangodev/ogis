use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use super::state::State;
use crate::generator::strategies::image_content;

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

    // If we're awaiting a rect for image replacement, check if this is it
    if state.awaiting_rect_for.is_some() {
        // Check if this is a <rect> element
        if e.name().as_ref() == b"rect" {
            // Get the image bytes for the ID we're waiting for
            let id = state.awaiting_rect_for.as_ref().unwrap();
            if let Some(Some(image_bytes)) = state.image_replacements.get(id) {
                // Some(image_bytes) means replace with image
                image_content::replace(&e, image_bytes, writer)?;

                // Clear the awaiting state and start skipping remaining siblings
                state.awaiting_rect_for = None;
                state.start_skip();
                return Ok(());
            }
            // None means remove the element entirely - just skip remaining siblings
            state.awaiting_rect_for = None;
            state.start_skip();
            return Ok(());
        }
        // Not a rect, skip this element (we're inside the group waiting for rect)
        return Ok(());
    }

    // Write element as-is
    writer
        .write_event(Event::Empty(e))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
