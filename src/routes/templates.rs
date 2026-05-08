use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct TemplatesResponse {
    /// Name of the template returned when the request supplies no `template` param.
    pub default: String,
    /// All template names registered at startup (file-based + auto-composed gradients).
    pub templates: Vec<String>,
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
    let mut templates: Vec<String> = state.templates.templates.keys().cloned().collect();
    templates.sort();
    Json(TemplatesResponse {
        default: state.templates.default.clone(),
        templates,
    })
}
