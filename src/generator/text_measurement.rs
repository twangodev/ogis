use std::sync::Arc;
use usvg::fontdb;

/// Font properties extracted from SVG text elements
#[derive(Debug, Clone)]
pub struct FontProperties {
    pub family: String,
    pub size: f32,
    pub weight: u16, // 400 = normal, 700 = bold
}

impl FontProperties {
    // Note: Default font properties are created in svg.rs for each text type
}

/// Measures the rendered width of text with given font properties
///
/// Creates a minimal SVG with the text and uses usvg to measure its bounding box
fn measure_text_width(
    text: &str,
    font_props: &FontProperties,
    fontdb: &Arc<fontdb::Database>,
) -> Result<f32, String> {
    if text.is_empty() {
        return Ok(0.0);
    }

    // Create a minimal SVG with just the text element
    let svg_content = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg">
            <text font-family="{}" font-size="{}" font-weight="{}">{}</text>
        </svg>"#,
        font_props.family,
        font_props.size,
        font_props.weight,
        xml_escape(text)
    );

    let options = usvg::Options {
        fontdb: Arc::clone(fontdb),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(&svg_content, &options)
        .map_err(|e| format!("Failed to parse SVG for text measurement: {}", e))?;

    // Get the bounding box of the rendered SVG
    let bbox = tree.root().abs_bounding_box();
    Ok(bbox.width())
}

/// Escapes XML special characters in text
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Truncates text to fit within the specified width, adding "..." if truncated
///
/// Uses binary search to find the maximum number of characters that fit
pub fn truncate_text_to_width(
    text: &str,
    max_width: f32,
    font_props: &FontProperties,
    fontdb: &Arc<fontdb::Database>,
) -> Result<String, String> {
    // If empty or already fits, return as-is
    if text.is_empty() {
        return Ok(text.to_string());
    }

    let full_width = measure_text_width(text, font_props, fontdb)?;
    if full_width <= max_width {
        return Ok(text.to_string());
    }

    // Measure ellipsis width
    let ellipsis = "...";
    let ellipsis_width = measure_text_width(ellipsis, font_props, fontdb)?;

    // If even ellipsis doesn't fit, return empty string
    if ellipsis_width > max_width {
        return Ok(String::new());
    }

    // Binary search for the maximum number of characters that fit
    let chars: Vec<char> = text.chars().collect();
    let mut left = 0;
    let mut right = chars.len();
    let mut best_fit = 0;

    let available_width = max_width - ellipsis_width;

    while left <= right {
        let mid = (left + right) / 2;
        if mid == 0 {
            break;
        }

        let substring: String = chars[..mid].iter().collect();
        let width = measure_text_width(&substring, font_props, fontdb)?;

        if width <= available_width {
            best_fit = mid;
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }

    // If no characters fit, return just ellipsis
    if best_fit == 0 {
        return Ok(ellipsis.to_string());
    }

    // Construct truncated string with ellipsis
    let truncated: String = chars[..best_fit].iter().collect();
    Ok(format!("{}{}", truncated, ellipsis))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("Hello & <World>"), "Hello &amp; &lt;World&gt;");
        assert_eq!(xml_escape("Quote: \"test\""), "Quote: &quot;test&quot;");
    }

    #[test]
    fn test_truncate_empty_text() {
        let fontdb = Arc::new(fontdb::Database::new());
        let font_props = FontProperties {
            family: "sans-serif".to_string(),
            size: 28.0,
            weight: 400,
        };

        let result = truncate_text_to_width("", 100.0, &font_props, &fontdb);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
