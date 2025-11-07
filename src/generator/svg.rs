use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use super::events::{
    ImageReplacement, State, handle_default, handle_empty, handle_end, handle_start,
};
use super::text_measurement::truncate_text_to_width;
use crate::fonts::SwashFontCache;
use crate::image::ValidatedImage;
use crate::templates::{TemplateFonts, TemplateMap, TextWidthConstraints};

fn get_template<'a>(template_map: &'a TemplateMap, template_name: &str) -> Result<&'a str, String> {
    template_map
        .templates
        .get(template_name)
        .map(|s| s.as_str())
        .ok_or_else(|| {
            format!(
                "Template '{}' not found. Available templates: {}",
                template_name,
                template_map.available_templates()
            )
        })
}

/// Get cached font properties for a template
fn get_template_fonts<'a>(templates: &'a TemplateMap, template_name: &str) -> &'a TemplateFonts {
    templates
        .font_properties
        .get(template_name)
        .expect("Template fonts should have been parsed during loading")
}

/// Get width constraints for a template, falling back to defaults
fn get_width_constraints(templates: &TemplateMap, template_name: &str) -> TextWidthConstraints {
    templates
        .width_constraints
        .get(template_name)
        .cloned()
        .unwrap_or_else(TextWidthConstraints::new)
}

/// Truncate text to fit within width constraints using cached font properties
fn apply_truncation(
    title: &str,
    description: &str,
    subtitle: &str,
    constraints: &TextWidthConstraints,
    fonts: &TemplateFonts,
    swash_fonts: &Arc<SwashFontCache>,
) -> Result<(String, String, String), String> {
    let truncated_title = truncate_text_to_width(
        title,
        constraints.get_title_width(),
        &fonts.title,
        swash_fonts,
    )?;

    let truncated_description = truncate_text_to_width(
        description,
        constraints.get_description_width(),
        &fonts.description,
        swash_fonts,
    )?;

    let truncated_subtitle = truncate_text_to_width(
        subtitle,
        constraints.get_subtitle_width(),
        &fonts.subtitle,
        swash_fonts,
    )?;

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

    let mut result = content.to_string();
    for (color_name, new_hex) in color_overrides {
        if let Some(default_hex) = template_colors.get(color_name) {
            result = result.replace(default_hex, new_hex);
        }
    }
    result
}

pub fn generate_svg(
    title: &str,
    description: &str,
    subtitle: &str,
    logo: Option<ValidatedImage>,
    image: Option<ValidatedImage>,
    template_name: &str,
    templates: &TemplateMap,
    color_overrides: &HashMap<String, String>,
    swash_fonts: &Arc<SwashFontCache>,
) -> Result<String, String> {
    let template_content = get_template(templates, template_name)?;
    let content = override_colors(template_content, template_name, templates, color_overrides);

    // Get cached font properties and width constraints, then apply truncation
    let fonts = get_template_fonts(templates, template_name);
    let constraints = get_width_constraints(templates, template_name);
    let (truncated_title, truncated_description, truncated_subtitle) = apply_truncation(
        title,
        description,
        subtitle,
        &constraints,
        fonts,
        swash_fonts,
    )?;

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Create text replacement map with truncated text: element ID -> replacement text
    let text_replacements = HashMap::from([
        ("ogis_title".to_string(), truncated_title),
        ("ogis_description".to_string(), truncated_description),
        ("ogis_subtitle".to_string(), truncated_subtitle),
    ]);

    // Convert ValidatedImage to ImageReplacement
    let logo_replacement = logo.map(|v| ImageReplacement {
        bytes: v.bytes,
        mime_type: v.mime_type,
    });
    let image_replacement = image.map(|v| ImageReplacement {
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
            Ok(Event::Start(e)) => handle_start(e, &mut writer, &mut state)?,
            Ok(Event::Empty(e)) => handle_empty(e, &mut writer, &mut state)?,
            Ok(Event::End(e)) => handle_end(e, &mut writer, &mut state)?,
            Ok(e) => handle_default(e, &mut writer, &state)?,
            Err(e) => return Err(format!("Parse error: {:?}", e)),
        }
        buf.clear();
    }

    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| format!("UTF-8 error: {}", e))
}
