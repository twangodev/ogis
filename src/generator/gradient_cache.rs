//! Cache for prerendered gradient backgrounds.
//!
//! Stores `tiny_skia::Pixmap`s of the gradient layer (defs + bg fills) keyed by
//! `(template_name, gradient-affecting color overrides)`. Foreground (logo + text)
//! is rendered per-request and composited on top.

use moka::sync::Cache;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::error::GeneratorError;

/// Wrapper around a Moka byte-weighted cache holding rendered gradient pixmaps.
pub struct GradientCache {
    cache: Arc<Cache<String, Arc<tiny_skia::Pixmap>>>,
}

impl GradientCache {
    /// Create a cache with the given byte budget. The weigher counts the raw
    /// pixel buffer size of each pixmap.
    pub fn new(max_bytes: u64) -> Self {
        let cache = Cache::builder()
            .weigher(|_key: &String, value: &Arc<tiny_skia::Pixmap>| -> u32 {
                value.data().len().try_into().unwrap_or(u32::MAX)
            })
            .max_capacity(max_bytes)
            .build();

        Self {
            cache: Arc::new(cache),
        }
    }

    /// Get the pixmap for `key`, or render and cache it via `render_fn`.
    ///
    /// Concurrent callers for the same key block on a single in-flight render
    /// (Moka's `get_with` provides single-flight semantics).
    pub fn get_or_render<F>(
        &self,
        key: String,
        render_fn: F,
    ) -> Result<Arc<tiny_skia::Pixmap>, GeneratorError>
    where
        F: FnOnce() -> Result<tiny_skia::Pixmap, GeneratorError>,
    {
        // Fast path: check existing entry first so we can record hit/miss accurately.
        if let Some(existing) = self.cache.get(&key) {
            return Ok(existing);
        }

        // Use try_get_with to single-flight the render. The closure type must
        // return Result<Arc<Pixmap>, _>; on error the entry is not inserted.
        self.cache
            .try_get_with(key, || render_fn().map(Arc::new))
            .map_err(|e: Arc<GeneratorError>| match Arc::try_unwrap(e) {
                Ok(err) => err,
                Err(arc) => GeneratorError::SvgParse(arc.to_string()),
            })
    }

    /// Side-effect-free check for whether a key is currently cached. Used by
    /// callers that want to record hit/miss telemetry before invoking
    /// `get_or_render` (which inserts on miss).
    pub fn contains_key(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Total bytes currently held in the cache (for telemetry).
    pub fn weighted_size(&self) -> u64 {
        self.cache.weighted_size()
    }

    /// Approximate entry count (debug / telemetry).
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }
}

/// Build a stable cache key from the template name plus only the color overrides
/// that actually affect the gradient SVG.
///
/// `gradient_color_keys` was precomputed at template-load time by scanning the
/// gradient SVG for default hex values from each color name.
pub fn build_key(
    template_name: &str,
    gradient_color_keys: &HashSet<String>,
    color_overrides: &HashMap<String, String>,
) -> String {
    let mut filtered: Vec<(&str, &str)> = color_overrides
        .iter()
        .filter(|(k, _)| gradient_color_keys.contains(k.as_str()))
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    filtered.sort_by_key(|(k, _)| *k);

    if filtered.is_empty() {
        return template_name.to_string();
    }

    let mut suffix = String::new();
    for (i, (k, v)) in filtered.iter().enumerate() {
        if i > 0 {
            suffix.push(';');
        }
        suffix.push_str(k);
        suffix.push('=');
        suffix.push_str(v);
    }
    format!("{template_name}|{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(items: &[&str]) -> HashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn overrides(items: &[(&str, &str)]) -> HashMap<String, String> {
        items
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn key_no_overrides_is_template_name() {
        let key = build_key(
            "gradient-aurora-centered",
            &keys(&["background", "blob_violet"]),
            &HashMap::new(),
        );
        assert_eq!(key, "gradient-aurora-centered");
    }

    #[test]
    fn key_filters_to_gradient_keys() {
        let bg_keys = keys(&["background", "blob_violet"]);
        // "text" is not in bg_keys, so it should be excluded
        let ovr = overrides(&[("text", "#aabbcc"), ("background", "#112233")]);
        let key = build_key("g", &bg_keys, &ovr);
        assert_eq!(key, "g|background=#112233");
    }

    #[test]
    fn key_is_deterministic_across_iteration_order() {
        let bg_keys = keys(&["background", "blob_violet"]);
        let a = overrides(&[("background", "#112233"), ("blob_violet", "#445566")]);
        let b = overrides(&[("blob_violet", "#445566"), ("background", "#112233")]);
        assert_eq!(build_key("g", &bg_keys, &a), build_key("g", &bg_keys, &b));
    }

    #[test]
    fn non_gradient_overrides_collapse_to_same_key() {
        let bg_keys = keys(&["background"]);
        let no_ovr = build_key("g", &bg_keys, &HashMap::new());
        let with_text = build_key("g", &bg_keys, &overrides(&[("text", "#aabbcc")]));
        assert_eq!(no_ovr, with_text);
    }

    #[test]
    fn cache_returns_same_pixmap_on_hit() {
        let cache = GradientCache::new(64 * 1024 * 1024);
        let key = "k".to_string();

        let first = cache
            .get_or_render(key.clone(), || {
                tiny_skia::Pixmap::new(10, 10).ok_or(GeneratorError::PixmapCreation)
            })
            .unwrap();
        let second = cache
            .get_or_render(key, || {
                // Should not be called; cache hit
                panic!("render_fn called on hit");
            })
            .unwrap();
        assert!(Arc::ptr_eq(&first, &second));
    }
}
