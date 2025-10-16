use quick_xml::Writer;
use quick_xml::events::Event;
use std::io::Cursor;

use crate::generator::events::State;
use crate::generator::utils::write_event;

pub fn handle_default(
    e: Event,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &State,
) -> Result<(), String> {
    // Only write if we're not inside a skipped element
    if !state.is_skipping() {
        write_event(writer, e)?;
    }

    Ok(())
}
