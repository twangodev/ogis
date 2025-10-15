use saphyr::{LoadableYamlNode, Yaml};

pub fn load_fonts() -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();

    let yaml_content = std::fs::read_to_string("fonts.yaml").expect("Failed to read fonts.yaml");

    let doc = &Yaml::load_from_str(&yaml_content).expect("Failed to parse fonts.yaml")[0];

    for family in ["sans-serif", "serif", "monospace"] {
        if let Some(path) = doc[family].as_str() {
            load_font(&mut fontdb, family, path);
        }
    }

    tracing::info!("Loaded {} font faces", fontdb.faces().count());
    fontdb
}

fn load_font(fontdb: &mut usvg::fontdb::Database, family: &str, path: &str) {
    let Ok(data) = std::fs::read(path) else {
        tracing::warn!("Font file not found: {}", path);
        return;
    };

    fontdb.load_font_data(data);

    let Some(face) = fontdb.faces().last() else {
        return;
    };

    let name = face.families[0].0.clone();

    match family {
        "sans-serif" => fontdb.set_sans_serif_family(&name),
        "serif" => fontdb.set_serif_family(&name),
        "monospace" => fontdb.set_monospace_family(&name),
        _ => return,
    }

    tracing::info!("Set {} to: {} (from {})", family, name, path);
}
