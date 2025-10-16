use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use super::state::State;
use crate::generator::strategies::{apply_text_replacement, image_content};
use crate::generator::utils::get_id_from_element;

/// Handles Event::Start (opening tags like `<g id="logo">`)
pub fn handle_start(
    e: BytesStart,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &mut State,
) -> Result<(), String> {
    // If we're already skipping, just increment depth and skip this element
    if state.is_skipping() {
        state.start_skip();
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

                // Clear the awaiting state and start skipping remaining children
                state.awaiting_rect_for = None;
                state.start_skip();
                return Ok(());
            }
            // None means remove the element entirely - just skip
            state.awaiting_rect_for = None;
            state.start_skip();
            return Ok(());
        }
        // Not a rect yet, skip this element (we're inside the group)
        return Ok(());
    }

    // Check if this element has an id we want to replace
    if let Some(id) = get_id_from_element(&e) {
        let id_str = String::from_utf8_lossy(&id);

        // Check if this is an image replacement (group element)
        if state.image_replacements.contains_key(id_str.as_ref()) {
            // Start waiting for the rect child to define image bounds
            state.awaiting_rect_for = Some(id_str.to_string());
            // Don't write the <g> element
            return Ok(());
        }

        // Try to apply text replacement strategy
        if apply_text_replacement(&e, &id, &state.text_replacements, writer)? {
            // Replacement applied, skip all nested content
            state.start_skip();
            return Ok(());
        }
        // No replacement found, fall through to write as-is
    }

    // Write element as-is
    writer
        .write_event(Event::Start(e))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
