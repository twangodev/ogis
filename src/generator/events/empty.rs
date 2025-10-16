use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use super::state::State;
use crate::generator::strategies::apply_replacement;
use crate::generator::utils::get_id_from_element;

/// Handles Event::Empty (self-closing tags like `<rect id="logo" />`)
pub fn handle_empty(
    e: BytesStart,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &mut State,
) -> Result<(), String> {
    // If we're inside a skipped element, don't write this
    if state.is_skipping() {
        return Ok(());
    }

    // Check if this element has an id we want to replace
    if let Some(id) = get_id_from_element(&e) {
        // Try to apply replacement strategy
        if apply_replacement(&e, &id, &state.text_replacements, writer)? {
            // Replacement applied
            return Ok(());
        }
        // No replacement found, fall through to write as-is
    }

    // Write element as-is
    writer
        .write_event(Event::Empty(e))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
