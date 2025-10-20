use saphyr::{LoadableYamlNode, Yaml};

/// Load and parse a YAML file
///
/// # Panics
/// Panics if the file cannot be read or parsed
pub fn load_yaml(path: &str) -> Yaml<'static> {
    let yaml_content =
        std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path));

    Yaml::load_from_str(&yaml_content)
        .unwrap_or_else(|_| panic!("Failed to parse {}", path))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{} is empty", path))
}

/// Load a binary file with logging
///
/// Returns Ok(data) on success, Err(message) on failure
/// Logs a warning if the file cannot be read
pub fn load_binary(path: &str, name: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|_e| {
        let msg = format!("{} file not found: {}", name, path);
        tracing::warn!("{}", msg);
        msg
    })
}

/// Load a text file with logging
///
/// Returns Ok(content) on success, Err(message) on failure
/// Logs a warning if the file cannot be read
pub fn load_text(path: &str, name: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| {
        let msg = format!("Failed to load {} from {}: {}", name, path, e);
        tracing::warn!("{}", msg);
        msg
    })
}
