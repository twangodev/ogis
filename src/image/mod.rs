use reqwest::Client;
use std::time::Duration;

mod cache;
mod error;
mod fetch;
mod parse;
mod resolver;
mod validate;

pub use error::ImageFetchError;
pub use validate::ValidatedImage;

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

    /// Fetch image with MIME type detection
    ///
    /// Pipeline stages:
    /// 1. Check cache (raw bytes, re-detect MIME type)
    /// 2. Parse URL + validate direct IPs
    /// 3. HTTP fetch with streaming size limit (SSRF protection via GlobalResolver)
    /// 4. Validate content-type and detect MIME type
    /// 5. Store in cache (raw bytes only)
    pub async fn fetch_image(&self, url: &str) -> Result<ValidatedImage, ImageFetchError> {
        // Stage 0: Check cache first
        if let Some(cached_bytes) = self.cache.get(url).await {
            tracing::debug!("Cache hit for URL: {}", url);
            // Re-detect MIME type from cached bytes
            let mime_type = infer::get(&cached_bytes)
                .map(|k| k.mime_type().to_string())
                .unwrap_or_else(|| "image/png".to_string());
            return Ok(ValidatedImage {
                bytes: cached_bytes,
                mime_type,
            });
        }

        let parsed = parse::parse_url(url, self.allow_http)?;
        let fetched = fetch::fetch_http(parsed, &self.client, self.max_size).await?;
        let validated = validate::validate_content_type(fetched)?;

        self.cache
            .insert(url.to_string(), validated.bytes.clone())
            .await;

        Ok(validated)
    }
}
