use crate::yaml_loader;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
enum FamilyFontType {
    Primary,
    Fallback,
}

impl FamilyFontType {
    fn from_index(idx: usize) -> Self {
        if idx == 0 {
            FamilyFontType::Primary
        } else {
            FamilyFontType::Fallback
        }
    }
}

pub fn load_fonts() -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();

    let doc = yaml_loader::load_yaml("fonts.yaml");

    for family in ["sans-serif", "serif", "monospace"] {
        if let Some(paths) = doc[family].as_vec() {
            for (idx, path_node) in paths.iter().enumerate() {
                if let Some(path) = path_node.as_str() {
                    let font_type = FamilyFontType::from_index(idx);
                    load_family_font_or_directory(&mut fontdb, family, path, font_type);
                }
            }
        }
    }

    if let Some(paths) = doc["global-fallback"].as_vec() {
        for path_node in paths {
            if let Some(path) = path_node.as_str() {
                load_global_fallback_or_directory(&mut fontdb, path);
            }
        }
    }

    tracing::info!("Loaded {} font faces", fontdb.faces().count());
    fontdb
}

fn load_family_font_or_directory(
    fontdb: &mut usvg::fontdb::Database,
    family: &str,
    path: &str,
    font_type: FamilyFontType,
) {
    let path_obj = Path::new(path);

    if path_obj.is_dir() {
        load_family_fonts_from_directory(fontdb, family, path, font_type);
    } else {
        load_family_font(fontdb, family, path, font_type);
    }
}

fn load_global_fallback_or_directory(fontdb: &mut usvg::fontdb::Database, path: &str) {
    let path_obj = Path::new(path);

    if path_obj.is_dir() {
        load_global_fallbacks_from_directory(fontdb, path);
    } else {
        load_global_fallback(fontdb, path);
    }
}

fn get_font_files_in_directory(dir_path: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir_path) else {
        tracing::warn!("Failed to read directory: {}", dir_path);
        return Vec::new();
    };

    let mut font_files: Vec<String> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let ext = path.extension()?.to_str()?.to_lowercase();
            if matches!(ext.as_str(), "ttf" | "ttc" | "otf") {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    // Sort to ensure consistent loading order
    font_files.sort();

    font_files
}

fn load_family_fonts_from_directory(
    fontdb: &mut usvg::fontdb::Database,
    family: &str,
    dir_path: &str,
    font_type: FamilyFontType,
) {
    for (idx, font_path) in get_font_files_in_directory(dir_path).iter().enumerate() {
        let file_font_type = match font_type {
            FamilyFontType::Primary => FamilyFontType::from_index(idx),
            FamilyFontType::Fallback => FamilyFontType::Fallback,
        };
        load_family_font(fontdb, family, font_path, file_font_type);
    }
}

fn load_global_fallbacks_from_directory(fontdb: &mut usvg::fontdb::Database, dir_path: &str) {
    for font_path in get_font_files_in_directory(dir_path) {
        load_global_fallback(fontdb, &font_path);
    }
}

#[derive(Debug)]
struct LoadedFont {
    names: Vec<String>,
}

/// Low-level function to load a font file into the database
/// Returns metadata about the loaded font
fn load_font_file(fontdb: &mut usvg::fontdb::Database, path: &str) -> Option<LoadedFont> {
    let data = yaml_loader::load_binary(path, "Font").ok()?;

    let faces_before = fontdb.faces().count();
    fontdb.load_font_data(data);
    let faces_after = fontdb.faces().count();
    let faces_loaded = faces_after - faces_before;

    if faces_loaded == 0 {
        tracing::warn!("No faces loaded from font file: {}", path);
        return None;
    }

    let names: Vec<String> = fontdb
        .faces()
        .skip(faces_before)
        .filter_map(|face| face.families.first().map(|(name, _)| name.clone()))
        .collect();

    Some(LoadedFont { names })
}

fn log_font_loaded(prefix: &str, loaded: &LoadedFont, path: &str) {
    let names_display = loaded.names.join(", ");
    tracing::info!(
        "{}: {} ({} face(s) from {})",
        prefix,
        names_display,
        loaded.names.len(),
        path
    );
}

fn load_family_font(
    fontdb: &mut usvg::fontdb::Database,
    family: &str,
    path: &str,
    font_type: FamilyFontType,
) {
    let Some(loaded) = load_font_file(fontdb, path) else {
        return;
    };

    match font_type {
        FamilyFontType::Primary => {
            if let Some(first_name) = loaded.names.first() {
                match family {
                    "sans-serif" => fontdb.set_sans_serif_family(first_name),
                    "serif" => fontdb.set_serif_family(first_name),
                    "monospace" => fontdb.set_monospace_family(first_name),
                    _ => return,
                }
            }
            log_font_loaded(&format!("Set {} primary to", family), &loaded, path);
        }
        FamilyFontType::Fallback => {
            log_font_loaded(&format!("Loaded {} fallback", family), &loaded, path);
        }
    }
}

fn load_global_fallback(fontdb: &mut usvg::fontdb::Database, path: &str) {
    let Some(loaded) = load_font_file(fontdb, path) else {
        return;
    };

    // Global fallbacks are available to all families when characters are missing
    log_font_loaded("Loaded global fallback", &loaded, path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_font_type_from_index() {
        assert!(matches!(
            FamilyFontType::from_index(0),
            FamilyFontType::Primary
        ));
        assert!(matches!(
            FamilyFontType::from_index(1),
            FamilyFontType::Fallback
        ));
        assert!(matches!(
            FamilyFontType::from_index(99),
            FamilyFontType::Fallback
        ));
    }

    #[test]
    fn test_get_font_files_nonexistent_directory() {
        assert!(get_font_files_in_directory("/nonexistent").is_empty());
    }
}
