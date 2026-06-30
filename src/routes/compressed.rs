use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::AppState;
use crate::{error::ApiError, wire};

/// Decompressed-size cap derived from per-field limits (spec §8.1).
pub(crate) fn max_decoded_len(max_input_length: usize) -> usize {
    // 6 max-length fields (template name + 5 text/URL) + MAX_EXTRA*(key+value),
    // plus header/overhead. Colors fold into the extra block (no separate term).
    max_input_length * (6 + 2 * wire::MAX_EXTRA) + 64
}

async fn handle(state: AppState, blob: String, sig: Option<String>) -> Response {
    let secret: Option<Vec<u8>> = state.hmac_validator.as_ref().map(|v| v.secret().to_vec());
    let max_decoded = max_decoded_len(state.max_input_length);

    let params = match wire::decode(
        &blob,
        sig.as_deref(),
        secret.as_deref(),
        state.max_input_length,
        max_decoded,
    ) {
        Ok(p) => p,
        Err(e) => return ApiError::from(e).into_response(),
    };

    // Unknown/literal template name → fall back to the runtime default.
    let params = drop_dead_template(params, &state);

    crate::routes::render::render_response(state, params).await
}

/// If the decoded literal template name isn't a live template (discontinued, or a
/// template that only exists in another deployment/tenant), clear it so the runtime
/// default renders. Published URLs therefore degrade gracefully instead of 404ing.
fn drop_dead_template(
    mut params: crate::params::OgParams,
    state: &AppState,
) -> crate::params::OgParams {
    if let Some(name) = &params.template
        && !state.templates.templates.contains_key(name)
    {
        params.template = None;
    }
    params
}

#[utoipa::path(
    get,
    path = "/c/{blob}",
    params(
        ("blob" = String, Path, description = "Base64url-encoded compressed parameter blob, produced by an ogis SDK")
    ),
    responses(
        (status = 200, description = "Successfully generated PNG image (1200x630)", content_type = "image/png"),
        (status = 400, description = "Invalid compressed URL - malformed blob or unsupported version"),
        (status = 401, description = "Authentication required - missing or invalid signature"),
        (status = 403, description = "Forbidden - SSRF blocked (private IP)"),
        (status = 404, description = "Template not found"),
        (status = 422, description = "Unprocessable - invalid image URL, unsupported format, or image too large"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "Bad gateway - upstream image fetch failed"),
        (status = 503, description = "Service unavailable - server overloaded"),
        (status = 504, description = "Gateway timeout - image fetch timed out")
    ),
    tag = "image"
)]
pub async fn generate_compressed(
    State(state): State<AppState>,
    Path(blob): Path<String>,
) -> Response {
    handle(state, blob, None).await
}

#[utoipa::path(
    get,
    path = "/c/{blob}/{sig}",
    params(
        ("blob" = String, Path, description = "Base64url-encoded compressed parameter blob, produced by an ogis SDK"),
        ("sig" = String, Path, description = "HMAC-SHA256 signature (first 8 hex characters) authorising this blob")
    ),
    responses(
        (status = 200, description = "Successfully generated PNG image (1200x630)", content_type = "image/png"),
        (status = 400, description = "Invalid compressed URL - malformed blob or unsupported version"),
        (status = 401, description = "Authentication required - missing or invalid signature"),
        (status = 403, description = "Forbidden - SSRF blocked (private IP)"),
        (status = 404, description = "Template not found"),
        (status = 422, description = "Unprocessable - invalid image URL, unsupported format, or image too large"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "Bad gateway - upstream image fetch failed"),
        (status = 503, description = "Service unavailable - server overloaded"),
        (status = 504, description = "Gateway timeout - image fetch timed out")
    ),
    tag = "image"
)]
pub async fn generate_compressed_signed(
    State(state): State<AppState>,
    Path((blob, sig)): Path<(String, String)>,
) -> Response {
    handle(state, blob, Some(sig)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_decoded_grows_and_admits_full_payload() {
        // Monotonic in input length, and large enough for the 5 full-length
        // text/URL fields (so a legitimate max-size request is never falsely capped).
        assert!(max_decoded_len(1000) > max_decoded_len(100));
        assert!(max_decoded_len(1000) >= 5 * 1000);
    }
}
