use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageFetchError {
    #[error("Request error: {0}")]
    Request(String),

    #[error("Image exceeds maximum size")]
    TooLarge,

    #[error("Invalid image content type")]
    InvalidContentType,

    #[error("SSRF protection: {0}")]
    PrivateIpBlocked(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}
