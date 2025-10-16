use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use crate::generator::events::{State, replacements};
use crate::generator::strategies::apply_text_replacement;
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

    // If we're awaiting a rect for image replacement, handle elements inside the group
    if state.awaiting_rect_for.is_some() {
        return replacements::image::handle_element_inside_image_group(&e, writer, state);
    }

    // Check if this element has an id we want to replace
    if let Some(id) = get_id_from_element(&e) {
        let id_str = String::from_utf8_lossy(&id);

        // Try image replacement first (for group elements)
        if replacements::image::try_start_image_replacement(&id_str, state) {
            return Ok(());
        }

        // Try text replacement
        if apply_text_replacement(&e, &id, &state.text_replacements, writer)? {
            state.start_skip();
            return Ok(());
        }
    }

    // Write element as-is
    writer
        .write_event(Event::Start(e))
        .map_err(|e| format!("Write error: {}", e))
}
