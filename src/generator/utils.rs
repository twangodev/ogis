use quick_xml::events::BytesStart;

/// Extract the ID attribute value from an element, if it exists
pub fn get_id_from_element(element: &BytesStart) -> Option<Vec<u8>> {
    element
        .attributes()
        .filter_map(|a| a.ok())
        .find(|attr| attr.key.as_ref() == b"id")
        .map(|attr| attr.value.to_vec())
}
