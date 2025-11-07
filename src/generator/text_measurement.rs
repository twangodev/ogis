use std::sync::Arc;

/// Font properties extracted from SVG text elements
#[derive(Debug, Clone)]
pub struct FontProperties {
    pub family: String,
    pub size: f32,
    pub weight: u16, // 400 = normal, 700 = bold
}

/// Measures the rendered width of text with given font properties using swash
fn measure_text_width(
    text: &str,
    font_props: &FontProperties,
    fontdb: &usvg::fontdb::Database,
) -> Result<f32, String> {
    if text.is_empty() {
        return Ok(0.0);
    }

    // Query fontdb using the same logic as usvg
    // Map generic family names to fontdb Family enum
    let family = match font_props.family.as_str() {
        "sans-serif" => usvg::fontdb::Family::SansSerif,
        "serif" => usvg::fontdb::Family::Serif,
        "monospace" => usvg::fontdb::Family::Monospace,
        "cursive" => usvg::fontdb::Family::Cursive,
        "fantasy" => usvg::fontdb::Family::Fantasy,
        name => usvg::fontdb::Family::Name(name),
    };

    let query = usvg::fontdb::Query {
        families: &[family],
        weight: usvg::fontdb::Weight(font_props.weight),
        ..Default::default()
    };

    let face_id = fontdb
        .query(&query)
        .ok_or_else(|| format!("Font family '{}' not found in database", font_props.family))?;

    // Access font data and measure text inside the closure
    let width = fontdb
        .with_face_data(face_id, |data, face_index| {
            // Create font reference from the data
            let font = swash::FontRef::from_index(data, face_index as usize)?;

            // Create charmap
            let charmap = font.charmap();

            // Set up variations for variable fonts
            let mut coords = Vec::new();

            // If the font supports variable weight, set it
            if let Some(wght_axis) = font
                .variations()
                .find(|v| v.tag() == swash::tag_from_bytes(b"wght"))
            {
                // Normalize the weight value to the axis range
                let normalized = wght_axis.normalize(font_props.weight as f32);
                coords.push(normalized);
            }

            // Get glyph metrics with the specified font size and variations
            let glyph_metrics = font.glyph_metrics(&coords);

            // Scale factor to convert design units to pixels
            let ppem = font_props.size;
            let scale = ppem / glyph_metrics.units_per_em() as f32;

            let mut total_width = 0.0;

            for ch in text.chars() {
                // Get glyph ID for character
                let glyph_id = charmap.map(ch);

                // Get advance width for this glyph
                let advance = glyph_metrics.advance_width(glyph_id);
                total_width += advance * scale;
            }

            Some(total_width)
        })
        .ok_or_else(|| "Failed to access font data from fontdb".to_string())?
        .ok_or_else(|| "Failed to measure text width".to_string())?;

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
