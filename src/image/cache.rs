use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

/// Cache wrapper for raw image bytes
pub struct ImageCache {
    cache: Arc<Cache<String, Arc<Vec<u8>>>>,
}

impl ImageCache {
    /// Create a new cache with byte-based memory limit
    ///
    /// `cache_max_bytes`: Total bytes the cache can hold (not entry count)
    /// `cache_ttl_secs`: Time-to-live for cached entries in seconds
    pub fn new(cache_max_bytes: u64, cache_ttl_secs: u64) -> Self {
        let cache = Cache::builder()
            // Weigher: count actual bytes of each cached image
            .weigher(|_key: &String, value: &Arc<Vec<u8>>| -> u32 {
                value.len().try_into().unwrap_or(u32::MAX)
            })
            // max_capacity now represents total bytes (not entry count)
            .max_capacity(cache_max_bytes)
            .time_to_live(Duration::from_secs(cache_ttl_secs))
            .build();

        Self {
            cache: Arc::new(cache),
        }
    }

    pub async fn get(&self, url: &str) -> Option<Vec<u8>> {
        self.cache.get(url).await.map(|arc| (*arc).clone())
    }

    pub async fn insert(&self, url: String, bytes: Vec<u8>) {
        self.cache.insert(url, Arc::new(bytes)).await;
    }
}
