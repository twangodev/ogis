use quick_xml::events::BytesStart;

/// Get an attribute value from an element as a String
///
/// Returns an error if the attribute is missing or contains invalid UTF-8
pub fn get_attr(elem: &BytesStart, attr_name: &str) -> Result<String, String> {
    elem.attributes()
        .filter_map(|a| a.ok())
        .find(|attr| attr.key.as_ref() == attr_name.as_bytes())
        .ok_or_else(|| format!("Missing required attribute: {}", attr_name))
        .and_then(|attr| {
            String::from_utf8(attr.value.to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))
        })
}

/// Extract the ID attribute value from an element, if it exists
///
/// Returns None if the element has no id attribute or if it contains invalid UTF-8
pub fn get_id_from_element(element: &BytesStart) -> Option<String> {
    get_attr(element, "id").ok()
}
