#[derive(Debug)]
pub enum ImageFetchError {
    Request(String),
    TooLarge,
    InvalidContentType,
    PrivateIpBlocked(String),
    InvalidUrl(String),
}

impl std::fmt::Display for ImageFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(msg) => write!(f, "Request error: {}", msg),
            Self::TooLarge => write!(f, "Image exceeds maximum size"),
            Self::InvalidContentType => write!(f, "Invalid image content type"),
            Self::PrivateIpBlocked(msg) => write!(f, "SSRF protection: {}", msg),
            Self::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
        }
    }
}

impl std::error::Error for ImageFetchError {}
