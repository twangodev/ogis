use quick_xml::Writer;
use quick_xml::events::BytesStart;
use std::collections::HashMap;
use std::io::Cursor;

pub mod image_content;
mod text_content;

/// Apply text replacement strategy based on element ID
///
/// Returns Ok(true) if replacement was applied, Ok(false) if no replacement found (element should be written as-is)
pub fn apply_text_replacement(
    original: &BytesStart,
    id: &[u8],
    text_replacements: &HashMap<String, String>,
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<bool, String> {
    // Convert id bytes to string for HashMap lookup
    let id_str = String::from_utf8_lossy(id);

    if let Some(text) = text_replacements.get(id_str.as_ref()) {
        text_content::replace(original, text, writer)?;
        Ok(true) // Replacement applied
    } else {
        Ok(false) // No replacement needed, element should be written as-is
    }
}
