use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

/// Cache wrapper for raw image bytes
pub struct ImageCache {
    cache: Arc<Cache<String, Arc<Vec<u8>>>>,
}

impl ImageCache {
    pub fn new(cache_size: u64, cache_ttl_secs: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(cache_size)
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
