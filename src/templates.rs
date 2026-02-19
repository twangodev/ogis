use crate::generator::FontProperties;
use crate::yaml_loader;
use quick_xml::Reader;
use quick_xml::events::Event;
use saphyr::Yaml;
use std::collections::HashMap;

/// Width constraints for text elements in a template
#[derive(Debug, Clone, Default)]
pub struct TextWidthConstraints {
    pub title: Option<f32>,
    pub description: Option<f32>,
    pub subtitle: Option<f32>,
}

/// Font properties for all text elements in a template
#[derive(Debug, Clone)]
pub struct TemplateFonts {
    pub title: FontProperties,
    pub description: FontProperties,
    pub subtitle: FontProperties,
}

impl TextWidthConstraints {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get width constraint with fallback to 75% of canvas width (900px for 1200px canvas)
    pub fn get_title_width(&self) -> f32 {
        self.title.unwrap_or(900.0)
    }

    pub fn get_description_width(&self) -> f32 {
        self.description.unwrap_or(900.0)
    }

    pub fn get_subtitle_width(&self) -> f32 {
        self.subtitle.unwrap_or(900.0)
    }
}

pub struct TemplateMap {
    pub templates: HashMap<String, String>,
    pub default: String,
    pub colors: HashMap<String, HashMap<String, String>>,
    pub width_constraints: HashMap<String, TextWidthConstraints>,
    pub font_properties: HashMap<String, TemplateFonts>,
    pub truncation: HashMap<String, bool>,
}

impl TemplateMap {
    /// Returns a comma-separated list of available template names
    pub fn available_templates(&self) -> String {
        self.templates
            .keys()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Parse color definitions from a template YAML node
fn parse_template_colors(template_node: &Yaml) -> Option<HashMap<String, String>> {
    let template_mapping = match template_node {
        Yaml::Mapping(m) => m,
        _ => return None,
    };

    let colors_value = template_mapping
        .iter()
        .find(|(key, _)| key.as_str() == Some("colors"))?
        .1;

    let colors_mapping = match colors_value {
        Yaml::Mapping(m) => m,
        _ => return None,
    };

    let template_colors: HashMap<String, String> = colors_mapping
        .iter()
        .filter_map(|(color_key, color_value)| {
            let key_str = color_key.as_str()?;
            let value_str = color_value.as_str()?;
            Some((key_str.to_string(), value_str.to_string()))
        })
        .collect();

    if template_colors.is_empty() {
        None
    } else {
        Some(template_colors)
    }
}

/// Parse truncation setting from a template YAML node (defaults to true)
fn parse_truncation(template_node: &Yaml) -> bool {
    let Yaml::Mapping(m) = template_node else {
        return true;
    };
    m.iter()
        .find(|(key, _)| key.as_str() == Some("truncation"))
        .and_then(|(_, v)| v.as_bool())
        .unwrap_or(true)
}

/// Parse width constraints from a template YAML node
fn parse_width_constraints(template_node: &Yaml) -> TextWidthConstraints {
    let mut constraints = TextWidthConstraints::new();

    let template_mapping = match template_node {
        Yaml::Mapping(m) => m,
        _ => return constraints,
    };

    // Look for max_widths section
    if let Some((_, widths_value)) = template_mapping
        .iter()
        .find(|(key, _)| key.as_str() == Some("max_widths"))
        && let Yaml::Mapping(widths) = widths_value
    {
        // Parse title width
        if let Some((_, title_value)) = widths.iter().find(|(k, _)| k.as_str() == Some("title")) {
            if let Some(width) = title_value.as_integer() {
                constraints.title = Some(width as f32);
            } else if let Some(width) = title_value.as_floating_point() {
                constraints.title = Some(width as f32);
            }
        }

        // Parse description width
        if let Some((_, desc_value)) = widths
            .iter()
            .find(|(k, _)| k.as_str() == Some("description"))
        {
            if let Some(width) = desc_value.as_integer() {
                constraints.description = Some(width as f32);
            } else if let Some(width) = desc_value.as_floating_point() {
                constraints.description = Some(width as f32);
            }
        }

        // Parse subtitle width
        if let Some((_, subtitle_value)) =
            widths.iter().find(|(k, _)| k.as_str() == Some("subtitle"))
        {
            if let Some(width) = subtitle_value.as_integer() {
                constraints.subtitle = Some(width as f32);
            } else if let Some(width) = subtitle_value.as_floating_point() {
                constraints.subtitle = Some(width as f32);
            }
        }
    }

    constraints
}

/// Extract font properties from an SVG element by ID
/// Note: Font properties may be on a parent <text> element, not on the <tspan> with the ID
fn extract_font_from_svg(svg_content: &str, element_id: &str) -> FontProperties {
    let mut reader = Reader::from_str(svg_content);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut current_font = FontProperties {
        family: "sans-serif".to_string(),
        size: 28.0,
        weight: 400,
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                // Extract font properties from this element (could be parent <text>)
                let mut temp_family = current_font.family.clone();
                let mut temp_size = current_font.size;
                let mut temp_weight = current_font.weight;
                let mut found_target_id = false;

                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let key = attr.key.as_ref();
                    if let Ok(value) = std::str::from_utf8(&attr.value) {
                        match key {
                            b"id" => {
                                if value == element_id {
                                    found_target_id = true;
                                }
                            }
                            b"font-family" => temp_family = value.to_string(),
                            b"font-size" => {
                                if let Ok(s) = value.parse::<f32>() {
                                    temp_size = s;
                                }
                            }
                            b"font-weight" => {
                                temp_weight = match value {
                                    "bold" => 700,
                                    "normal" => 400,
                                    _ => value.parse().unwrap_or(400),
                                };
                            }
                            _ => {}
                        }
                    }
                }

                // Update current font context with this element's properties
                current_font.family = temp_family;
                current_font.size = temp_size;
                current_font.weight = temp_weight;

                // If we found the target ID, return current font properties
                if found_target_id {
                    return current_font;
                }
            }
            _ => {}
        }
        buf.clear();
    }

    current_font
}

/// Parse font properties from template SVG content
fn parse_font_properties(svg_content: &str) -> TemplateFonts {
    let title = extract_font_from_svg(svg_content, "ogis_title");
    let description = extract_font_from_svg(svg_content, "ogis_description");
    let subtitle = extract_font_from_svg(svg_content, "ogis_subtitle");

    TemplateFonts {
        title,
        description,
        subtitle,
    }
}

/// Load a single template from a YAML node
fn load_template(
    template_node: &Yaml,
    templates: &mut HashMap<String, String>,
    colors: &mut HashMap<String, HashMap<String, String>>,
    width_constraints: &mut HashMap<String, TextWidthConstraints>,
    font_properties: &mut HashMap<String, TemplateFonts>,
    truncation: &mut HashMap<String, bool>,
) {
    let name = template_node["name"].as_str();
    let file_path = template_node["file"].as_str();

    let (Some(template_name), Some(file_path)) = (name, file_path) else {
        return;
    };

    // Load template content
    if let Ok(content) = yaml_loader::load_text(file_path, &format!("template '{}'", template_name))
    {
        // Parse font properties from SVG content
        let fonts = parse_font_properties(&content);
        tracing::info!(
            "Parsed fonts for '{}': title={}px/w{}, desc={}px/w{}, subtitle={}px/w{}",
            template_name,
            fonts.title.size,
            fonts.title.weight,
            fonts.description.size,
            fonts.description.weight,
            fonts.subtitle.size,
            fonts.subtitle.weight
        );
        font_properties.insert(template_name.to_string(), fonts);

        templates.insert(template_name.to_string(), content);
        tracing::info!("Loaded template '{}' from {}", template_name, file_path);
    }

    // Parse and load color definitions if present
    if let Some(template_colors) = parse_template_colors(template_node) {
        let color_count = template_colors.len();
        colors.insert(template_name.to_string(), template_colors);
        tracing::info!(
            "Loaded {} color(s) for template '{}'",
            color_count,
            template_name
        );
    }

    // Parse and load width constraints
    let constraints = parse_width_constraints(template_node);
    width_constraints.insert(template_name.to_string(), constraints);

    // Parse truncation setting
    let trunc = parse_truncation(template_node);
    truncation.insert(template_name.to_string(), trunc);
}

#[cfg(test)]
mod tests {
    use super::*;
    use saphyr::{LoadableYamlNode, Yaml};

    fn parse_yaml(s: &str) -> Yaml<'static> {
        Yaml::load_from_str(s).unwrap().into_iter().next().unwrap()
    }

    #[test]
    fn test_parse_truncation_defaults_to_true() {
        let yaml = parse_yaml("name: test\nfile: test.svg\n");
        assert!(parse_truncation(&yaml));
    }

    #[test]
    fn test_parse_truncation_false() {
        let yaml = parse_yaml("name: test\ntruncation: false\n");
        assert!(!parse_truncation(&yaml));
    }

    #[test]
    fn test_parse_truncation_true() {
        let yaml = parse_yaml("name: test\ntruncation: true\n");
        assert!(parse_truncation(&yaml));
    }

    #[test]
    fn test_parse_truncation_non_mapping() {
        let yaml = parse_yaml("just a string");
        assert!(parse_truncation(&yaml));
    }
}

pub fn load_templates() -> TemplateMap {
    let mut templates = HashMap::new();
    let mut colors = HashMap::new();
    let mut width_constraints = HashMap::new();
    let mut font_properties = HashMap::new();
    let mut truncation = HashMap::new();

    let doc = yaml_loader::load_yaml("templates.yaml");

    // Load all templates
    if let Some(template_list) = doc["templates"].as_vec() {
        for template_node in template_list {
            load_template(
                template_node,
                &mut templates,
                &mut colors,
                &mut width_constraints,
                &mut font_properties,
                &mut truncation,
            );
        }
    }

    if templates.is_empty() {
        panic!("No templates were loaded from templates.yaml");
    }

    // Read default template
    let default = doc["default"]
        .as_str()
        .expect("Missing 'default' field in templates.yaml")
        .to_string();

    // Construct the TemplateMap
    let template_map = TemplateMap {
        templates,
        default,
        colors,
        width_constraints,
        font_properties,
        truncation,
    };

    // Validate that the default template exists
    if !template_map.templates.contains_key(&template_map.default) {
        panic!(
            "Default template '{}' not found. Available templates: {}",
            template_map.default,
            template_map.available_templates()
        );
    }

    tracing::info!(
        "Loaded {} template(s), default: {}",
        template_map.templates.len(),
        template_map.default
    );

    template_map
}
