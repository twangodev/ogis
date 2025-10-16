use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use crate::generator::events::{State, replacements};
use crate::generator::strategies::apply_text_replacement;
use crate::generator::utils::{get_id_from_element, write_event};

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

    // If we're currently processing an image replacement, handle elements inside the group
    if state.replacement_id.is_some() {
        return replacements::image::handle_element_inside_image_group(&e, writer, state);
    }

    // Check if this element has an id we want to replace
    if let Some(id) = get_id_from_element(&e) {
        // Try image replacement first (for group elements)
        if replacements::image::try_start_image_replacement(&id, state) {
            return Ok(());
        }

        // Try text replacement (needs bytes for now)
        if apply_text_replacement(&e, id.as_bytes(), &state.text_replacements, writer)? {
            state.start_skip();
            return Ok(());
        }
    }

    // Write element as-is
    write_event(writer, Event::Start(e))
}
