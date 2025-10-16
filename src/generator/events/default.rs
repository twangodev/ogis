use quick_xml::Writer;
use quick_xml::events::Event;
use std::io::Cursor;

use super::state::State;

pub fn handle_default(
    e: Event,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &State,
) -> Result<(), String> {
    // Only write if we're not inside a skipped element
    if !state.is_skipping() {
        writer
            .write_event(e)
            .map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(())
}
