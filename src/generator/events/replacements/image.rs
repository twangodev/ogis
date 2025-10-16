use quick_xml::Writer;
use quick_xml::events::BytesStart;
use std::io::Cursor;

use crate::generator::events::State;
use crate::generator::strategies::image_content;

/// Handle the rect element that defines image bounds for replacement
///
/// When we encounter a <rect> inside a group marked for image replacement,
/// this function reads the rect's attributes and creates an <image> element.
pub fn handle_rect_for_image_replacement(
    rect: &BytesStart,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &mut State,
) -> Result<(), String> {
    let id = state.awaiting_rect_for.as_ref().unwrap();

    // Check if we have image bytes for this ID
    if let Some(Some(image_bytes)) = state.image_replacements.get(id) {
        // Replace with image element using rect's positioning attributes
        image_content::replace(rect, image_bytes, writer)?;
    }
    // If None or no entry, we remove the entire group (skip remaining children)

    // Clear the awaiting state and start skipping remaining group children
    state.awaiting_rect_for = None;
    state.start_skip();

    Ok(())
}

/// Handle elements encountered while inside an image replacement group
///
/// When we're waiting for a rect to define image bounds, this handles
/// any child elements we encounter.
pub fn handle_element_inside_image_group(
    e: &BytesStart,
    writer: &mut Writer<Cursor<Vec<u8>>>,
    state: &mut State,
) -> Result<(), String> {
    // Check if this is the rect we're waiting for
    if is_rect_element(e) {
        return handle_rect_for_image_replacement(e, writer, state);
    }

    // Not a rect, skip this element (we're inside the group waiting for rect)
    Ok(())
}

/// Try to start an image replacement for a group element with the given ID
///
/// Returns true if this element should be replaced with an image,
/// which means we'll wait for a child rect element to define the bounds.
pub fn try_start_image_replacement(id_str: &str, state: &mut State) -> bool {
    if state.image_replacements.contains_key(id_str) {
        // Start waiting for the rect child to define image bounds
        state.awaiting_rect_for = Some(id_str.to_string());
        true // Don't write the <g> element
    } else {
        false
    }
}

/// Check if an element is a rect
fn is_rect_element(e: &BytesStart) -> bool {
    e.name().as_ref() == b"rect"
}
