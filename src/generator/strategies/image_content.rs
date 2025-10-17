use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};
use std::io::Cursor;

use crate::generator::utils::{get_attr, write_event};

/// Strategy for replacing a group element with an SVG <image> element with rounded corners
///
/// The positioning and sizing is determined by reading a <rect> child element.
/// The rect's x, y, width, height, and rx attributes define the bounding box and corner radius.
/// The image is centered and scaled to fit while maintaining aspect ratio.
///
/// Example input:
/// <g id="ogis_logo">
///   <rect x="80" y="60" width="400" height="120" rx="12"/>
///   <text>...</text>
/// </g>
///
/// Example output:
/// <g>
///   <defs>
///     <clipPath id="clip_80_60"><rect x="80" y="60" width="400" height="120" rx="12"/></clipPath>
///   </defs>
///   <image x="80" y="60" width="400" height="120" preserveAspectRatio="xMidYMid meet" clip-path="url(#clip_80_60)" href="data:image/png;base64,..."/>
/// </g>
pub fn replace(
    rect_element: &BytesStart,
    image_bytes: &[u8],
    mime_type: &str,
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<(), String> {
    // Extract positioning attributes from the rect
    let x = get_attr(rect_element, "x")?;
    let y = get_attr(rect_element, "y")?;
    let width = get_attr(rect_element, "width")?;
    let height = get_attr(rect_element, "height")?;

    // Extract rx (border radius) if present, default to "0" if not
    let rx = get_attr(rect_element, "rx").unwrap_or_else(|_| "0".to_string());

    // Convert image bytes to base64 at write time
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    let image_base64 = BASE64.encode(image_bytes);

    // Generate unique clip path ID based on position
    let clip_id = format!("clip_{}_{}", x, y);

    // Start wrapper group
    write_event(writer, Event::Start(BytesStart::new("g")))?;

    // Create <defs> with clip path for rounded corners
    write_event(writer, Event::Start(BytesStart::new("defs")))?;

    let mut clip_path = BytesStart::new("clipPath");
    clip_path.push_attribute(("id", clip_id.as_str()));
    write_event(writer, Event::Start(clip_path.clone()))?;

    let mut clip_rect = BytesStart::new("rect");
    clip_rect.push_attribute(("x", x.as_str()));
    clip_rect.push_attribute(("y", y.as_str()));
    clip_rect.push_attribute(("width", width.as_str()));
    clip_rect.push_attribute(("height", height.as_str()));
    clip_rect.push_attribute(("rx", rx.as_str()));
    write_event(writer, Event::Empty(clip_rect))?;

    write_event(writer, Event::End(clip_path.to_end()))?;
    write_event(writer, Event::End(BytesStart::new("defs").to_end()))?;

    // Create <image> element with centered, aspect-ratio-preserving behavior
    let mut image_elem = BytesStart::new("image");
    image_elem.push_attribute(("x", x.as_str()));
    image_elem.push_attribute(("y", y.as_str()));
    image_elem.push_attribute(("width", width.as_str()));
    image_elem.push_attribute(("height", height.as_str()));
    image_elem.push_attribute(("preserveAspectRatio", "xMidYMid meet"));

    // Use href attribute with base64 data URI (with correct MIME type)
    let data_uri = format!("data:{};base64,{}", mime_type, image_base64);
    image_elem.push_attribute(("href", data_uri.as_str()));

    // Apply clip path for rounded corners
    let clip_path_url = format!("url(#{})", clip_id);
    image_elem.push_attribute(("clip-path", clip_path_url.as_str()));

    write_event(writer, Event::Empty(image_elem))?;

    // Close wrapper group
    write_event(writer, Event::End(BytesStart::new("g").to_end()))
}
