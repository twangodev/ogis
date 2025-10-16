use std::net::IpAddr;
use url::Url;

use super::error::ImageFetchError;

/// Parsed and validated URL
#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub url: Url,
    pub original: String,
}

/// Stage 1: Parse and validate URL scheme + check for direct IPs
pub fn parse_url(url: &str) -> Result<ParsedUrl, ImageFetchError> {
    // Parse URL
    let parsed = Url::parse(url)
        .map_err(|e| ImageFetchError::InvalidUrl(format!("Failed to parse URL: {}", e)))?;

    // Only allow HTTP/HTTPS
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ImageFetchError::InvalidUrl(format!(
            "Only http/https schemes allowed, got: {}",
            parsed.scheme()
        )));
    }

    // SSRF Protection: Check if URL contains a direct IP address
    if let Some(host) = parsed.host_str() {
        if let Ok(ip) = host.parse::<IpAddr>() {
            if !ip_rfc::global(&ip) {
                tracing::warn!("Blocked direct private IP in URL: {} ({})", url, ip);
                return Err(ImageFetchError::PrivateIpBlocked(format!(
                    "Private IP address {} is not allowed",
                    ip
                )));
            }
        }
        // For hostnames, DNS resolution will be validated by SSRFSafeResolver
    }

    Ok(ParsedUrl {
        url: parsed,
        original: url.to_string(),
    })
}
