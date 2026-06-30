use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::AppState;
use crate::{error::ApiError, wire};

/// Decompressed-size cap derived from per-field limits (spec §8.1).
pub(crate) fn max_decoded_len(max_input_length: usize) -> usize {
    // Width of a length-varint for a field of up to `max_input_length` bytes
    // (2 bytes at the default 1000, but 3+ once the limit exceeds 16383).
    let mut vw = 1;
    let mut n = max_input_length;
    while n >= 128 {
        n >>= 7;
        vw += 1;
    }
    // Worst-case schema-pack body. Data: 6 max-length fields (template + 3 text +
    // 2 URL) and MAX_EXTRA extra entries (key+value). Framing: presence(2) +
    // format/scale/quality(4) + 6 field length-varints + 2 URL scheme tags(2) +
    // extra count(1) + MAX_EXTRA*(2 length-varints). Colors fold into the extra block.
    let data = max_input_length * (6 + 2 * wire::MAX_EXTRA);
    let framing = 2 + 4 + 6 * vw + 2 + 1 + wire::MAX_EXTRA * 2 * vw;
    data + framing
}

async fn handle(state: AppState, blob: String, sig: Option<String>) -> Response {
    let secret: Option<&[u8]> = state.hmac_validator.as_ref().map(|v| v.secret());
    let max_decoded = max_decoded_len(state.max_input_length);

    let params = match wire::decode(
        &blob,
        sig.as_deref(),
        secret,
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
        ("sig" = String, Path, description = "base64url-nopad of the leading 6 bytes of HMAC-SHA256 over [version byte ++ uncompressed body] (8 characters)")
    ),
    responses(
        (status = 200, description = "Successfully generated PNG image (1200x630)", content_type = "image/png"),
        (status = 400, description = "Invalid compressed URL - malformed blob or unsupported version"),
        (status = 401, description = "Authentication required - missing or invalid signature"),
        (status = 403, description = "Forbidden - SSRF blocked (private IP)"),
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

    fn assert_admits_maximal(max: usize) {
        // A request with every field at max_input_length and MAX_EXTRA full-length
        // extra entries must pack within the decode cap, so a maximal legal /c/ URL
        // is never falsely 400'd at decompression.
        let big = "x".repeat(max);
        let mut p = crate::params::OgParams {
            title: Some(big.clone()),
            description: Some(big.clone()),
            subtitle: Some(big.clone()),
            logo: Some(big.clone()),
            image: Some(big.clone()),
            template: Some(big.clone()),
            signature: None,
            format: Some("webp".into()),
            scale: Some(2.0),
            quality: Some(50),
            extra: std::collections::HashMap::new(),
        };
        for i in 0..crate::wire::MAX_EXTRA {
            p.extra
                .insert(format!("{i:04}{}", "k".repeat(max - 4)), big.clone());
        }
        let body = crate::wire::body::pack_body(&p).unwrap();
        assert!(
            body.len() <= max_decoded_len(max),
            "maximal body {} exceeds decode cap {} (max_input_length={max})",
            body.len(),
            max_decoded_len(max),
        );
    }

    #[test]
    fn admits_a_maximal_legal_body() {
        assert_admits_maximal(1000); // 2-byte length varints
        assert_admits_maximal(20_000); // 3-byte length varints (> 16383) — was under-provisioned
        assert!(max_decoded_len(1000) > max_decoded_len(100)); // still monotonic
    }
}
