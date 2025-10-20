use crate::yaml_loader;
use std::collections::HashMap;

pub struct TemplateMap {
    pub templates: HashMap<String, String>,
    pub default: String,
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

pub fn load_templates() -> TemplateMap {
    let mut templates = HashMap::new();

    let doc = yaml_loader::load_yaml("templates.yaml");

    // Iterate through template list
    if let Some(template_list) = doc["templates"].as_vec() {
        for template_node in template_list {
            let name = template_node["name"].as_str();
            let file_path = template_node["file"].as_str();

            if let (Some(template_name), Some(file_path)) = (name, file_path) {
                if let Ok(content) =
                    yaml_loader::load_text(file_path, &format!("template '{}'", template_name))
                {
                    templates.insert(template_name.to_string(), content);
                    tracing::info!("Loaded template '{}' from {}", template_name, file_path);
                }
            }
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
    let template_map = TemplateMap { templates, default };

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
