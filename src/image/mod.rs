use reqwest::Client;
use std::time::Duration;

mod cache;
mod encode;
mod error;
mod fetch;
mod parse;
mod resolver;
mod validate;

pub use error::ImageFetchError;

use cache::ImageCache;
use resolver::GlobalResolver;

pub struct ImageFetcher {
    client: Client,
    cache: ImageCache,
    max_size: usize,
    allow_http: bool,
}

impl ImageFetcher {
    pub fn new(
        connect_timeout_secs: u64,
        total_timeout_secs: u64,
        max_size: usize,
        cache_size: u64,
        cache_ttl_secs: u64,
        max_redirects: usize,
        allow_http: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create global-only DNS resolver (SSRF protection)
        let resolver = GlobalResolver::new()?;

        // Create HTTP client with custom resolver and timeouts
        let client = Client::builder()
            .timeout(Duration::from_secs(total_timeout_secs))
            .connect_timeout(Duration::from_secs(connect_timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(max_redirects))
            .dns_resolver(std::sync::Arc::new(resolver))
            .build()?;

        // Initialize cache
        let cache = ImageCache::new(cache_size, cache_ttl_secs);

        tracing::info!(
            "ImageFetcher initialized with GlobalResolver (SSRF protection), HTTP allowed: {}",
            allow_http
        );

        Ok(Self {
            client,
            cache,
            max_size,
            allow_http,
        })
    }

    /// Fetch image and convert to base64
    ///
    /// Pipeline stages:
    /// 1. Check cache (raw bytes)
    /// 2. Parse URL + validate direct IPs
    /// 3. HTTP fetch with streaming size limit (SSRF protection via GlobalResolver)
    /// 4. Validate content-type
    /// 5. Store in cache (raw bytes)
    /// 6. Encode to base64 (on-demand)
    pub async fn fetch_image(&self, url: &str) -> Result<String, ImageFetchError> {
        // Stage 0: Check cache first
        if let Some(cached_bytes) = self.cache.get(url).await {
            tracing::debug!("Cache hit for URL: {}", url);
            return Ok(encode::encode_base64_bytes(&cached_bytes));
        }

        // Stage 1: Parse URL + validate direct IPs
        let parsed = parse::parse_url(url, self.allow_http)?;

        // Stage 2: Fetch HTTP with size validation (GlobalResolver validates hostname IPs)
        let fetched = fetch::fetch_http(parsed, &self.client, self.max_size).await?;

        // Stage 3: Validate content-type
        let validated_bytes = validate::validate_content_type(fetched)?;

        // Stage 4: Store raw bytes in cache
        self.cache
            .insert(url.to_string(), validated_bytes.clone())
            .await;

        // Stage 5: Encode to base64 on-demand
        let base64 = encode::encode_base64_bytes(&validated_bytes);

        Ok(base64)
    }
}
