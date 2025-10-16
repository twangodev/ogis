use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use super::state::State;
use crate::generator::strategies::apply_replacement;
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

    // Check if this element has an id we want to replace
    if let Some(id) = get_id_from_element(&e) {
        // Try to apply replacement strategy
        if apply_replacement(&e, &id, &state.context, writer).is_ok() {
            // Start skipping all nested content
            state.start_skip();
            return Ok(());
        }
        // If no replacement found, fall through to write as-is
    }

    // Write element as-is
    writer
        .write_event(Event::Start(e))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
