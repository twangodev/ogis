use crate::generator::FontProperties;
use crate::yaml_loader;
use quick_xml::Reader;
use quick_xml::events::Event;
use saphyr::Yaml;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Public types — consumed by the rest of the codebase
// ---------------------------------------------------------------------------

/// Width constraints for text elements in a template
#[derive(Debug, Clone, Default)]
pub struct TextWidthConstraints {
    pub title: Option<f32>,
    pub description: Option<f32>,
    pub subtitle: Option<f32>,
}

impl TextWidthConstraints {
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

/// Font properties for all text elements in a template
#[derive(Debug, Clone)]
pub struct TemplateFonts {
    pub title: FontProperties,
    pub description: FontProperties,
    pub subtitle: FontProperties,
}

pub struct TemplateMap {
    pub templates: HashMap<String, String>,
    pub default: String,
    pub colors: HashMap<String, HashMap<String, String>>,
    pub width_constraints: HashMap<String, TextWidthConstraints>,
    pub font_properties: HashMap<String, TemplateFonts>,
    pub truncation: HashMap<String, bool>,
    pub max_scale: HashMap<String, f32>,
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

// ---------------------------------------------------------------------------
// Gradient composition types
// ---------------------------------------------------------------------------

/// A single stop in a gradient (linear or radial)
struct GradientStop {
    offset: f32,
    color: String,
    opacity: Option<f32>,
}

/// A radial gradient blob definition
struct BlobDef {
    name: String,
    cx: f32,
    cy: f32,
    r: f32,
    stops: Vec<GradientStop>,
}

/// Noise filter configuration
struct NoiseDef {
    coarse_opacity: f32,
    fine_opacity: f32,
    coarse_frequency: f32,
    coarse_octaves: u32,
    coarse_slope: f32,
    coarse_intercept: f32,
    fine_frequency: f32,
    fine_octaves: u32,
    fine_slope: f32,
    fine_intercept: f32,
}

impl Default for NoiseDef {
    fn default() -> Self {
        Self {
            coarse_opacity: 0.18,
            fine_opacity: 0.10,
            coarse_frequency: 0.4,
            coarse_octaves: 3,
            coarse_slope: 1.5,
            coarse_intercept: -0.3,
            fine_frequency: 1.2,
            fine_octaves: 5,
            fine_slope: 2.0,
            fine_intercept: -0.5,
        }
    }
}

/// Text color configuration for a gradient
struct GradientTextColors {
    title: String,
    description: String,
    subtitle: String,
    subtitle_opacity: f32,
    desc_opacity: f32,
}

impl Default for GradientTextColors {
    fn default() -> Self {
        Self {
            title: "#ffffff".to_string(),
            description: "#ffffff".to_string(),
            subtitle: "#ffffff".to_string(),
            subtitle_opacity: 0.9,
            desc_opacity: 0.85,
        }
    }
}

/// Complete gradient definition
struct GradientDef {
    base_stops: Vec<GradientStop>,
    direction: [f32; 4],
    blobs: Vec<BlobDef>,
    noise: NoiseDef,
    text_colors: GradientTextColors,
}

/// Everything needed to register a single template into the TemplateMap
struct TemplateEntry {
    name: String,
    svg: String,
    colors: HashMap<String, String>,
    fonts: TemplateFonts,
    width_constraints: TextWidthConstraints,
    truncation: bool,
    max_scale: f32,
}

// ---------------------------------------------------------------------------
// Safe YAML helpers — saphyr's Index trait panics on missing keys
// ---------------------------------------------------------------------------

fn yaml_get<'a, 'b>(node: &'a Yaml<'b>, key: &str) -> Option<&'a Yaml<'b>> {
    match node {
        Yaml::Mapping(m) => m
            .iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .map(|(_, v)| v),
        _ => None,
    }
}

fn yaml_str(node: &Yaml, key: &str) -> Option<String> {
    yaml_get(node, key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn yaml_num_val(node: &Yaml) -> Option<f32> {
    node.as_floating_point()
        .map(|f| f as f32)
        .or_else(|| node.as_integer().map(|i| i as f32))
}

fn yaml_num(node: &Yaml, key: &str) -> f32 {
    yaml_get(node, key).and_then(yaml_num_val).unwrap_or(0.0)
}

fn yaml_num_or(node: &Yaml, key: &str, default: f32) -> f32 {
    yaml_get(node, key)
        .and_then(yaml_num_val)
        .unwrap_or(default)
}

fn yaml_num_opt(node: &Yaml, key: &str) -> Option<f32> {
    yaml_get(node, key).and_then(yaml_num_val)
}

fn yaml_vec<'a, 'b>(node: &'a Yaml<'b>, key: &str) -> Option<&'a Vec<Yaml<'b>>> {
    yaml_get(node, key).and_then(|v| v.as_vec())
}

fn yaml_int_or(node: &Yaml, key: &str, default: i64) -> i64 {
    yaml_get(node, key)
        .and_then(|v| v.as_integer())
        .unwrap_or(default)
}

fn yaml_bool_or(node: &Yaml, key: &str, default: bool) -> bool {
    yaml_get(node, key)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// Template metadata parsing (shared by both composed and file-based paths)
// ---------------------------------------------------------------------------

/// Parse color definitions from a template YAML node
fn parse_template_colors(node: &Yaml) -> HashMap<String, String> {
    let Some(colors_yaml) = yaml_get(node, "colors") else {
        return HashMap::new();
    };
    let Yaml::Mapping(m) = colors_yaml else {
        return HashMap::new();
    };
    m.iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
        .collect()
}

fn parse_width_constraints(node: &Yaml) -> TextWidthConstraints {
    let Some(widths) = yaml_get(node, "max_widths") else {
        return TextWidthConstraints::default();
    };
    TextWidthConstraints {
        title: yaml_num_opt(widths, "title"),
        description: yaml_num_opt(widths, "description"),
        subtitle: yaml_num_opt(widths, "subtitle"),
    }
}

// ---------------------------------------------------------------------------
// Gradient YAML parsing
// ---------------------------------------------------------------------------

fn parse_gradient_stop(node: &Yaml) -> Option<GradientStop> {
    Some(GradientStop {
        color: yaml_str(node, "color")?,
        offset: yaml_num(node, "offset"),
        opacity: yaml_get(node, "opacity").and_then(yaml_num_val),
    })
}

fn parse_blob_def(node: &Yaml) -> Option<BlobDef> {
    Some(BlobDef {
        name: yaml_str(node, "name")?,
        cx: yaml_num(node, "cx"),
        cy: yaml_num(node, "cy"),
        r: yaml_num(node, "r"),
        stops: yaml_vec(node, "stops")?
            .iter()
            .filter_map(parse_gradient_stop)
            .collect(),
    })
}

fn parse_noise_def(node: &Yaml) -> NoiseDef {
    let d = NoiseDef::default();
    NoiseDef {
        coarse_opacity: yaml_num_or(node, "coarse_opacity", d.coarse_opacity),
        fine_opacity: yaml_num_or(node, "fine_opacity", d.fine_opacity),
        coarse_frequency: yaml_num_or(node, "coarse_frequency", d.coarse_frequency),
        coarse_octaves: yaml_int_or(node, "coarse_octaves", d.coarse_octaves as i64) as u32,
        coarse_slope: yaml_num_or(node, "coarse_slope", d.coarse_slope),
        coarse_intercept: yaml_num_or(node, "coarse_intercept", d.coarse_intercept),
        fine_frequency: yaml_num_or(node, "fine_frequency", d.fine_frequency),
        fine_octaves: yaml_int_or(node, "fine_octaves", d.fine_octaves as i64) as u32,
        fine_slope: yaml_num_or(node, "fine_slope", d.fine_slope),
        fine_intercept: yaml_num_or(node, "fine_intercept", d.fine_intercept),
    }
}

fn parse_gradient_text_colors(node: &Yaml) -> GradientTextColors {
    let d = GradientTextColors::default();
    GradientTextColors {
        title: yaml_str(node, "title").unwrap_or(d.title),
        description: yaml_str(node, "description").unwrap_or(d.description),
        subtitle: yaml_str(node, "subtitle").unwrap_or(d.subtitle),
        subtitle_opacity: yaml_num_or(node, "subtitle_opacity", d.subtitle_opacity),
        desc_opacity: yaml_num_or(node, "desc_opacity", d.desc_opacity),
    }
}

fn parse_gradient_def(node: &Yaml) -> Option<GradientDef> {
    let base_stops: Vec<GradientStop> = yaml_vec(node, "base_stops")?
        .iter()
        .filter_map(parse_gradient_stop)
        .collect();
    if base_stops.is_empty() {
        return None;
    }

    let dir_vec = yaml_vec(node, "direction")?;
    if dir_vec.len() < 4 {
        return None;
    }

    let blobs: Vec<BlobDef> = yaml_vec(node, "blobs")?
        .iter()
        .filter_map(parse_blob_def)
        .collect();

    Some(GradientDef {
        base_stops,
        direction: [
            yaml_num_val(&dir_vec[0]).unwrap_or(0.0),
            yaml_num_val(&dir_vec[1]).unwrap_or(0.0),
            yaml_num_val(&dir_vec[2]).unwrap_or(100.0),
            yaml_num_val(&dir_vec[3]).unwrap_or(100.0),
        ],
        blobs,
        noise: yaml_get(node, "noise")
            .map(parse_noise_def)
            .unwrap_or_default(),
        text_colors: yaml_get(node, "text_colors")
            .map(parse_gradient_text_colors)
            .unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------------
// SVG fragment builder — generates defs and background layers from a gradient
// ---------------------------------------------------------------------------

/// Format a number: integers without decimal point, floats with
fn fmt_num(v: f32) -> String {
    if v == v.floor() && v.is_finite() {
        format!("{}", v as i32)
    } else {
        format!("{v}")
    }
}

fn write_stop(out: &mut String, stop: &GradientStop) {
    out.push_str("      <stop offset=\"");
    out.push_str(&fmt_num(stop.offset));
    out.push_str("%\" stop-color=\"");
    out.push_str(&stop.color);
    out.push('"');
    if let Some(opacity) = stop.opacity {
        out.push_str(" stop-opacity=\"");
        out.push_str(&format!("{opacity}"));
        out.push('"');
    }
    out.push_str("/>\n");
}

fn write_noise_filter(
    out: &mut String,
    id: &str,
    freq: f32,
    octaves: u32,
    slope: f32,
    intercept: f32,
) {
    out.push_str("    <filter id=\"");
    out.push_str(id);
    out.push_str("\" x=\"0%\" y=\"0%\" width=\"100%\" height=\"100%\">");
    out.push_str("<feTurbulence type=\"fractalNoise\" baseFrequency=\"");
    out.push_str(&format!("{freq}"));
    out.push_str("\" numOctaves=\"");
    out.push_str(&format!("{octaves}"));
    out.push_str("\"/>");
    out.push_str("<feColorMatrix type=\"saturate\" values=\"0\"/>");
    out.push_str("<feComponentTransfer><feFuncA type=\"linear\" slope=\"");
    out.push_str(&format!("{slope}"));
    out.push_str("\" intercept=\"");
    out.push_str(&format!("{intercept}"));
    out.push_str("\"/></feComponentTransfer>");
    out.push_str("</filter>\n");
}

fn build_gradient_defs(gradient: &GradientDef) -> String {
    let mut out = String::new();
    let d = &gradient.direction;

    // Base linear gradient
    out.push_str(&format!(
        "    <linearGradient id=\"baseGradient\" x1=\"{}%\" y1=\"{}%\" x2=\"{}%\" y2=\"{}%\">\n",
        fmt_num(d[0]),
        fmt_num(d[1]),
        fmt_num(d[2]),
        fmt_num(d[3])
    ));
    for stop in &gradient.base_stops {
        write_stop(&mut out, stop);
    }
    out.push_str("    </linearGradient>\n");

    // Radial gradient blobs
    for (i, blob) in gradient.blobs.iter().enumerate() {
        out.push_str(&format!(
            "    <radialGradient id=\"blob{}\" cx=\"{}%\" cy=\"{}%\" r=\"{}%\">\n",
            i + 1,
            fmt_num(blob.cx),
            fmt_num(blob.cy),
            fmt_num(blob.r)
        ));
        for stop in &blob.stops {
            write_stop(&mut out, stop);
        }
        out.push_str("    </radialGradient>\n");
    }

    // Noise filters
    let n = &gradient.noise;
    write_noise_filter(
        &mut out,
        "noiseCoarse",
        n.coarse_frequency,
        n.coarse_octaves,
        n.coarse_slope,
        n.coarse_intercept,
    );
    write_noise_filter(
        &mut out,
        "noiseFine",
        n.fine_frequency,
        n.fine_octaves,
        n.fine_slope,
        n.fine_intercept,
    );

    out
}

fn build_gradient_layers(gradient: &GradientDef) -> String {
    let mut out = String::new();

    out.push_str("  <rect width=\"100%\" height=\"100%\" fill=\"url(#baseGradient)\"/>\n");

    for i in 0..gradient.blobs.len() {
        out.push_str(&format!(
            "  <rect width=\"100%\" height=\"100%\" fill=\"url(#blob{})\"/>\n",
            i + 1
        ));
    }

    let n = &gradient.noise;
    out.push_str(&format!(
        "  <rect width=\"100%\" height=\"100%\" fill=\"#000\" filter=\"url(#noiseCoarse)\" opacity=\"{}\"/>\n",
        n.coarse_opacity
    ));
    out.push_str(&format!(
        "  <rect width=\"100%\" height=\"100%\" fill=\"#000\" filter=\"url(#noiseFine)\" opacity=\"{}\"/>\n",
        n.fine_opacity
    ));

    out
}

// ---------------------------------------------------------------------------
// Template composition — combines a layout SVG with a gradient definition
// ---------------------------------------------------------------------------

fn compose_template(layout_svg: &str, gradient: &GradientDef) -> String {
    let tc = &gradient.text_colors;
    layout_svg
        .replace(
            "<!-- ogis_gradient_defs -->",
            &build_gradient_defs(gradient),
        )
        .replace(
            "<!-- ogis_background_layers -->",
            &build_gradient_layers(gradient),
        )
        .replace("{{title_color}}", &tc.title)
        .replace("{{desc_color}}", &tc.description)
        .replace("{{subtitle_color}}", &tc.subtitle)
        .replace("{{desc_opacity}}", &fmt_num(tc.desc_opacity))
        .replace("{{subtitle_opacity}}", &fmt_num(tc.subtitle_opacity))
}

fn build_colors_map(gradient: &GradientDef) -> HashMap<String, String> {
    let mut colors = HashMap::new();
    if let Some(first) = gradient.base_stops.first() {
        colors.insert("background".to_string(), first.color.clone());
    }
    colors.insert("text".to_string(), gradient.text_colors.title.clone());
    for blob in &gradient.blobs {
        if let Some(first) = blob.stops.first() {
            colors.insert(format!("blob_{}", blob.name), first.color.clone());
        }
    }
    colors
}

// ---------------------------------------------------------------------------
// SVG font property extraction
// ---------------------------------------------------------------------------

/// Extract font properties from an SVG element by ID.
/// Font properties may be on a parent `<text>` element, not on the `<tspan>` with the ID.
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
                let mut temp_family = current_font.family.clone();
                let mut temp_size = current_font.size;
                let mut temp_weight = current_font.weight;
                let mut found_target_id = false;

                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let key = attr.key.as_ref();
                    if let Ok(value) = std::str::from_utf8(&attr.value) {
                        match key {
                            b"id" if value == element_id => found_target_id = true,
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

                current_font.family = temp_family;
                current_font.size = temp_size;
                current_font.weight = temp_weight;

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

fn parse_font_properties(svg_content: &str) -> TemplateFonts {
    TemplateFonts {
        title: extract_font_from_svg(svg_content, "ogis_title"),
        description: extract_font_from_svg(svg_content, "ogis_description"),
        subtitle: extract_font_from_svg(svg_content, "ogis_subtitle"),
    }
}

// ---------------------------------------------------------------------------
// Template loading — loads layouts, gradients, and template entries
// ---------------------------------------------------------------------------

fn load_layouts(doc: &Yaml) -> HashMap<String, String> {
    let mut layouts = HashMap::new();
    let Some(Yaml::Mapping(m)) = yaml_get(doc, "layouts") else {
        return layouts;
    };
    for (key, value) in m.iter() {
        let (Some(name), Some(path)) = (key.as_str(), value.as_str()) else {
            continue;
        };
        match yaml_loader::load_text(path, &format!("layout '{name}'")) {
            Ok(content) => {
                tracing::info!("Loaded layout '{name}' from {path}");
                layouts.insert(name.to_string(), content);
            }
            Err(e) => tracing::error!("Failed to load layout '{name}': {e}"),
        }
    }
    layouts
}

fn load_gradients() -> HashMap<String, GradientDef> {
    let mut gradients = HashMap::new();
    let dir = match std::fs::read_dir("gradients") {
        Ok(dir) => dir,
        Err(e) => {
            tracing::warn!("No gradients directory found: {e}");
            return gradients;
        }
    };
    for entry in dir.flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            let doc = yaml_loader::load_yaml(path.to_str().unwrap_or_default());
            match parse_gradient_def(&doc) {
                Some(def) => {
                    tracing::info!(
                        "Parsed gradient '{name}': {} base stops, {} blobs",
                        def.base_stops.len(),
                        def.blobs.len()
                    );
                    gradients.insert(name, def);
                }
                None => tracing::error!("Failed to parse gradient from {}", path.display()),
            }
        }
    }
    gradients
}

/// Load a file-based template, returning a TemplateEntry if successful.
fn load_file_template(node: &Yaml) -> Option<TemplateEntry> {
    let name = yaml_str(node, "name")?;
    let path = yaml_str(node, "file")?;
    let svg = yaml_loader::load_text(&path, &format!("template '{name}'")).ok()?;

    let fonts = parse_font_properties(&svg);
    tracing::info!(
        "Loaded '{name}' from {path}: title={}px/w{}, desc={}px/w{}, subtitle={}px/w{}",
        fonts.title.size,
        fonts.title.weight,
        fonts.description.size,
        fonts.description.weight,
        fonts.subtitle.size,
        fonts.subtitle.weight,
    );

    let colors = parse_template_colors(node);

    Some(TemplateEntry {
        name,
        svg,
        colors,
        fonts,
        width_constraints: parse_width_constraints(node),
        truncation: yaml_bool_or(node, "truncation", true),
        max_scale: yaml_num_or(node, "max_scale", 1.0),
    })
}

/// Build a composed template from a layout SVG + gradient definition.
fn build_composed_template(
    template_name: &str,
    layout_svg: &str,
    gradient_key: &str,
    gradient_def: &GradientDef,
) -> TemplateEntry {
    let svg = compose_template(layout_svg, gradient_def);
    let fonts = parse_font_properties(&svg);
    tracing::info!(
        "Composed '{template_name}' (gradient={gradient_key}): title={}px/w{}",
        fonts.title.size,
        fonts.title.weight,
    );

    TemplateEntry {
        name: template_name.to_string(),
        svg,
        colors: build_colors_map(gradient_def),
        fonts,
        width_constraints: TextWidthConstraints::default(),
        truncation: true,
        max_scale: 1.0,
    }
}

fn register_entry(
    entry: TemplateEntry,
    templates: &mut HashMap<String, String>,
    colors: &mut HashMap<String, HashMap<String, String>>,
    width_constraints: &mut HashMap<String, TextWidthConstraints>,
    font_properties: &mut HashMap<String, TemplateFonts>,
    truncation: &mut HashMap<String, bool>,
    max_scale: &mut HashMap<String, f32>,
) {
    if !entry.colors.is_empty() {
        colors.insert(entry.name.clone(), entry.colors);
    }
    width_constraints.insert(entry.name.clone(), entry.width_constraints);
    font_properties.insert(entry.name.clone(), entry.fonts);
    truncation.insert(entry.name.clone(), entry.truncation);
    max_scale.insert(entry.name.clone(), entry.max_scale);
    templates.insert(entry.name, entry.svg);
}

pub fn load_templates() -> TemplateMap {
    let mut templates = HashMap::new();
    let mut colors = HashMap::new();
    let mut width_constraints = HashMap::new();
    let mut font_properties = HashMap::new();
    let mut truncation = HashMap::new();
    let mut max_scale = HashMap::new();

    let doc = yaml_loader::load_yaml("templates.yaml");

    let layouts = load_layouts(&doc);
    let gradients = load_gradients();

    if !layouts.is_empty() {
        tracing::info!(
            "Loaded {} layout(s) and {} gradient(s)",
            layouts.len(),
            gradients.len()
        );
    }

    // Auto-generate all layout × gradient combinations
    let mut gradient_names: Vec<&String> = gradients.keys().collect();
    gradient_names.sort();
    let mut layout_names: Vec<&String> = layouts.keys().collect();
    layout_names.sort();
    for gradient_name in &gradient_names {
        for layout_name in &layout_names {
            let template_name = format!("gradient-{gradient_name}-{layout_name}");
            let layout_svg = &layouts[layout_name.as_str()];
            let gradient_def = &gradients[gradient_name.as_str()];
            let entry =
                build_composed_template(&template_name, layout_svg, gradient_name, gradient_def);
            register_entry(
                entry,
                &mut templates,
                &mut colors,
                &mut width_constraints,
                &mut font_properties,
                &mut truncation,
                &mut max_scale,
            );
        }
    }

    // Load file-based templates from templates.yaml
    if let Some(template_list) = yaml_vec(&doc, "templates") {
        for node in template_list {
            if let Some(entry) = load_file_template(node) {
                register_entry(
                    entry,
                    &mut templates,
                    &mut colors,
                    &mut width_constraints,
                    &mut font_properties,
                    &mut truncation,
                    &mut max_scale,
                );
            }
        }
    }

    if templates.is_empty() {
        panic!("No templates were loaded from templates.yaml");
    }

    let default = yaml_str(&doc, "default").expect("Missing 'default' field in templates.yaml");

    let template_map = TemplateMap {
        templates,
        default,
        colors,
        width_constraints,
        font_properties,
        truncation,
        max_scale,
    };

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

#[cfg(test)]
mod tests {
    use super::*;
    use saphyr::{LoadableYamlNode, Yaml};

    fn parse_yaml(s: &str) -> Yaml<'static> {
        Yaml::load_from_str(s).unwrap().into_iter().next().unwrap()
    }

    fn test_gradient(text_colors: GradientTextColors) -> GradientDef {
        GradientDef {
            base_stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: "#111111".to_string(),
                    opacity: None,
                },
                GradientStop {
                    offset: 100.0,
                    color: "#222222".to_string(),
                    opacity: None,
                },
            ],
            direction: [0.0, 0.0, 100.0, 100.0],
            blobs: vec![BlobDef {
                name: "violet".to_string(),
                cx: 30.0,
                cy: 70.0,
                r: 60.0,
                stops: vec![
                    GradientStop {
                        offset: 0.0,
                        color: "#7c3aed".to_string(),
                        opacity: Some(0.8),
                    },
                    GradientStop {
                        offset: 100.0,
                        color: "#7c3aed".to_string(),
                        opacity: Some(0.0),
                    },
                ],
            }],
            noise: NoiseDef::default(),
            text_colors,
        }
    }

    #[test]
    fn test_parse_truncation_defaults_to_true() {
        assert!(yaml_bool_or(&parse_yaml("name: test"), "truncation", true));
    }

    #[test]
    fn test_parse_truncation_false() {
        assert!(!yaml_bool_or(
            &parse_yaml("truncation: false"),
            "truncation",
            true
        ));
    }

    #[test]
    fn test_parse_max_scale_defaults_to_one() {
        assert_eq!(
            yaml_num_or(&parse_yaml("name: test"), "max_scale", 1.0),
            1.0
        );
    }

    #[test]
    fn test_parse_max_scale_custom() {
        assert_eq!(
            yaml_num_or(&parse_yaml("max_scale: 2.0"), "max_scale", 1.0),
            2.0
        );
    }

    #[test]
    fn test_build_gradient_defs_basic() {
        let defs = build_gradient_defs(&test_gradient(GradientTextColors::default()));
        assert!(defs.contains("linearGradient"));
        assert!(defs.contains("#111111"));
        assert!(defs.contains("blob1"));
        assert!(defs.contains("#7c3aed"));
        assert!(defs.contains("noiseCoarse"));
        assert!(defs.contains("noiseFine"));
    }

    #[test]
    fn test_compose_template_replaces_tokens() {
        let layout = r#"<svg>
  <defs>
<!-- ogis_gradient_defs -->
  </defs>
<!-- ogis_background_layers -->
  <text fill="{{title_color}}">title</text>
  <text fill="{{desc_color}}" opacity="{{desc_opacity}}">desc</text>
  <text fill="{{subtitle_color}}" opacity="{{subtitle_opacity}}">sub</text>
</svg>"#;

        let gradient = test_gradient(GradientTextColors {
            title: "#ff0000".to_string(),
            description: "#00ff00".to_string(),
            subtitle: "#0000ff".to_string(),
            subtitle_opacity: 0.7,
            desc_opacity: 0.9,
        });

        let result = compose_template(layout, &gradient);
        assert!(result.contains("fill=\"#ff0000\""));
        assert!(result.contains("fill=\"#00ff00\""));
        assert!(result.contains("fill=\"#0000ff\""));
        assert!(result.contains("opacity=\"0.9\""));
        assert!(result.contains("opacity=\"0.7\""));
        assert!(result.contains("baseGradient"));
        assert!(!result.contains("{{title_color}}"));
        assert!(!result.contains("<!-- ogis_gradient_defs -->"));
    }

    #[test]
    fn test_build_colors_map() {
        let colors = build_colors_map(&test_gradient(GradientTextColors::default()));
        assert_eq!(colors.get("background").unwrap(), "#111111");
        assert_eq!(colors.get("text").unwrap(), "#ffffff");
        assert_eq!(colors.get("blob_violet").unwrap(), "#7c3aed");
    }

    #[test]
    fn test_build_colors_map_empty_stops() {
        let gradient = GradientDef {
            base_stops: vec![GradientStop {
                offset: 0.0,
                color: "#000".to_string(),
                opacity: None,
            }],
            direction: [0.0; 4],
            blobs: vec![BlobDef {
                name: "empty".to_string(),
                cx: 0.0,
                cy: 0.0,
                r: 0.0,
                stops: vec![],
            }],
            noise: NoiseDef::default(),
            text_colors: GradientTextColors::default(),
        };
        let colors = build_colors_map(&gradient);
        assert_eq!(colors.get("background").unwrap(), "#000");
        assert!(!colors.contains_key("blob_empty"));
    }
}
