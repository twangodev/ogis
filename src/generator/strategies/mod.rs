use quick_xml::Writer;
use quick_xml::events::BytesStart;
use std::io::Cursor;

use super::events::ReplacementContext;

mod text_content;

/// Get the replacement text for a given ID from the context
fn get_text_for_id<'a>(id: &[u8], context: &'a ReplacementContext) -> Option<&'a str> {
    match id {
        b"ogis_title" => Some(context.title),
        b"ogis_description" => Some(context.description),
        b"ogis_subtitle" => Some(context.subtitle),
        b"ogis_logo" => Some(context.logo),
        _ => None,
    }
}

/// Apply replacement strategy based on element ID
pub fn apply_replacement(
    original: &BytesStart,
    id: &[u8],
    context: &ReplacementContext,
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<(), String> {
    // For now, we only have text_content strategy
    // In the future, you can match on different ID patterns for different strategies
    if let Some(text) = get_text_for_id(id, context) {
        text_content::replace(original, text, writer)
    } else {
        Err(format!(
            "No replacement found for id: {:?}",
            String::from_utf8_lossy(id)
        ))
    }
}
