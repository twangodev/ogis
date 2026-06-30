use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use std::io::Read;

use super::{FORMAT_VERSION, MAX_ENCODED_LEN, WireError};

const MODE_RAW: u8 = 0;
const MODE_BROTLI: u8 = 1;
/// Brotli window (log2) the encoder pins and the decoder accepts. 18 = 256 KiB,
/// ample for our <=~134 KB bodies; a larger *declared* window only inflates the
/// decoder's ring-buffer allocation (up to 16 MiB at lgwin 24), so it is rejected
/// before decompression. Pinned pre-launch; SDKs MUST encode with window <= this.
const MAX_WINDOW_BITS: u8 = 18;

fn container_bytes(mode: u8, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(payload.len() + 1);
    v.push((FORMAT_VERSION << 4) | mode);
    v.extend_from_slice(payload);
    v
}

fn brotli_compress(input: &[u8]) -> Vec<u8> {
    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        lgwin: MAX_WINDOW_BITS as i32,
        ..Default::default()
    };
    let mut out = Vec::new();
    let mut reader = input;
    brotli::BrotliCompress(&mut reader, &mut out, &params)
        .expect("brotli compress is infallible into a Vec");
    out
}

/// Encode `body` as the smaller of {raw, brotli} containers, base64url-nopad.
pub fn encode_container(body: &[u8]) -> String {
    let raw = URL_SAFE_NO_PAD.encode(container_bytes(MODE_RAW, body));
    let bro = URL_SAFE_NO_PAD.encode(container_bytes(MODE_BROTLI, &brotli_compress(body)));
    if bro.len() < raw.len() { bro } else { raw }
}

/// Decode a blob to `(version, uncompressed body)`, bounding the decompressed size.
pub fn decode_container(blob: &str, max_decoded: usize) -> Result<(u8, Vec<u8>), WireError> {
    if blob.len() > MAX_ENCODED_LEN {
        return Err(WireError::TooLarge);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(blob.as_bytes())
        .map_err(|_| WireError::BadBase64)?;
    let (&header, payload) = bytes.split_first().ok_or(WireError::Truncated)?;
    let version = header >> 4;
    let mode = header & 0x0f;
    if version != FORMAT_VERSION {
        return Err(WireError::BadVersion(version));
    }
    let body = match mode {
        MODE_RAW => {
            if payload.len() > max_decoded {
                return Err(WireError::TooLarge);
            }
            payload.to_vec()
        }
        MODE_BROTLI => {
            // Reject an oversized declared window before the decoder allocates its
            // ring buffer (a tiny blob can otherwise force up to a 16 MiB alloc).
            if brotli_window_bits(payload) > MAX_WINDOW_BITS {
                return Err(WireError::TooLarge);
            }
            decompress_bounded(payload, max_decoded)?
        }
        other => return Err(WireError::BadMode(other)),
    };
    Ok((version, body))
}

/// Read the brotli WBITS (window size, log2) from the stream's first byte
/// (RFC 7932 §9.1, non-large-window form). Used to reject an oversized declared
/// window before the decoder allocates its ring buffer.
fn brotli_window_bits(payload: &[u8]) -> u8 {
    let b = payload.first().copied().unwrap_or(0);
    let bit = |i: u8| (b >> i) & 1;
    if bit(0) == 0 {
        return 16;
    }
    let n = bit(1) | (bit(2) << 1) | (bit(3) << 2);
    if n != 0 {
        return 17 + n; // 18..=24
    }
    let m = bit(4) | (bit(5) << 1) | (bit(6) << 2);
    if m != 0 {
        return 8 + m; // 9..=15
    }
    17
}

fn decompress_bounded(input: &[u8], max: usize) -> Result<Vec<u8>, WireError> {
    let mut decompressor = brotli::Decompressor::new(input, 4096);
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = decompressor
            .read(&mut buf)
            .map_err(|_| WireError::BadBrotli)?;
        if n == 0 {
            break;
        }
        if out.len() + n > max {
            return Err(WireError::TooLarge);
        }
        out.extend_from_slice(&buf[..n]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_small_and_large_bodies() {
        for body in [
            vec![],
            b"hi".to_vec(),
            vec![7u8; 500],
            (0..255).collect::<Vec<u8>>(),
        ] {
            let blob = encode_container(&body);
            let (version, out) = decode_container(&blob, 100_000).unwrap();
            assert_eq!(version, FORMAT_VERSION);
            assert_eq!(out, body);
        }
    }

    #[test]
    fn picks_smaller_encoding() {
        // Highly compressible → brotli should win and decode back.
        let body = vec![0u8; 2000];
        let blob = encode_container(&body);
        assert!(blob.len() < 2000 * 4 / 3);
        assert_eq!(decode_container(&blob, 100_000).unwrap().1, body);
    }

    #[test]
    fn rejects_oversized_decompressed_body() {
        let body = vec![0u8; 5000]; // compresses tiny, inflates past the cap
        let blob = encode_container(&body);
        assert!(matches!(
            decode_container(&blob, 1000),
            Err(WireError::TooLarge)
        ));
    }

    #[test]
    fn rejects_oversized_brotli_window() {
        // A brotli payload whose first byte declares a 24-bit (16 MiB) window must be
        // rejected by the window check BEFORE decompression allocates the ring buffer.
        // 0x0F => WBITS 24 (first bit 1, next 3 bits = 0b111 = 7 => 17+7).
        let blob = URL_SAFE_NO_PAD.encode(container_bytes(MODE_BROTLI, &[0x0F, 0x00, 0x00]));
        assert!(matches!(
            decode_container(&blob, 100_000),
            Err(WireError::TooLarge)
        ));
    }

    #[test]
    fn rejects_bad_version_and_mode() {
        // header 0x2X = version 2 (unsupported)
        let bytes = [0x20u8, 0, 0];
        let blob = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes);
        assert!(matches!(
            decode_container(&blob, 1000),
            Err(WireError::BadVersion(2))
        ));
        // header 0x12 = version 1, mode 2 (unsupported mode)
        let bytes2 = [0x12u8, 0, 0];
        let blob2 =
            base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes2);
        assert!(matches!(
            decode_container(&blob2, 1000),
            Err(WireError::BadMode(2))
        ));
    }

    #[test]
    fn rejects_blob_over_length_cap() {
        let blob = "A".repeat(MAX_ENCODED_LEN + 1);
        assert!(matches!(
            decode_container(&blob, 100_000),
            Err(WireError::TooLarge)
        ));
    }
}
