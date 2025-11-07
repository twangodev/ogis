use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};

const ELLIPSIS: &str = "…";

/// Font properties extracted from SVG text elements
#[derive(Debug, Clone)]
pub struct FontProperties {
    pub family: String,
    pub size: f32,
    pub weight: u16,
}

/// Measures the rendered width of text with given font properties using cosmic-text
/// Automatically handles per-character font fallback
fn measure_text_width(
    text: &str,
    font_props: &FontProperties,
    font_system: &mut FontSystem,
) -> Result<f32, String> {
    if text.is_empty() {
        return Ok(0.0);
    }

    // Set up text attributes matching the font properties
    let attrs = Attrs::new()
        .family(cosmic_text::Family::Name(&font_props.family))
        .weight(cosmic_text::Weight(font_props.weight));

    let metrics = Metrics::new(font_props.size, font_props.size);

    let mut buffer = Buffer::new(font_system, metrics);
    buffer.set_text(font_system, text, &attrs, Shaping::Advanced, None);

    buffer.shape_until_scroll(font_system, false);

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
    font_system: &mut FontSystem,
) -> Result<String, String> {
    // If empty or already fits, return as-is
    if text.is_empty() {
        return Ok(text.to_string());
    }

    let full_width = measure_text_width(text, font_props, font_system)?;
    if full_width <= max_width {
        return Ok(text.to_string());
    }

    // Measure ellipsis width
    let ellipsis_width = measure_text_width(ELLIPSIS, font_props, font_system)?;

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
        let width = measure_text_width(&substring, font_props, font_system)?;

        if width <= available_width {
            best_fit = mid;
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }

    // If no characters fit, return just ellipsis
    if best_fit == 0 {
        return Ok(ELLIPSIS.to_string());
    }

    // Construct truncated string with ellipsis
    let truncated: String = chars[..best_fit].iter().collect();
    Ok(format!("{}{}", truncated, ELLIPSIS))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_font_system() -> FontSystem {
        // Load system fonts so cosmic-text has fonts available
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();
        FontSystem::new_with_locale_and_db("en-US".into(), fontdb)
    }

    fn create_test_font_props() -> FontProperties {
        FontProperties {
            family: "sans-serif".to_string(),
            size: 24.0,
            weight: 400,
        }
    }

    #[test]
    fn test_truncate_long_text() {
        let mut font_system = create_test_font_system();
        let font_props = create_test_font_props();

        let text = "This is a very long text that should definitely be truncated";
        let result = truncate_text_to_width(text, 100.0, &font_props, &mut font_system);

        assert!(result.is_ok());
        let truncated = result.unwrap();
        assert!(truncated.len() < text.len());
        assert!(truncated.ends_with(ELLIPSIS));
    }
}
