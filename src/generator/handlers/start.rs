use quick_xml::events::{BytesStart, Event};
use quick_xml::Writer;
use std::io::Cursor;

use super::state::State;

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
    if should_replace_element(&e) {
        // Write replacement element instead
        write_replacement(&e, writer)?;

        // Start skipping all nested content
        state.start_skip();
    } else {
        // Write element as-is
        writer
            .write_event(Event::Start(e))
            .map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(())
}

/// Check if an element should be replaced based on its attributes
fn should_replace_element(e: &BytesStart) -> bool {
    e.attributes()
        .filter_map(|a| a.ok())
        .any(|attr| {
            attr.key.as_ref() == b"id" && is_target_id(&attr.value)
        })
}

/// Check if an id value is one we want to replace
/// TODO: Make this configurable based on your needs
fn is_target_id(id_value: &[u8]) -> bool {
    // Example: replace elements with id="logo" or id="title"
    matches!(id_value, b"logo" | b"title" | b"description")
}

/// Write a replacement element
/// TODO: Customize this based on what you want to replace with
fn write_replacement(
    _original: &BytesStart,
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<(), String> {
    // For now, just write an empty placeholder
    // You can customize this to write your actual replacement content
    let replacement = BytesStart::new("g")
        .with_attributes(vec![("id", "replaced")]);

    writer
        .write_event(Event::Empty(replacement))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}