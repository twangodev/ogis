use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};

/// Font properties extracted from SVG text elements
#[derive(Debug, Clone)]
pub struct FontProperties {
    pub family: String,
    pub size: f32,
    pub weight: u16, // 400 = normal, 700 = bold
}

/// Measures the rendered width of text with given font properties using cosmic-text
/// Automatically handles per-character font fallback
fn measure_text_width(
    text: &str,
    font_props: &FontProperties,
    fontdb: &usvg::fontdb::Database,
) -> Result<f32, String> {
    if text.is_empty() {
        return Ok(0.0);
    }

    // Create FontSystem from the existing fontdb
    // cosmic-text will use the same fonts we loaded for rendering
    let mut font_system = FontSystem::new_with_locale_and_db("en-US".into(), fontdb.clone());

    // Set up text attributes matching the font properties
    let attrs = Attrs::new()
        .family(cosmic_text::Family::Name(&font_props.family))
        .weight(cosmic_text::Weight(font_props.weight));

    // Create metrics with the font size
    let metrics = Metrics::new(font_props.size, font_props.size);

    // Create a buffer and set the text
    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced, None);

    // Shape the text (this applies font fallback automatically)
    buffer.shape_until_scroll(&mut font_system, false);

    // Get the width from the first layout run
    let width = buffer
        .layout_runs()
        .next()
        .map(|run| run.line_w)
        .unwrap_or(0.0);

    Ok(width)
}

/// Truncates text to fit within the specified width, adding "…" if truncated
///
/// Uses binary search to find the maximum number of characters that fit
pub fn truncate_text_to_width(
    text: &str,
    max_width: f32,
    font_props: &FontProperties,
    fontdb: &Arc<usvg::fontdb::Database>,
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
    let ellipsis = "…";
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
    #[test]
    fn test_truncate_empty_text() {
        // This test would require setting up SwashFontCache
        // Skipping for now - integration tests would be better
    }
}
