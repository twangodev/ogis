use cosmic_text::FontSystem;
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use super::error::GeneratorError;
use super::events::{
    ImageReplacement, State, handle_default, handle_empty, handle_end, handle_start,
};
use super::text_measurement::truncate_text_to_width;
use crate::image::ValidatedImage;
use crate::templates::{TemplateFonts, TemplateMap, TextWidthConstraints};

pub struct TextContent<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub subtitle: &'a str,
    pub extra: HashMap<String, String>,
}

pub struct Images {
    pub logo: Option<ValidatedImage>,
    pub image: Option<ValidatedImage>,
}

fn get_template<'a>(
    template_map: &'a TemplateMap,
    template_name: &str,
) -> Result<&'a str, GeneratorError> {
    template_map
        .templates
        .get(template_name)
        .map(|s| s.as_str())
        .ok_or_else(|| GeneratorError::TemplateNotFound {
            name: template_name.to_string(),
            available: template_map.available_templates(),
        })
}

/// Get cached font properties for a template
fn get_template_fonts<'a>(
    templates: &'a TemplateMap,
    template_name: &str,
) -> Result<&'a TemplateFonts, GeneratorError> {
    templates
        .font_properties
        .get(template_name)
        .ok_or_else(|| GeneratorError::FontPropertiesNotFound(template_name.to_string()))
}

/// Get width constraints for a template, falling back to defaults
fn get_width_constraints(templates: &TemplateMap, template_name: &str) -> TextWidthConstraints {
    templates
        .width_constraints
        .get(template_name)
        .cloned()
        .unwrap_or_default()
}

/// Truncate text to fit within width constraints using cached font properties
fn apply_truncation(
    title: &str,
    description: &str,
    subtitle: &str,
    constraints: &TextWidthConstraints,
    fonts: &TemplateFonts,
    fontdb: &Arc<usvg::fontdb::Database>,
) -> Result<(String, String, String), GeneratorError> {
    // Create FontSystem once and reuse it for all truncation operations
    // This is a major performance optimization: previously we created a new FontSystem
    // for each text measurement (21-63 times per request), now we create it just once
    let mut font_system =
        FontSystem::new_with_locale_and_db("en-US".into(), fontdb.as_ref().clone());

    let truncated_title = truncate_text_to_width(
        title,
        constraints.get_title_width(),
        &fonts.title,
        &mut font_system,
    )
    .map_err(GeneratorError::TextMeasurement)?;

    let truncated_description = truncate_text_to_width(
        description,
        constraints.get_description_width(),
        &fonts.description,
        &mut font_system,
    )
    .map_err(GeneratorError::TextMeasurement)?;

    let truncated_subtitle = truncate_text_to_width(
        subtitle,
        constraints.get_subtitle_width(),
        &fonts.subtitle,
        &mut font_system,
    )
    .map_err(GeneratorError::TextMeasurement)?;

    Ok((truncated_title, truncated_description, truncated_subtitle))
}

/// Apply color overrides to template content by replacing default hex values
fn override_colors(
    content: &str,
    template_name: &str,
    templates: &TemplateMap,
    color_overrides: &HashMap<String, String>,
) -> String {
    let Some(template_colors) = templates.colors.get(template_name) else {
        return content.to_string();
    };
    apply_color_overrides(content, template_colors, color_overrides)
}

/// Apply color overrides given an explicit colors map (for the gradient cache path
/// where the caller already has the map and wants to apply it to a partial SVG).
pub(crate) fn apply_color_overrides(
    content: &str,
    template_colors: &HashMap<String, String>,
    color_overrides: &HashMap<String, String>,
) -> String {
    let mut result = content.to_string();
    for (color_name, new_hex) in color_overrides {
        if let Some(default_hex) = template_colors.get(color_name) {
            result = result.replace(default_hex, new_hex);
        }
    }
    result
}

pub fn generate_svg(
    text: TextContent,
    images: Images,
    template_name: &str,
    templates: &TemplateMap,
    color_overrides: &HashMap<String, String>,
    fontdb: &Arc<usvg::fontdb::Database>,
    truncation: bool,
) -> Result<String, GeneratorError> {
    let template_content = get_template(templates, template_name)?;
    generate_svg_from(
        text,
        images,
        template_content,
        template_name,
        templates,
        color_overrides,
        fontdb,
        truncation,
    )
}

/// Like [`generate_svg`] but the caller supplies the SVG content directly.
/// Used by the gradient-cache path on the foreground-only SVG fragment, so the
/// rest of the pipeline (truncation, text/image substitution, color overrides)
/// runs against just the foreground half.
#[allow(clippy::too_many_arguments)]
pub fn generate_svg_from(
    text: TextContent,
    images: Images,
    svg_content: &str,
    template_name: &str,
    templates: &TemplateMap,
    color_overrides: &HashMap<String, String>,
    fontdb: &Arc<usvg::fontdb::Database>,
    truncation: bool,
) -> Result<String, GeneratorError> {
    let content = override_colors(svg_content, template_name, templates, color_overrides);

    // Apply truncation if enabled for this template
    let (truncated_title, truncated_description, truncated_subtitle) = if truncation {
        let fonts = get_template_fonts(templates, template_name)?;
        let constraints = get_width_constraints(templates, template_name);
        apply_truncation(
            text.title,
            text.description,
            text.subtitle,
            &constraints,
            fonts,
            fontdb,
        )?
    } else {
        (
            text.title.to_string(),
            text.description.to_string(),
            text.subtitle.to_string(),
        )
    };

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Create text replacement map with truncated text: element ID -> replacement text
    let mut text_replacements = HashMap::from([
        ("ogis_title".to_string(), truncated_title),
        ("ogis_description".to_string(), truncated_description),
        ("ogis_subtitle".to_string(), truncated_subtitle),
    ]);
    // Add extra text fields (no truncation applied)
    text_replacements.extend(text.extra);

    // Convert ValidatedImage to ImageReplacement
    let logo_replacement = images.logo.map(|v| ImageReplacement {
        bytes: v.bytes,
        mime_type: v.mime_type,
    });
    let image_replacement = images.image.map(|v| ImageReplacement {
        bytes: v.bytes,
        mime_type: v.mime_type,
    });

    // Create image replacement map: element ID -> Option<ImageReplacement>
    // None means remove the element, Some means replace with image
    let image_replacements = HashMap::from([
        ("ogis_logo".to_string(), logo_replacement),
        ("ogis_image".to_string(), image_replacement),
    ]);

    let mut state = State::new(text_replacements, image_replacements);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                handle_start(e, &mut writer, &mut state).map_err(GeneratorError::Xml)?
            }
            Ok(Event::Empty(e)) => {
                handle_empty(e, &mut writer, &mut state).map_err(GeneratorError::Xml)?
            }
            Ok(Event::End(e)) => {
                handle_end(e, &mut writer, &mut state).map_err(GeneratorError::Xml)?
            }
            Ok(e) => handle_default(e, &mut writer, &state).map_err(GeneratorError::Xml)?,
            Err(e) => return Err(GeneratorError::SvgParse(format!("{:?}", e))),
        }
        buf.clear();
    }

    String::from_utf8(writer.into_inner().into_inner())
        .map_err(|e| GeneratorError::Utf8(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template_fonts_not_found() {
        let templates = TemplateMap {
            templates: HashMap::new(),
            default: "default".to_string(),
            colors: HashMap::new(),
            width_constraints: HashMap::new(),
            font_properties: HashMap::new(),
            truncation: HashMap::new(),
            max_scale: HashMap::new(),
            gradient_splits: HashMap::new(),
        };

        let result = get_template_fonts(&templates, "test");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GeneratorError::FontPropertiesNotFound(_)
        ));
    }

    #[test]
    fn test_get_width_constraints_returns_defaults() {
        let templates = TemplateMap {
            templates: HashMap::new(),
            default: "default".to_string(),
            colors: HashMap::new(),
            width_constraints: HashMap::new(),
            font_properties: HashMap::new(),
            truncation: HashMap::new(),
            max_scale: HashMap::new(),
            gradient_splits: HashMap::new(),
        };

        let constraints = get_width_constraints(&templates, "test");
        assert_eq!(constraints.get_title_width(), 900.0);
    }
}
