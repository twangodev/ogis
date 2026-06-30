use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::wire::WireError;

type HmacSha256 = Hmac<Sha256>;

/// Truncated tag length in bytes (→ 8 base64url chars). Pinned by the format version.
pub const SIG_LEN: usize = 6;

fn mac(secret: &[u8], version: u8, body: &[u8]) -> HmacSha256 {
    let mut m = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
    m.update(&[version]);
    m.update(body);
    m
}

/// Sign `[version] ++ body`, returning the 8-char base64url segment.
#[allow(dead_code)] // encoder reference; the server only verifies
pub fn sign(secret: &[u8], version: u8, body: &[u8]) -> String {
    let tag = mac(secret, version, body).finalize().into_bytes();
    URL_SAFE_NO_PAD.encode(&tag[..SIG_LEN])
}

/// Verify a `/c/` path-segment signature in constant time.
pub fn verify(secret: &[u8], version: u8, body: &[u8], seg: &str) -> Result<(), WireError> {
    let provided = URL_SAFE_NO_PAD
        .decode(seg.as_bytes())
        .map_err(|_| WireError::InvalidSignature)?;
    if provided.len() != SIG_LEN {
        return Err(WireError::InvalidSignature);
    }
    mac(secret, version, body)
        .verify_truncated_left(&provided)
        .map_err(|_| WireError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_ok() {
        let body = b"hello body";
        let seg = sign(b"secret", 1, body);
        assert_eq!(seg.len(), 8); // 6 bytes → 8 base64url chars
        assert!(verify(b"secret", 1, body, &seg).is_ok());
    }

    #[test]
    fn wrong_secret_or_body_rejected() {
        let seg = sign(b"secret", 1, b"body");
        assert!(verify(b"other", 1, b"body", &seg).is_err());
        assert!(verify(b"secret", 1, b"BODY", &seg).is_err());
        assert!(verify(b"secret", 2, b"body", &seg).is_err()); // version is signed
    }

    #[test]
    fn malformed_segment_rejected() {
        assert!(matches!(
            verify(b"s", 1, b"b", "!!!"),
            Err(WireError::InvalidSignature)
        ));
        assert!(matches!(
            verify(b"s", 1, b"b", "AAAA"),
            Err(WireError::InvalidSignature)
        )); // 3 bytes ≠ 6
    }
}
