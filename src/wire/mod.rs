//! Compressed-URL (`/c/<blob>`) wire codec. See design/compressed-urls.md.
pub mod body;
pub mod container;
pub mod registry;
pub mod varint;

use crate::params::OgParams;
use crate::templates::TemplateMap;
use registry::Registry;

pub const FORMAT_VERSION: u8 = 1;
pub const MAX_COLORS: usize = 32;
pub const MAX_EXTRA: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum WireError {
    #[error("blob truncated")]
    Truncated,
    #[error("trailing bytes after body")]
    TrailingBytes,
    #[error("malformed varint")]
    Malformed,
    #[error("invalid base64url")]
    BadBase64,
    #[error("unsupported format version {0}")]
    BadVersion(u8),
    #[error("unsupported compression mode {0}")]
    BadMode(u8),
    #[error("reserved presence bit set")]
    ReservedBit,
    #[error("invalid format code")]
    BadFormat,
    #[error("invalid scheme tag")]
    BadSchemeTag,
    #[error("invalid UTF-8")]
    BadUtf8,
    #[error("field exceeds maximum length")]
    FieldTooLong,
    #[error("too many entries")]
    TooManyEntries,
    #[error("unknown template id")]
    UnknownTemplate,
    #[error("unknown color id")]
    UnknownColor,
    #[error("decompressed body too large")]
    TooLarge,
    #[error("brotli error")]
    BadBrotli,
    #[error("unauthorized")]
    Unauthorized,
}

/// Encode `params` to `(blob, optional sig)`. `secret` present ⇒ sign.
pub fn encode(
    p: &OgParams,
    reg: &Registry,
    templates: &TemplateMap,
    secret: Option<&[u8]>,
) -> Result<(String, Option<String>), WireError> {
    let body = body::pack_body(p, reg, templates)?;
    let sig = secret.map(|s| crate::auth::blob::sign(s, FORMAT_VERSION, &body));
    Ok((container::encode_container(&body), sig))
}

/// Decode a `/c/` blob (+ optional sig) into `OgParams`.
pub fn decode(
    blob: &str,
    sig: Option<&str>,
    reg: &Registry,
    secret: Option<&[u8]>,
    max_field_len: usize,
    max_decoded: usize,
) -> Result<OgParams, WireError> {
    let (version, body) = container::decode_container(blob, max_decoded)?;
    match (secret, sig) {
        (Some(s), Some(seg)) => crate::auth::blob::verify(s, version, &body, seg)?,
        (Some(_), None) => return Err(WireError::Unauthorized),
        (None, _) => {} // auth disabled
    }
    body::unpack_body(&body, reg, max_field_len)
}
