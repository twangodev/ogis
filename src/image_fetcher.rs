use base64::Engine;
use moka::future::Cache;
use reqwest::Client;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

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

pub struct ImageFetcher {
    client: Client,
    cache: Arc<Cache<String, Arc<String>>>,
    max_size: usize,
}

impl ImageFetcher {
    pub fn new(
        connect_timeout_secs: u64,
        total_timeout_secs: u64,
        max_size: usize,
        cache_size: u64,
        cache_ttl_secs: u64,
        max_redirects: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create simple reqwest client with timeouts
        let client = Client::builder()
            .timeout(Duration::from_secs(total_timeout_secs))
            .connect_timeout(Duration::from_secs(connect_timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(max_redirects))
            .build()?;

        // Initialize cache
        let cache = Cache::builder()
            .max_capacity(cache_size)
            .time_to_live(Duration::from_secs(cache_ttl_secs))
            .build();

        tracing::info!("ImageFetcher initialized with SSRF protection");

        Ok(Self {
            client,
            cache: Arc::new(cache),
            max_size,
        })
    }

    pub async fn fetch_image(&self, url: &str) -> Result<String, ImageFetchError> {
        // Check cache first
        if let Some(cached) = self.cache.get(url).await {
            tracing::debug!("Cache hit for logo URL: {}", url);
            return Ok((*cached).clone());
        }

        // Parse and validate URL
        let parsed_url = Url::parse(url)
            .map_err(|e| ImageFetchError::InvalidUrl(format!("Failed to parse URL: {}", e)))?;

        // Only allow HTTP/HTTPS
        if !matches!(parsed_url.scheme(), "http" | "https") {
            return Err(ImageFetchError::InvalidUrl(format!(
                "Only http/https schemes allowed, got: {}",
                parsed_url.scheme()
            )));
        }

        // SSRF Protection: Check if URL contains a direct IP address
        if let Some(host) = parsed_url.host_str() {
            if let Ok(ip) = host.parse::<IpAddr>() {
                if !ip_rfc::global(&ip) {
                    tracing::warn!("Blocked direct private IP in URL: {} ({})", url, ip);
                    return Err(ImageFetchError::PrivateIpBlocked(format!(
                        "Private IP address {} is not allowed",
                        ip
                    )));
                }
            } else {
                // It's a hostname - resolve and validate all IPs
                let host_str = host.to_string();
                match tokio::net::lookup_host((host_str.as_str(), 443)).await {
                    Ok(addrs) => {
                        let ips: Vec<IpAddr> = addrs.map(|addr| addr.ip()).collect();

                        if ips.is_empty() {
                            return Err(ImageFetchError::Request(
                                "No IP addresses resolved".to_string(),
                            ));
                        }

                        // Check ALL resolved IPs - if ANY are private, block
                        for ip in &ips {
                            if !ip_rfc::global(ip) {
                                tracing::warn!(
                                    "Blocked hostname {} that resolves to private IP: {}",
                                    host,
                                    ip
                                );
                                return Err(ImageFetchError::PrivateIpBlocked(format!(
                                    "Hostname {} resolves to private IP {}",
                                    host, ip
                                )));
                            }
                        }

                        tracing::debug!("Hostname {} resolved to public IPs: {:?}", host, ips);
                    }
                    Err(e) => {
                        return Err(ImageFetchError::Request(format!(
                            "DNS lookup failed: {}",
                            e
                        )));
                    }
                }
            }
        }

        tracing::info!("Fetching logo image from URL: {}", url);

        // Make HTTP request
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ImageFetchError::Request(e.to_string()))?;

        // Check status
        if !response.status().is_success() {
            return Err(ImageFetchError::Request(format!(
                "HTTP {}",
                response.status()
            )));
        }

        // Check Content-Length if available
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.max_size {
                tracing::warn!(
                    "Image from {} exceeds max size: {} > {}",
                    url,
                    content_length,
                    self.max_size
                );
                return Err(ImageFetchError::TooLarge);
            }
        }

        // Stream response with size limit enforcement
        let bytes = response.bytes().await.map_err(|e| {
            ImageFetchError::Request(format!("Failed to read response body: {}", e))
        })?;

        if bytes.len() > self.max_size {
            tracing::warn!(
                "Image from {} exceeds max size: {} > {}",
                url,
                bytes.len(),
                self.max_size
            );
            return Err(ImageFetchError::TooLarge);
        }

        // Validate content-type using magic numbers (not headers!)
        let kind = infer::get(&bytes).ok_or_else(|| {
            tracing::warn!("Could not determine file type for image from {}", url);
            ImageFetchError::InvalidContentType
        })?;

        // Allow only safe image types
        let mime_type = kind.mime_type();
        if !matches!(
            mime_type,
            "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml"
        ) {
            tracing::warn!("Invalid image type from {}: {}", url, mime_type);
            return Err(ImageFetchError::InvalidContentType);
        }

        tracing::info!(
            "Successfully fetched {} image ({} bytes) from {}",
            mime_type,
            bytes.len(),
            url
        );

        // Convert to base64
        let base64_string = base64::engine::general_purpose::STANDARD.encode(&bytes);

        // Store in cache
        let cached_value = Arc::new(base64_string.clone());
        self.cache.insert(url.to_string(), cached_value).await;

        Ok(base64_string)
    }
}
