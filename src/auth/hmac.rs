use axum::http::StatusCode;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum HmacError {
    #[error("Authentication required: missing signature parameter")]
    MissingSignature,

    #[error("Authentication failed: invalid signature")]
    InvalidSignature(#[from] hmac::digest::MacError),

    #[error("Invalid signature format: must be hex-encoded")]
    InvalidHexFormat(#[from] hex::FromHexError),
}

impl HmacError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingSignature | Self::InvalidSignature(_) => StatusCode::UNAUTHORIZED,
            Self::InvalidHexFormat(_) => StatusCode::BAD_REQUEST,
        }
    }
}

pub struct HmacValidator {
    secret: Vec<u8>,
}

impl HmacValidator {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    /// Validate HMAC signature for query parameters
    ///
    /// # Algorithm
    /// 1. Extract signature parameter from query params
    /// 2. Build canonical query string from remaining params (sorted by key)
    /// 3. Compute HMAC-SHA256(secret, canonical_query_string)
    /// 4. Compare with provided signature (constant-time)
    ///
    /// # Example
    /// Query: ?title=Hello&description=World&signature=abc123
    /// Canonical: description=World&title=Hello
    /// Signature: HMAC-SHA256(secret, "description=World&title=Hello")
    pub fn validate(&self, query_string: &str) -> Result<(), HmacError> {
        // Parse query parameters
        let params: BTreeMap<String, String> = url::form_urlencoded::parse(query_string.as_bytes())
            .into_owned()
            .collect();

        let provided_signature = params.get("signature").ok_or(HmacError::MissingSignature)?;
        let provided_bytes = hex::decode(provided_signature)?;
        let canonical = self.build_canonical_query(&params);

        let mut mac =
            HmacSha256::new_from_slice(&self.secret).expect("HMAC can take key of any size");
        mac.update(canonical.as_bytes());

        mac.verify_slice(&provided_bytes)?;
        Ok(())
    }

    /// Build canonical query string from params (sorted, excluding signature)
    /// Parameters are properly URL-encoded to prevent injection attacks
    fn build_canonical_query(&self, params: &BTreeMap<String, String>) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());

        // BTreeMap already keeps keys sorted alphabetically
        for (k, v) in params.iter() {
            if k != "signature" {
                serializer.append_pair(k, v);
            }
        }

        serializer.finish()
    }

    /// Raw secret bytes (used by the /c/ blob verifier).
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }

    /// Generate signature for testing/client examples
    #[cfg(test)]
    pub fn sign(&self, query_string: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(&self.secret).expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_validation_success() {
        let validator = HmacValidator::new(b"test-secret".to_vec());

        // Build canonical query
        let canonical = "description=World&title=Hello";
        let signature = validator.sign(canonical);

        // Validate full query
        let query = format!("title=Hello&description=World&signature={}", signature);
        assert!(validator.validate(&query).is_ok());
    }

    #[test]
    fn test_hmac_validation_invalid_signature() {
        let validator = HmacValidator::new(b"test-secret".to_vec());
        let query = "title=Hello&signature=invalid";
        assert!(matches!(
            validator.validate(query),
            Err(HmacError::InvalidHexFormat(_))
        ));
    }

    #[test]
    fn test_hmac_validation_missing_signature() {
        let validator = HmacValidator::new(b"test-secret".to_vec());
        let query = "title=Hello";
        assert!(matches!(
            validator.validate(query),
            Err(HmacError::MissingSignature)
        ));
    }

    #[test]
    fn test_hmac_validation_wrong_signature() {
        let validator = HmacValidator::new(b"test-secret".to_vec());
        // Valid hex but wrong signature
        let query = "title=Hello&signature=abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234";
        assert!(matches!(
            validator.validate(query),
            Err(HmacError::InvalidSignature(_))
        ));
    }

    #[test]
    fn test_canonical_query_order() {
        let validator = HmacValidator::new(b"secret".to_vec());

        // Parameters should be sorted alphabetically
        let mut params = BTreeMap::new();
        params.insert("z".to_string(), "last".to_string());
        params.insert("a".to_string(), "first".to_string());
        params.insert("m".to_string(), "middle".to_string());
        params.insert("signature".to_string(), "excluded".to_string());

        let canonical = validator.build_canonical_query(&params);
        assert_eq!(canonical, "a=first&m=middle&z=last");
    }

    #[test]
    fn test_url_encoding_prevents_injection() {
        let validator = HmacValidator::new(b"secret".to_vec());

        let mut params = BTreeMap::new();
        params.insert("title".to_string(), "Hello&World".to_string());
        params.insert("desc".to_string(), "key=value".to_string());

        let canonical = validator.build_canonical_query(&params);

        // Special characters should be URL-encoded
        assert!(!canonical.contains("Hello&World"));
        assert!(!canonical.contains("key=value"));
        assert_eq!(canonical, "desc=key%3Dvalue&title=Hello%26World");
    }

    #[test]
    fn test_signature_collision_prevention() {
        let validator = HmacValidator::new(b"secret".to_vec());

        // Single parameter with injected &
        let mut params1 = BTreeMap::new();
        params1.insert("a".to_string(), "1&b=2".to_string());
        let canonical1 = validator.build_canonical_query(&params1);

        // Two separate parameters
        let mut params2 = BTreeMap::new();
        params2.insert("a".to_string(), "1".to_string());
        params2.insert("b".to_string(), "2".to_string());
        let canonical2 = validator.build_canonical_query(&params2);

        // Should produce different canonical strings
        assert_ne!(canonical1, canonical2);
    }
}
