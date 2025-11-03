use crate::yaml_loader;
use saphyr::Yaml;
use std::collections::HashMap;

pub struct TemplateMap {
    pub templates: HashMap<String, String>,
    pub default: String,
    pub colors: HashMap<String, HashMap<String, String>>,
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

/// Load a single template from a YAML node
fn load_template(
    template_node: &Yaml,
    templates: &mut HashMap<String, String>,
    colors: &mut HashMap<String, HashMap<String, String>>,
) {
    let name = template_node["name"].as_str();
    let file_path = template_node["file"].as_str();

    let (Some(template_name), Some(file_path)) = (name, file_path) else {
        return;
    };

    // Load template content
    if let Ok(content) = yaml_loader::load_text(file_path, &format!("template '{}'", template_name))
    {
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
}

pub fn load_templates() -> TemplateMap {
    let mut templates = HashMap::new();
    let mut colors = HashMap::new();

    let doc = yaml_loader::load_yaml("templates.yaml");

    // Load all templates
    if let Some(template_list) = doc["templates"].as_vec() {
        for template_node in template_list {
            load_template(template_node, &mut templates, &mut colors);
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
