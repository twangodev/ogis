use crate::yaml_loader;
use std::collections::HashMap;

pub struct TemplateMap {
    pub templates: HashMap<String, String>,
    pub default: String,
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
                if let Ok(content) = yaml_loader::load_text(
                    file_path,
                    &format!("template '{}'", template_name),
                ) {
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

    // Validate that the default template exists
    if !templates.contains_key(&default) {
        panic!(
            "Default template '{}' not found. Available templates: {}",
            default,
            templates.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
        );
    }

    tracing::info!("Loaded {} template(s), default: {}", templates.len(), default);

    TemplateMap { templates, default }
}