use crate::yaml_loader;
use std::sync::Arc;

pub fn load_fonts() -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();

    let doc = yaml_loader::load_yaml("fonts.yaml");

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

    // Load global fallback fonts (after family-specific fonts for correct priority)
    if let Some(paths) = doc["global-fallback"].as_vec() {
        for path_node in paths {
            if let Some(path) = path_node.as_str() {
                load_global_fallback(&mut fontdb, path);
            }
        }
    }

    tracing::info!("Loaded {} font faces", fontdb.faces().count());
    fontdb
}

fn load_font(fontdb: &mut usvg::fontdb::Database, family: &str, path: &str, is_primary: bool) {
    let Ok(data) = yaml_loader::load_binary(path, "Font") else {
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

fn load_global_fallback(fontdb: &mut usvg::fontdb::Database, path: &str) {
    let Ok(data) = yaml_loader::load_binary(path, "Global fallback font") else {
        return;
    };

    fontdb.load_font_data(data);

    let Some(face) = fontdb.faces().last() else {
        return;
    };

    let name = face.families[0].0.clone();

    // Global fallbacks are not assigned to any specific family
    // They are available to all families when characters are missing
    tracing::info!("Loaded global fallback: {} (from {})", name, path);
}

/// Swash font cache for text measurement
pub struct SwashFontCache {
    /// Font data storage (keeps data alive)
    font_data: Vec<Arc<Vec<u8>>>,
}

impl SwashFontCache {
    /// Get a font by family name
    pub fn get_font<'a>(&'a self, family: &str) -> Option<swash::FontRef<'a>> {
        // Find the index of the font data for this family
        // For now, we'll use a simple mapping based on family name
        let index = match family {
            "sans-serif" => 0,
            "serif" => 1,
            "monospace" => 2,
            _ => 0, // Default to sans-serif
        };

        self.font_data
            .get(index)
            .and_then(|data| swash::FontRef::from_index(data, 0))
    }
}

/// Load fonts for swash text measurement (parallel to usvg fontdb)
pub fn load_swash_fonts() -> SwashFontCache {
    let mut font_data = Vec::new();

    let doc = yaml_loader::load_yaml("fonts.yaml");

    // Load fonts from each family in order: sans-serif, serif, monospace
    for family in ["sans-serif", "serif", "monospace"] {
        if let Some(paths) = doc[family].as_vec() {
            // Use the first (primary) font for each family
            if let Some(path_node) = paths.first() {
                if let Some(path) = path_node.as_str() {
                    if let Ok(data) = yaml_loader::load_binary(path, "Font") {
                        let data_arc = Arc::new(data);

                        // Verify we can create a FontRef from this data
                        if let Some(font_ref) = swash::FontRef::from_index(&data_arc, 0) {
                            // Get font family name from localized strings
                            let font_family = font_ref
                                .localized_strings()
                                .find_map(|s| {
                                    if s.id() == swash::StringId::Family {
                                        Some(s.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_else(|| family.to_string());

                            tracing::info!(
                                "Loaded swash font for {}: {} (from {})",
                                family,
                                font_family,
                                path
                            );

                            font_data.push(data_arc);
                        } else {
                            tracing::warn!("Failed to load swash font from {}", path);
                        }
                    }
                }
            }
        }
    }

    tracing::info!("Loaded {} swash fonts", font_data.len());

    SwashFontCache { font_data }
}
