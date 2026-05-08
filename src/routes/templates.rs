use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::templates::TemplateMap;

#[derive(Serialize, ToSchema)]
pub struct TemplatesResponse {
    /// Name of the template returned when the request supplies no `template` param.
    pub default: String,
    /// All template names registered at startup (file-based + auto-composed gradients).
    pub templates: Vec<String>,
}

/// Pure builder so unit tests don't need a full `AppState`.
fn build_response(templates: &TemplateMap) -> TemplatesResponse {
    let mut names: Vec<String> = templates.templates.keys().cloned().collect();
    names.sort();
    TemplatesResponse {
        default: templates.default.clone(),
        templates: names,
    }
}

/// List all registered template names.
///
/// Useful for benchmarking and tooling that needs to enumerate every template
/// the running server can generate without coupling to the on-disk layout.
#[utoipa::path(
    get,
    path = "/templates",
    responses(
        (status = 200, description = "List of template names", body = TemplatesResponse)
    ),
    tag = "monitoring"
)]
pub async fn list_templates(State(state): State<AppState>) -> Json<TemplatesResponse> {
    Json(build_response(&state.templates))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::TemplateMap;
    use std::collections::HashMap;

    fn empty_map() -> TemplateMap {
        TemplateMap {
            templates: HashMap::new(),
            default: "default".to_string(),
            colors: HashMap::new(),
            width_constraints: HashMap::new(),
            font_properties: HashMap::new(),
            truncation: HashMap::new(),
            max_scale: HashMap::new(),
            gradient_splits: HashMap::new(),
        }
    }

    #[test]
    fn returns_default_and_sorted_template_names() {
        let mut map = empty_map();
        map.default = "minimal".to_string();
        map.templates
            .insert("twilight".to_string(), "<svg/>".to_string());
        map.templates
            .insert("minimal".to_string(), "<svg/>".to_string());
        map.templates
            .insert("gradient-aurora-centered".to_string(), "<svg/>".to_string());

        let resp = build_response(&map);

        assert_eq!(resp.default, "minimal");
        assert_eq!(
            resp.templates,
            vec![
                "gradient-aurora-centered".to_string(),
                "minimal".to_string(),
                "twilight".to_string(),
            ]
        );
    }

    #[test]
    fn empty_template_map_returns_empty_list() {
        let resp = build_response(&empty_map());
        assert_eq!(resp.default, "default");
        assert!(resp.templates.is_empty());
    }

    #[test]
    fn response_serialises_with_expected_field_names() {
        let mut map = empty_map();
        map.default = "minimal".to_string();
        map.templates
            .insert("foo".to_string(), "<svg/>".to_string());

        let json = serde_json::to_value(build_response(&map)).unwrap();
        assert_eq!(json["default"], "minimal");
        assert_eq!(json["templates"], serde_json::json!(["foo"]));
    }
}
