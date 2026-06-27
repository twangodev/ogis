use std::collections::HashMap;
use std::sync::OnceLock;

/// Append-only `name → id` maps for templates and color names, embedded at build time.
pub struct Registry {
    template_to_id: HashMap<String, u16>,
    id_to_template: HashMap<u16, String>,
    color_to_id: HashMap<String, u16>,
    id_to_color: HashMap<u16, String>,
}

const TEMPLATE_IDS_JSON: &str = include_str!("template-ids.json");
const COLOR_IDS_JSON: &str = include_str!("color-ids.json");

fn invert(map: &HashMap<String, u16>) -> HashMap<u16, String> {
    map.iter().map(|(k, v)| (*v, k.clone())).collect()
}

impl Registry {
    pub fn load() -> &'static Registry {
        static REG: OnceLock<Registry> = OnceLock::new();
        REG.get_or_init(|| {
            let template_to_id: HashMap<String, u16> =
                serde_json::from_str(TEMPLATE_IDS_JSON).expect("template-ids.json");
            let color_to_id: HashMap<String, u16> =
                serde_json::from_str(COLOR_IDS_JSON).expect("color-ids.json");
            let id_to_template = invert(&template_to_id);
            let id_to_color = invert(&color_to_id);
            debug_assert_eq!(
                id_to_template.len(),
                template_to_id.len(),
                "template registry has duplicate ids"
            );
            debug_assert_eq!(
                id_to_color.len(),
                color_to_id.len(),
                "color registry has duplicate ids"
            );
            Registry {
                id_to_template,
                id_to_color,
                template_to_id,
                color_to_id,
            }
        })
    }

    pub fn template_id(&self, name: &str) -> Option<u16> {
        self.template_to_id.get(name).copied()
    }
    pub fn template_name(&self, id: u16) -> Option<&str> {
        self.id_to_template.get(&id).map(String::as_str)
    }
    pub fn color_id(&self, name: &str) -> Option<u16> {
        self.color_to_id.get(name).copied()
    }
    pub fn color_name(&self, id: u16) -> Option<&str> {
        self.id_to_color.get(&id).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// Live names from the loaded templates: template names + union of all per-template color names.
    fn live_names() -> (Vec<String>, Vec<String>) {
        let map = crate::templates::load_templates();
        let mut templates: Vec<String> = map.templates.keys().cloned().collect();
        templates.sort();
        let mut colors: std::collections::BTreeSet<String> = Default::default();
        for palette in map.colors.values() {
            for key in palette.keys() {
                colors.insert(key.clone());
            }
        }
        (templates, colors.into_iter().collect())
    }

    /// Append missing names to `committed` with the next free id (append-only).
    fn extend(committed: &mut BTreeMap<String, u16>, names: &[String]) {
        let mut next = committed
            .values()
            .copied()
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        for n in names {
            if !committed.contains_key(n) {
                committed.insert(n.clone(), next);
                next += 1;
            }
        }
    }

    #[test]
    fn registries_have_no_duplicate_or_gap_ids() {
        let template_map: HashMap<String, u16> = serde_json::from_str(TEMPLATE_IDS_JSON).unwrap();
        let color_map: HashMap<String, u16> = serde_json::from_str(COLOR_IDS_JSON).unwrap();
        for (label, map) in [("template", &template_map), ("color", &color_map)] {
            if map.is_empty() {
                continue;
            }
            let inverted: HashMap<u16, String> = map.iter().map(|(k, v)| (*v, k.clone())).collect();
            assert_eq!(
                inverted.len(),
                map.len(),
                "{label} registry has duplicate ids"
            );
            let max_id = *map.values().max().unwrap();
            assert_eq!(
                max_id as usize,
                map.len() - 1,
                "{label} registry ids are not contiguous 0..N (max={max_id}, count={})",
                map.len()
            );
        }
    }

    #[test]
    fn registries_cover_all_live_names() {
        let (templates, colors) = live_names();
        let reg = Registry::load();

        if std::env::var("OGIS_REGEN_WIRE_IDS").is_ok() {
            let mut t: BTreeMap<String, u16> = serde_json::from_str(TEMPLATE_IDS_JSON).unwrap();
            let mut c: BTreeMap<String, u16> = serde_json::from_str(COLOR_IDS_JSON).unwrap();
            extend(&mut t, &templates);
            extend(&mut c, &colors);
            std::fs::write(
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/wire/template-ids.json"),
                serde_json::to_string_pretty(&t).unwrap(),
            )
            .unwrap();
            std::fs::write(
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/wire/color-ids.json"),
                serde_json::to_string_pretty(&c).unwrap(),
            )
            .unwrap();
            return; // regenerated; rerun without the env var to assert
        }

        let missing_t: Vec<_> = templates
            .iter()
            .filter(|n| reg.template_id(n).is_none())
            .collect();
        let missing_c: Vec<_> = colors
            .iter()
            .filter(|n| reg.color_id(n).is_none())
            .collect();
        assert!(
            missing_t.is_empty() && missing_c.is_empty(),
            "registry missing live names - run `OGIS_REGEN_WIRE_IDS=1 cargo test registries_cover_all_live_names`.\n  templates: {missing_t:?}\n  colors: {missing_c:?}"
        );
    }
}
