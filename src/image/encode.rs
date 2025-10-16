use base64::Engine;

/// Encode raw bytes to base64 string
pub fn encode_base64_bytes(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
