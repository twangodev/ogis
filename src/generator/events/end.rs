use quick_xml::Writer;
use quick_xml::events::{BytesEnd, Event};
use std::io::Cursor;

use super::state::State;

/// Handles Event::End (closing tags like `</g>`)
pub fn handle_end(
    e: BytesEnd,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &mut State,
) -> Result<(), String> {
    // If we're inside a skipped element, decrement depth but don't write the closing tag
    if state.is_skipping() {
        state.end_skip();
        return Ok(());
    }

    // Write closing tag as-is
    writer
        .write_event(Event::End(e))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
