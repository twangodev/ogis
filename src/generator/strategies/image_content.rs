use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

/// Strategy for replacing a group element with an SVG <image> element
///
/// The positioning and sizing is determined by reading a <rect> child element.
/// The rect's x, y, width, and height attributes define the bounding box.
/// The image is centered and scaled to fit while maintaining aspect ratio.
///
/// Example input:
/// <g id="ogis_logo">
///   <rect x="80" y="60" width="400" height="120"/>
///   <text>...</text>
/// </g>
///
/// Example output:
/// <image x="80" y="60" width="400" height="120" preserveAspectRatio="xMidYMid meet" href="data:image/png;base64,..."/>
pub fn replace(
    rect_element: &BytesStart,
    image_bytes: &[u8],
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<(), String> {
    // Extract positioning attributes from the rect
    let x = get_attr(rect_element, b"x")?;
    let y = get_attr(rect_element, b"y")?;
    let width = get_attr(rect_element, b"width")?;
    let height = get_attr(rect_element, b"height")?;

    // Convert image bytes to base64 at write time
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    let image_base64 = BASE64.encode(image_bytes);

    // Create <image> element with centered, aspect-ratio-preserving behavior
    let mut image_elem = BytesStart::new("image");
    image_elem.push_attribute(("x", x.as_str()));
    image_elem.push_attribute(("y", y.as_str()));
    image_elem.push_attribute(("width", width.as_str()));
    image_elem.push_attribute(("height", height.as_str()));

    // Center and maintain aspect ratio (contain/letterbox behavior)
    image_elem.push_attribute(("preserveAspectRatio", "xMidYMid meet"));

    // Use href attribute with base64 data URI
    let data_uri = format!("data:image/png;base64,{}", image_base64);
    image_elem.push_attribute(("href", data_uri.as_str()));

    // Write self-closing <image/> element
    writer
        .write_event(Event::Empty(image_elem))
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}

/// Get an attribute value from an element
fn get_attr(elem: &BytesStart, attr_name: &[u8]) -> Result<String, String> {
    elem.attributes()
        .filter_map(|a| a.ok())
        .find(|attr| attr.key.as_ref() == attr_name)
        .ok_or_else(|| {
            format!(
                "Missing required attribute: {}",
                String::from_utf8_lossy(attr_name)
            )
        })
        .and_then(|attr| {
            String::from_utf8(attr.value.to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))
        })
}
