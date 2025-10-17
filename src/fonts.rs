use saphyr::{LoadableYamlNode, Yaml};

pub fn load_fonts() -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();

    let yaml_content = std::fs::read_to_string("fonts.yaml").expect("Failed to read fonts.yaml");

    let doc = &Yaml::load_from_str(&yaml_content).expect("Failed to parse fonts.yaml")[0];

    for family in ["sans-serif", "serif", "monospace"] {
        if let Some(paths) = doc[family].as_vec() {
            // List of fonts - first is primary, rest are fallbacks
            for (idx, path_node) in paths.iter().enumerate() {
                if let Some(path) = path_node.as_str() {
                    let is_primary = idx == 0;
                    load_font(&mut fontdb, family, path, is_primary);
                }
            }
        }
    }

    tracing::info!("Loaded {} font faces", fontdb.faces().count());
    fontdb
}

fn load_font(fontdb: &mut usvg::fontdb::Database, family: &str, path: &str, is_primary: bool) {
    let Ok(data) = std::fs::read(path) else {
        tracing::warn!("Font file not found: {}", path);
        return;
    };

    fontdb.load_font_data(data);

    let Some(face) = fontdb.faces().last() else {
        return;
    };

    let name = face.families[0].0.clone();

    // Only set the family mapping for the primary font
    // Fallback fonts are automatically used by fontdb when characters are missing
    if is_primary {
        match family {
            "sans-serif" => fontdb.set_sans_serif_family(&name),
            "serif" => fontdb.set_serif_family(&name),
            "monospace" => fontdb.set_monospace_family(&name),
            _ => return,
        }
        tracing::info!("Set {} primary to: {} (from {})", family, name, path);
    } else {
        tracing::info!("Loaded {} fallback: {} (from {})", family, name, path);
    }
}
