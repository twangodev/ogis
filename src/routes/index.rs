use crate::{AppState, params::OgParams};
use axum::extract::{Query, State};
use axum::response::IntoResponse;

#[utoipa::path(
    get,
    path = "/",
    params(OgParams),
    responses(
        (status = 200, description = "Successfully generated PNG image (1200x630)", content_type = "image/png"),
        (status = 400, description = "Invalid input - field exceeds maximum length or invalid hex color"),
        (status = 401, description = "Authentication required - missing signature"),
        (status = 403, description = "Forbidden - invalid signature or SSRF blocked"),
        (status = 404, description = "Template not found"),
        (status = 422, description = "Unprocessable - invalid image URL, unsupported format, or image too large"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "Bad gateway - upstream image fetch failed"),
        (status = 503, description = "Service unavailable - server overloaded"),
        (status = 504, description = "Gateway timeout - image fetch timed out")
    ),
    tag = "image"
)]
pub async fn generate(
    State(state): State<AppState>,
    Query(params): Query<OgParams>,
) -> impl IntoResponse {
    crate::routes::render::render_response(state, params).await
}
